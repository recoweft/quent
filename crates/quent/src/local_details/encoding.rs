//! Private bounded output-only encoding. Never use redacted Debug as source data.
#![cfg(target_os = "linux")]
use super::DetailError;
use quent_core::*;
use std::fmt::{self, Write};
const MAX_BYTES: usize = 16 * 1024 * 1024;
struct Json(String);
impl Write for Json {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if s.len() > MAX_BYTES - self.0.len() {
            return Err(fmt::Error);
        }
        self.0.push_str(s);
        Ok(())
    }
}
impl Json {
    fn text(&mut self, s: &str) -> fmt::Result {
        self.write_char('"')?;
        for c in s.chars() {
            match c {
                '"' => self.write_str("\\\"")?,
                '\\' => self.write_str("\\\\")?,
                c if c <= '\u{1f}' => write!(self, "\\u{:04x}", c as u32)?,
                c => self.write_char(c)?,
            }
        }
        self.write_char('"')
    }
    fn array<T>(&mut self, items: &[T], f: fn(&mut Self, &T) -> fmt::Result) -> fmt::Result {
        self.write_char('[')?;
        for (i, v) in items.iter().enumerate() {
            if i != 0 {
                self.write_char(',')?;
            }
            f(self, v)?;
        }
        self.write_char(']')
    }
    fn ident(&mut self, v: &Ident) -> fmt::Result {
        self.text(v.sensitive_text())
    }
    fn alias(&mut self, v: Option<&Ident>) -> fmt::Result {
        match v {
            Some(v) => self.ident(v),
            None => self.write_str("null"),
        }
    }
    fn table(&mut self, t: &Table) -> fmt::Result {
        self.write_str("{\"path\":")?;
        self.array(t.sensitive_path(), Self::ident)?;
        self.write_str(",\"alias\":")?;
        self.alias(t.sensitive_alias())?;
        self.write_char('}')
    }
    fn column(&mut self, c: &Column) -> fmt::Result {
        self.write_str("{\"table\":")?;
        self.table(c.sensitive_table())?;
        self.write_str(",\"name\":")?;
        self.ident(c.sensitive_name())?;
        self.write_str(",\"type\":")?;
        self.text(ty(c.sql_type()))?;
        write!(self, ",\"nullable\":{}}}", c.nullable())
    }
    fn projection(&mut self, p: &Projection) -> fmt::Result {
        self.write_str("{\"column\":")?;
        self.column(p.sensitive_column())?;
        self.write_str(",\"alias\":")?;
        self.alias(p.sensitive_alias())?;
        self.write_char('}')
    }
    fn value(&mut self, v: &Value) -> fmt::Result {
        self.write_str("{\"type\":")?;
        self.text(ty(v.sql_type()))?;
        self.write_str(",\"value\":")?;
        // Integers are decimal strings: even non-Rust consumers retain u64 exactly.
        match v {
            Value::Null(_) => self.write_str("null")?,
            Value::Text(s) => self.text(s)?,
            Value::Bool(b) => write!(self, "{b}")?,
            Value::I8(n) => self.text(&n.to_string())?,
            Value::I16(n) => self.text(&n.to_string())?,
            Value::I32(n) => self.text(&n.to_string())?,
            Value::I64(n) => self.text(&n.to_string())?,
            Value::U8(n) => self.text(&n.to_string())?,
            Value::U16(n) => self.text(&n.to_string())?,
            Value::U32(n) => self.text(&n.to_string())?,
            Value::U64(n) => self.text(&n.to_string())?,
        }
        self.write_char('}')
    }
    fn data(&mut self, pair: &(Column, Input)) -> fmt::Result {
        self.write_str("{\"column\":")?;
        self.column(&pair.0)?;
        self.write_str(",\"input\":")?;
        match &pair.1 {
            Input::Missing => self.write_str("{\"missing\":true}")?,
            Input::Value(v) => self.value(v)?,
        }
        self.write_char('}')
    }
    fn predicate(&mut self, p: &Predicate) -> fmt::Result {
        self.write_str("{\"op\":")?;
        match p {
            Predicate::And(ps) | Predicate::Or(ps) => {
                self.text(if matches!(p, Predicate::And(_)) {
                    "and"
                } else {
                    "or"
                })?;
                self.write_str(",\"children\":")?;
                self.array(ps, Self::predicate)?;
            }
            Predicate::Not(p) => {
                self.text("not")?;
                self.write_str(",\"child\":")?;
                self.predicate(p)?;
            }
            Predicate::Compare(c, op, v) => {
                self.text(match op {
                    Compare::Eq => "eq",
                    Compare::Ne => "ne",
                    Compare::Lt => "lt",
                    Compare::Le => "le",
                    Compare::Gt => "gt",
                    Compare::Ge => "ge",
                })?;
                self.write_str(",\"column\":")?;
                self.column(c)?;
                self.write_str(",\"value\":")?;
                self.value(v)?;
            }
            Predicate::In(c, vs) => {
                self.text("in")?;
                self.write_str(",\"column\":")?;
                self.column(c)?;
                self.write_str(",\"values\":")?;
                self.array(vs, Self::value)?;
            }
            Predicate::IsNull(c) => {
                self.text("is_null")?;
                self.write_str(",\"column\":")?;
                self.column(c)?;
            }
        }
        self.write_char('}')
    }
    fn record(&mut self, source: SensitiveSource<'_>) -> fmt::Result {
        self.write_str("{\"kind\":\"quent_local_source\",\"version\":1,\"dialect\":")?;
        self.text(match source.dialect() {
            Dialect::Postgres => "postgres",
            Dialect::Mysql => "mysql",
        })?;
        self.write_str(",\"sql\":")?;
        self.text(source.sql())?;
        self.write_str(",\"args\":")?;
        self.array(source.args(), Self::value)?;
        self.write_str(",\"metadata\":{\"table\":")?;
        self.table(source.sensitive_table())?;
        let m = source.metadata();
        self.write_str(",\"family\":")?;
        self.text(match m.family() {
            Family::Select => "select",
            Family::Count => "count",
            Family::Insert => "insert",
            Family::Update => "update",
            Family::Delete => "delete",
        })?;
        self.write_str(",\"projection\":")?;
        match m.projection() {
            FinalProjection::Columns(ps) => self.array(ps, Self::projection)?,
            FinalProjection::Count => self.text("count")?,
            FinalProjection::None => self.write_str("null")?,
        }
        self.write_str(",\"predicate\":")?;
        match m.predicate() {
            Some(p) => self.predicate(p)?,
            None => self.write_str("null")?,
        }
        self.write_str(",\"write_data\":")?;
        self.array(m.ordered_write_data(), Self::data)?;
        self.write_str(",\"order\":")?;
        self.array(m.order(), |j, o| {
            j.write_str("{\"column\":")?;
            j.column(&o.column)?;
            j.write_str(",\"direction\":")?;
            j.text(match o.direction {
                Direction::Asc => "asc",
                Direction::Desc => "desc",
            })?;
            j.write_char('}')
        })?;
        self.write_str(",\"limit\":")?;
        match m.limit() {
            Some(n) => self.text(&n.to_string())?,
            None => self.write_str("null")?,
        }
        self.write_str(",\"version\":")?;
        match m.sensitive_version() {
            Some((c, v)) => {
                self.write_str("{\"column\":")?;
                self.column(c)?;
                self.write_str(",\"expected\":")?;
                self.value(v)?;
                self.write_char('}')?;
            }
            None => self.write_str("null")?,
        }
        write!(
            self,
            ",\"caller_missing_where\":{}}}}}",
            source.caller_missing_where()
        )
    }
}
fn ty(t: SqlType) -> &'static str {
    match t {
        SqlType::I8 => "i8",
        SqlType::I16 => "i16",
        SqlType::I32 => "i32",
        SqlType::I64 => "i64",
        SqlType::U8 => "u8",
        SqlType::U16 => "u16",
        SqlType::U32 => "u32",
        SqlType::U64 => "u64",
        SqlType::Bool => "bool",
        SqlType::Text => "text",
    }
}
pub(super) fn encode(source: SensitiveSource<'_>) -> Result<String, DetailError> {
    let mut json = Json(String::new());
    json.record(source).map_err(|_| DetailError::Budget)?;
    Ok(json.0)
}

