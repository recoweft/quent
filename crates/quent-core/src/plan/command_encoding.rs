//! Version 1 typed, length-delimited canonical tokens. No Debug/Display/serde,
//! hashes, SQL-only equality, normalization, sorting or ambient state.
use super::*;
/// Sensitive output only. Bytes never reconstruct an operation or linear handle.
pub struct CommandCanonical(pub(super) Vec<u8>);
impl CommandCanonical {
    pub fn sensitive_bytes(&self) -> &[u8] {
        &self.0
    }
}
redacted!(CommandCanonical);
pub(super) struct Encoder {
    bytes: Vec<u8>,
}
impl Encoder {
    pub(super) fn new(header: &[u8; 4]) -> Self {
        Self {
            bytes: header.to_vec(),
        }
    }
    fn token(&mut self, tag: u8, bytes: &[u8]) -> Result<()> {
        if self
            .bytes
            .len()
            .saturating_add(bytes.len())
            .saturating_add(5)
            > 1_048_576
        {
            return Err(PlanError::Budget);
        }
        self.bytes.push(tag);
        self.bytes
            .extend_from_slice(&(bytes.len() as u32).to_be_bytes());
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    pub(super) fn number(&mut self, n: u64) -> Result<()> {
        self.token(8, &n.to_be_bytes())
    }
    pub(super) fn boolean(&mut self, b: bool) -> Result<()> {
        self.token(9, &[u8::from(b)])
    }
    pub(super) fn text(&mut self, s: &str) -> Result<()> {
        if s.len() > 4096 {
            return Err(PlanError::Budget);
        }
        self.token(10, s.as_bytes())
    }
    pub(super) fn bytes(&mut self, b: &[u8]) -> Result<()> {
        if b.len() > 131072 {
            return Err(PlanError::Budget);
        }
        self.token(11, b)
    }
    pub(super) fn list(&mut self, f: impl FnOnce(&mut Self) -> Result<()>) -> Result<()> {
        let start = self.bytes.len();
        self.token(128, &[])?;
        f(self)?;
        let n = (self.bytes.len() - start - 5) as u32;
        self.bytes[start + 1..start + 5].copy_from_slice(&n.to_be_bytes());
        Ok(())
    }
    pub(super) fn ty(&mut self, t: SqlType) -> Result<()> {
        self.token(
            12,
            &[match t {
                SqlType::I8 => 1,
                SqlType::I16 => 2,
                SqlType::I32 => 3,
                SqlType::I64 => 4,
                SqlType::U8 => 5,
                SqlType::U16 => 6,
                SqlType::U32 => 7,
                SqlType::U64 => 8,
                SqlType::Bool => 9,
                SqlType::Text => 10,
            }],
        )
    }
    pub(super) fn value(&mut self, v: &Value) -> Result<()> {
        match v {
            Value::I8(n) => self.token(1, &n.to_be_bytes()),
            Value::I16(n) => self.token(2, &n.to_be_bytes()),
            Value::I32(n) => self.token(3, &n.to_be_bytes()),
            Value::I64(n) => self.token(4, &n.to_be_bytes()),
            Value::U8(n) => self.token(5, &n.to_be_bytes()),
            Value::U16(n) => self.token(6, &n.to_be_bytes()),
            Value::U32(n) => self.token(7, &n.to_be_bytes()),
            Value::U64(n) => self.token(8, &n.to_be_bytes()),
            Value::Bool(n) => self.boolean(*n),
            Value::Text(n) => self.text(n),
            Value::Null(t) => self.list(|e| e.ty(*t)),
        }
    }
    pub(super) fn values(&mut self, vs: &[Value]) -> Result<()> {
        self.list(|e| {
            for v in vs {
                e.value(v)?;
            }
            Ok(())
        })
    }
    pub(super) fn table(&mut self, t: &Table) -> Result<()> {
        self.list(|e| {
            e.list(|e| {
                for p in &t.path {
                    e.text(&p.0)?;
                }
                Ok(())
            })?;
            e.list(|e| {
                if let Some(a) = &t.alias {
                    e.text(&a.0)?;
                }
                Ok(())
            })
        })
    }
    pub(super) fn column(&mut self, c: &Column) -> Result<()> {
        self.list(|e| {
            e.table(&c.table)?;
            e.text(&c.name.0)?;
            e.ty(c.sql_type)?;
            e.boolean(c.nullable)
        })
    }
    pub(super) fn columns(&mut self, cs: &[Column]) -> Result<()> {
        self.list(|e| {
            for c in cs {
                e.column(c)?;
            }
            Ok(())
        })
    }
    fn data(&mut self, ds: &[(Column, Input)]) -> Result<()> {
        self.list(|e| {
            for (c, v) in ds {
                e.list(|e| {
                    e.column(c)?;
                    e.list(|e| {
                        if let Input::Value(v) = v {
                            e.value(v)?;
                        }
                        Ok(())
                    })
                })?;
            }
            Ok(())
        })
    }
    fn predicate(&mut self, p: &Predicate) -> Result<()> {
        self.list(|e| {
            match p {
                Predicate::And(ps) | Predicate::Or(ps) => {
                    e.number(if matches!(p, Predicate::And(_)) { 1 } else { 2 })?;
                    for p in ps {
                        e.predicate(p)?;
                    }
                }
                Predicate::Not(p) => {
                    e.number(3)?;
                    e.predicate(p)?;
                }
                Predicate::Compare(c, op, v) => {
                    e.number(4)?;
                    e.column(c)?;
                    e.number(match op {
                        Compare::Eq => 1,
                        Compare::Ne => 2,
                        Compare::Lt => 3,
                        Compare::Le => 4,
                        Compare::Gt => 5,
                        Compare::Ge => 6,
                    })?;
                    e.value(v)?;
                }
                Predicate::In(c, vs) => {
                    e.number(5)?;
                    e.column(c)?;
                    e.values(vs)?;
                }
                Predicate::IsNull(c) => {
                    e.number(6)?;
                    e.column(c)?;
                }
            }
            Ok(())
        })
    }
    fn projections(&mut self, ps: &[Projection]) -> Result<()> {
        self.list(|e| {
            for p in ps {
                e.list(|e| {
                    e.column(&p.column)?;
                    e.list(|e| {
                        if let Some(a) = &p.alias {
                            e.text(&a.0)?;
                        }
                        Ok(())
                    })
                })?;
            }
            Ok(())
        })
    }
    pub(super) fn operation(&mut self, o: &Operation) -> Result<()> {
        o.check_budget()?;
        self.list(|e| {
            let d = &o.0;
            e.table(&d.table)?;
            match &d.body {
                Body::Select(ps) => {
                    e.number(1)?;
                    e.projections(ps)?;
                }
                Body::Count => e.number(2)?,
                Body::Insert(ds) => {
                    e.number(3)?;
                    e.data(ds)?;
                }
                Body::Update(ds) => {
                    e.number(4)?;
                    e.data(ds)?;
                }
                Body::Delete => e.number(5)?,
            }
            e.list(|e| {
                if let Some(p) = &d.predicate {
                    e.predicate(p)?;
                }
                Ok(())
            })?;
            e.list(|e| {
                for o in &d.order {
                    e.column(&o.column)?;
                    e.number(if o.direction == Direction::Asc { 1 } else { 2 })?;
                }
                Ok(())
            })?;
            e.list(|e| {
                if let Some(n) = d.limit {
                    e.number(n)?;
                }
                Ok(())
            })?;
            e.projections(&d.returning)?;
            e.list(|e| {
                if let Some((c, v)) = &d.version {
                    e.column(c)?;
                    e.value(v)?;
                }
                Ok(())
            })
        })
    }
    pub(super) fn finish(self) -> Vec<u8> {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_tags_and_bytes_are_injective_and_bounded() {
        let values = [
            Value::I8(0),
            Value::I16(0),
            Value::I32(0),
            Value::I64(0),
            Value::U8(0),
            Value::U16(0),
            Value::U32(0),
            Value::U64(0),
            Value::Bool(false),
            Value::Text(String::new()),
            Value::Null(SqlType::I64),
            Value::Null(SqlType::Text),
        ];
        let encoded: Vec<_> = values
            .iter()
            .map(|v| {
                let mut e = Encoder::new(b"TEST");
                e.value(v).unwrap();
                e.finish()
            })
            .collect();
        for (i, a) in encoded.iter().enumerate() {
            for b in &encoded[..i] {
                assert_ne!(a, b);
            }
        }
        let mut a = Encoder::new(b"TEST");
        a.text("utf8").unwrap();
        let mut b = Encoder::new(b"TEST");
        b.bytes(b"utf8").unwrap();
        assert_ne!(a.finish(), b.finish());
        let mut e = Encoder::new(b"TEST");
        assert_eq!(e.text(&"a".repeat(4097)), Err(PlanError::Budget));
        assert_eq!(e.bytes(&vec![0; 131073]), Err(PlanError::Budget));
        let mut a = Encoder::new(b"TEST");
        a.values(&[Value::I64(1), Value::I64(2)]).unwrap();
        let mut b = Encoder::new(b"TEST");
        b.values(&[Value::I64(2), Value::I64(1)]).unwrap();
        assert_ne!(a.finish(), b.finish());
    }
}