/// Restricted output-only migration artifact: no source parser or apply transport.
pub(super) fn migration(
    source: quent_core::migration::SensitiveMigration<'_>,
) -> Result<String, DetailError> {
    let mut out = Json(String::new());
    (|| -> fmt::Result {
        out.write_str("{\"kind\":\"quent_local_migration\",\"version\":1,\"sql\":")?;
        out.array(source.sql(), |o, s| o.text(s))?;
        out.write_str(",\"differences\":[")?;
        for (i, (area, kind, support, table, name)) in source.differences().enumerate() {
            if i > 0 { out.write_char(',')?; }
            write!(out, "{{\"area\":\"{area:?}\",\"change\":\"{kind:?}\",\"support\":\"{support:?}\",\"table\":")?;
            if let Some(table) = table { let (namespace, table) = table.sensitive_parts(); out.write_char('[')?; out.ident(namespace)?; out.write_char(',')?; out.ident(table)?; out.write_char(']')?; } else { out.write_str("null")?; }
            out.write_str(",\"name\":")?; out.alias(name)?; out.write_char('}')?;
        }
        out.write_str("]}")
    })().map_err(|_| DetailError::Budget)?;
    Ok(out.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn structural_metadata_all_predicates_alias_order_limit_and_count_are_lossless() {
        for d in [Dialect::Postgres, Dialect::Mysql] {
            let t = Table::new(
                vec![
                    Ident::new("canary_schema").unwrap(),
                    Ident::new("canary_table").unwrap(),
                ],
                Some(Ident::new("canary_alias").unwrap()),
            )
            .unwrap();
            let c = t.column(Ident::new("canary_column").unwrap(), SqlType::I64, true);
            let predicate = Predicate::And(vec![
                Predicate::Or(vec![
                    Predicate::Compare(c.clone(), Compare::Ne, Value::I64(-1)),
                    Predicate::IsNull(c.clone()),
                ]),
                Predicate::Not(Box::new(Predicate::In(
                    c.clone(),
                    vec![Value::I64(0), Value::Null(SqlType::I64)],
                ))),
                Predicate::Compare(c.clone(), Compare::Lt, Value::I64(1)),
                Predicate::Compare(c.clone(), Compare::Le, Value::I64(2)),
                Predicate::Compare(c.clone(), Compare::Gt, Value::I64(3)),
                Predicate::Compare(c.clone(), Compare::Ge, Value::I64(4)),
            ]);
            let draft = Draft::select(
                t.clone(),
                vec![Projection::new(
                    c.clone(),
                    Some(Ident::new("canary_output").unwrap()),
                )],
            )
            .filter(predicate)
            .order(vec![
                Order {
                    column: c.clone(),
                    direction: Direction::Desc,
                },
                Order {
                    column: c,
                    direction: Direction::Asc,
                },
            ])
            .limit(u64::MAX >> 1);
            let p = draft.clone().freeze().finalize(d).unwrap();
            let j: serde_json::Value =
                serde_json::from_str(&encode(p.sensitive_source()).unwrap()).unwrap();
            let tj =
                serde_json::json!({"path":["canary_schema","canary_table"],"alias":"canary_alias"});
            let cj =
                serde_json::json!({"table":tj,"name":"canary_column","type":"i64","nullable":true});
            let val = |v: Option<&str>| serde_json::json!({"type":"i64","value":v});
            assert!(j["sql"] == p.sensitive_source().sql());
            assert!(
                j["args"]
                    == serde_json::json!([
                        val(Some("-1")),
                        val(Some("0")),
                        val(None),
                        val(Some("1")),
                        val(Some("2")),
                        val(Some("3")),
                        val(Some("4")),
                        val(Some("9223372036854775807"))
                    ])
            );
            assert!(
                j["metadata"]
                    == serde_json::json!({"table":tj,"family":"select","projection":[{"column":cj,"alias":"canary_output"}],"predicate":{"op":"and","children":[{"op":"or","children":[{"op":"ne","column":cj,"value":val(Some("-1"))},{"op":"is_null","column":cj}]},{"op":"not","child":{"op":"in","column":cj,"values":[val(Some("0")),val(None)]}},{"op":"lt","column":cj,"value":val(Some("1"))},{"op":"le","column":cj,"value":val(Some("2"))},{"op":"gt","column":cj,"value":val(Some("3"))},{"op":"ge","column":cj,"value":val(Some("4"))}]},"write_data":[],"order":[{"column":cj,"direction":"desc"},{"column":cj,"direction":"asc"}],"limit":"9223372036854775807","version":null,"caller_missing_where":false})
            );
            let p = draft.count().unwrap().freeze().finalize(d).unwrap();
            let j: serde_json::Value =
                serde_json::from_str(&encode(p.sensitive_source()).unwrap()).unwrap();
            assert!(j["metadata"]["projection"] == "count" && j["metadata"]["family"] == "count");
            assert!(
                j["metadata"]["limit"].is_null() && j["metadata"]["order"] == serde_json::json!([])
            );
        }
    }
    #[test]
    fn bounded_writer_refuses_without_truncating_to_valid_success() {
        let mut json = Json(String::new());
        assert!(json.write_str(&"x".repeat(MAX_BYTES)).is_ok());
        assert!(json.write_str("x").is_err());
        assert_eq!(json.0.len(), MAX_BYTES);
    }
}
