use std::{collections::BTreeMap, fmt};

/// Fixed classifications; no source errors or caller text are retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    Identifier,
    Shape,
    Type,
    Missing,
    Unsupported,
    Budget,
}
impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_PLAN_{self:?}")
    }
}
impl std::error::Error for PlanError {}
type Result<T> = std::result::Result<T, PlanError>;
macro_rules! redacted { ($($t:ty),+ $(,)?) => {$ (impl fmt::Debug for $t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(concat!(stringify!($t), "([REDACTED])")) }
})+}; }

/// One identifier component, never SQL or a dotted path. ASCII is deliberately finite.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ident(String);
impl Ident {
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        if s.is_empty()
            || s.len() > 63
            || !s
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'"' || b == b'`')
        {
            return Err(PlanError::Identifier);
        }
        Ok(Self(s))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Postgres,
    Mysql,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    Bool,
    Text,
}
/// Finite native values; no float, decimal, opaque or callback fallback.
/// ```compile_fail
/// let value = quent_core::Value::Float(f64::NAN);
/// ```
/// ```compile_fail
/// let value = quent_core::Value::I8(128);
/// ```
#[derive(Clone, PartialEq, Eq)]
pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Bool(bool),
    Text(String),
    Null(SqlType),
}
impl Value {
    pub fn sql_type(&self) -> SqlType {
        match self {
            Self::I8(_) => SqlType::I8,
            Self::I16(_) => SqlType::I16,
            Self::I32(_) => SqlType::I32,
            Self::I64(_) => SqlType::I64,
            Self::U8(_) => SqlType::U8,
            Self::U16(_) => SqlType::U16,
            Self::U32(_) => SqlType::U32,
            Self::U64(_) => SqlType::U64,
            Self::Bool(_) => SqlType::Bool,
            Self::Text(_) => SqlType::Text,
            Self::Null(t) => *t,
        }
    }
}
/// Missing is distinct from explicit SQL NULL, and cannot be bound.
#[derive(Clone, PartialEq, Eq)]
pub enum Input {
    Missing,
    Value(Value),
}
#[derive(Clone, PartialEq, Eq)]
pub struct Table {
    path: Vec<Ident>,
    alias: Option<Ident>,
}
impl Table {
    pub fn new(path: Vec<Ident>, alias: Option<Ident>) -> Result<Self> {
        if !(1..=2).contains(&path.len()) {
            return Err(PlanError::Shape);
        }
        Ok(Self { path, alias })
    }
    pub fn column(&self, name: Ident, sql_type: SqlType, nullable: bool) -> Column {
        Column {
            table: self.clone(),
            name,
            sql_type,
            nullable,
        }
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Column {
    table: Table,
    name: Ident,
    sql_type: SqlType,
    nullable: bool,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Projection {
    column: Column,
    alias: Option<Ident>,
}
impl Projection {
    pub fn new(column: Column, alias: Option<Ident>) -> Self {
        Self { column, alias }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compare {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
#[derive(Clone, PartialEq, Eq)]
pub enum Predicate {
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
    Not(Box<Predicate>),
    Compare(Column, Compare, Value),
    In(Column, Vec<Value>),
    IsNull(Column),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Asc,
    Desc,
}
#[derive(Clone, PartialEq, Eq)]
pub struct Order {
    pub column: Column,
    pub direction: Direction,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Select,
    Count,
    Insert,
    Update,
    Delete,
}
#[derive(Clone, PartialEq, Eq)]
enum Body {
    Select(Vec<Projection>),
    Count,
    Insert(Vec<(Column, Input)>),
    Update(Vec<(Column, Input)>),
    Delete,
}
/// Construction surface. Consumed into an immutable owned operation.
#[derive(Clone, PartialEq, Eq)]
pub struct Draft {
    table: Table,
    body: Body,
    predicate: Option<Predicate>,
    order: Vec<Order>,
    limit: Option<u64>,
}
impl Draft {
    fn new(table: Table, body: Body) -> Self {
        Self {
            table,
            body,
            predicate: None,
            order: vec![],
            limit: None,
        }
    }
    pub fn select(table: Table, projection: Vec<Projection>) -> Self {
        Self::new(table, Body::Select(projection))
    }
    pub fn insert(table: Table, data: Vec<(Column, Input)>) -> Self {
        Self::new(table, Body::Insert(data))
    }
    pub fn update(table: Table, data: Vec<(Column, Input)>) -> Self {
        Self::new(table, Body::Update(data))
    }
    pub fn delete(table: Table) -> Self {
        Self::new(table, Body::Delete)
    }
    pub fn filter(mut self, predicate: Predicate) -> Self {
        self.predicate = Some(predicate);
        self
    }
    pub fn order(mut self, order: Vec<Order>) -> Self {
        self.order = order;
        self
    }
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
    /// COUNT rows matching the filter, independent of SELECT projection/order/limit.
    pub fn count(mut self) -> Result<Self> {
        if !matches!(self.body, Body::Select(_)) {
            return Err(PlanError::Shape);
        }
        self.body = Body::Count;
        self.order.clear();
        self.limit = None;
        Ok(self)
    }
    pub fn freeze(self) -> Operation {
        Operation(self)
    }
}
/// No mutable access, serialization, provider, connection or dispatch capability.
#[derive(Clone, PartialEq, Eq)]
pub struct Operation(Draft);
#[derive(Clone, PartialEq, Eq)]
pub enum FinalProjection {
    Columns(Vec<Projection>),
    Count,
    None,
}
/// Captures final semantics, including ordered tree and resolved column identity.
#[derive(Clone, PartialEq, Eq)]
pub struct Metadata {
    operation: Operation,
    projection: FinalProjection,
    family: Family,
}
impl Metadata {
    pub fn projection(&self) -> &FinalProjection {
        &self.projection
    }
    pub fn ordered_write_data(&self) -> &[(Column, Input)] {
        match &self.operation.0.body {
            Body::Insert(data) | Body::Update(data) => data,
            _ => &[],
        }
    }
    pub fn order(&self) -> &[Order] {
        &self.operation.0.order
    }
    pub fn limit(&self) -> Option<u64> {
        self.operation.0.limit
    }
    pub fn family(&self) -> Family {
        self.family
    }
    pub fn predicate(&self) -> Option<&Predicate> {
        self.operation.0.predicate.as_ref()
    }
    pub fn operation(&self) -> &Operation {
        &self.operation
    }
}
#[derive(PartialEq, Eq)]
struct Record {
    sql: String,
    args: Vec<Value>,
    metadata: Metadata,
    dialect: Dialect,
}
/// Private finalized record. No execute, authorization, serde or SQL reconstruction API.
/// ```compile_fail
/// fn execute(plan: quent_core::Plan) { plan.execute(); }
/// ```
#[derive(PartialEq, Eq)]
pub struct Plan {
    record: Record,
}
/// Explicitly sensitive borrowed inspection; never an executable/authorized handle.
/// ```compile_fail
/// fn execute(source: quent_core::SensitiveSource<'_>) { source.execute(); }
/// ```
pub struct SensitiveSource<'a> {
    record: &'a Record,
}
impl Plan {
    pub fn sensitive_source(&self) -> SensitiveSource<'_> {
        SensitiveSource {
            record: &self.record,
        }
    }
}
impl SensitiveSource<'_> {
    pub fn sql(&self) -> &str {
        &self.record.sql
    }
    pub fn args(&self) -> &[Value] {
        &self.record.args
    }
    pub fn metadata(&self) -> &Metadata {
        &self.record.metadata
    }
    pub fn dialect(&self) -> Dialect {
        self.record.dialect
    }
}
redacted!(
    Ident,
    Value,
    Input,
    Table,
    Column,
    Projection,
    Predicate,
    Order,
    Body,
    Draft,
    Operation,
    FinalProjection,
    Metadata,
    Record,
    Plan,
    SensitiveSource<'_>
);

struct Renderer<'a> {
    dialect: Dialect,
    table: &'a Table,
    sql: String,
    args: Vec<Value>,
    columns: BTreeMap<Ident, (SqlType, bool)>,
    nodes: usize,
}
impl Renderer<'_> {
    fn ident(&self, id: &Ident) -> String {
        let q = if self.dialect == Dialect::Postgres {
            '"'
        } else {
            '`'
        };
        format!("{q}{}{q}", id.0.replace(q, &format!("{q}{q}")))
    }
    fn table_sql(&self) -> String {
        self.table
            .path
            .iter()
            .map(|s| self.ident(s))
            .collect::<Vec<_>>()
            .join(".")
    }
    fn column(&mut self, c: &Column, qualified: bool) -> Result<String> {
        if &c.table != self.table {
            return Err(PlanError::Shape);
        }
        self.ty(c.sql_type)?;
        if self
            .columns
            .insert(c.name.clone(), (c.sql_type, c.nullable))
            .is_some_and(|v| v != (c.sql_type, c.nullable))
        {
            return Err(PlanError::Type);
        }
        let name = self.ident(&c.name);
        Ok(if qualified {
            format!(
                "{}.{}",
                self.ident(
                    self.table
                        .alias
                        .as_ref()
                        .unwrap_or(self.table.path.last().unwrap())
                ),
                name
            )
        } else {
            name
        })
    }
    fn ty(&self, t: SqlType) -> Result<()> {
        if self.dialect == Dialect::Postgres
            && matches!(
                t,
                SqlType::I8 | SqlType::U8 | SqlType::U16 | SqlType::U32 | SqlType::U64
            )
        {
            return Err(PlanError::Unsupported);
        }
        Ok(())
    }
    fn bind(&mut self, v: &Value) -> Result<String> {
        self.ty(v.sql_type())?;
        if matches!(v, Value::Text(s) if s.len() > 4096) || self.args.len() >= 1024 {
            return Err(PlanError::Budget);
        }
        // PostgreSQL text cannot contain NUL; refuse for both dialects consistently.
        if matches!(v, Value::Text(s) if s.contains('\0')) {
            return Err(PlanError::Type);
        }
        self.args.push(v.clone());
        Ok(if self.dialect == Dialect::Postgres {
            format!("${}", self.args.len())
        } else {
            "?".into()
        })
    }
    fn typed(&mut self, c: &Column, v: &Value) -> Result<String> {
        if c.sql_type != v.sql_type() {
            return Err(PlanError::Type);
        }
        self.bind(v)
    }
    fn predicate(&mut self, p: &Predicate, depth: usize) -> Result<String> {
        self.nodes += 1;
        if depth > 16 || self.nodes > 1024 {
            return Err(PlanError::Budget);
        }
        Ok(match p {
            Predicate::And(v) | Predicate::Or(v) => {
                if v.is_empty() {
                    return Err(PlanError::Shape);
                }
                if v.len() > 256 {
                    return Err(PlanError::Budget);
                }
                let separator = if matches!(p, Predicate::And(_)) {
                    " AND "
                } else {
                    " OR "
                };
                let parts = v
                    .iter()
                    .map(|p| self.predicate(p, depth + 1))
                    .collect::<Result<Vec<_>>>()?;
                format!("({})", parts.join(separator))
            }
            Predicate::Not(p) => format!("(NOT {})", self.predicate(p, depth + 1)?),
            Predicate::Compare(c, op, v) => {
                let col = self.column(c, true)?;
                if !matches!(op, Compare::Eq | Compare::Ne)
                    && matches!(c.sql_type, SqlType::Text | SqlType::Bool)
                {
                    return Err(PlanError::Unsupported);
                }
                let value = self.typed(c, v)?;
                let op = match op {
                    Compare::Eq => "=",
                    Compare::Ne => "<>",
                    Compare::Lt => "<",
                    Compare::Le => "<=",
                    Compare::Gt => ">",
                    Compare::Ge => ">=",
                };
                format!("({col} {op} {value})")
            }
            Predicate::In(c, v) => {
                let col = self.column(c, true)?;
                if v.len() > 256 {
                    return Err(PlanError::Budget);
                }
                if v.is_empty() {
                    "(1 = 0)".into()
                } else {
                    let args = v
                        .iter()
                        .map(|v| self.typed(c, v))
                        .collect::<Result<Vec<_>>>()?;
                    format!("({col} IN ({}))", args.join(", "))
                }
            }
            Predicate::IsNull(c) => format!("({} IS NULL)", self.column(c, true)?),
        })
    }
    fn data(&mut self, data: &[(Column, Input)]) -> Result<(Vec<String>, Vec<String>)> {
        if data.is_empty() {
            return Err(PlanError::Shape);
        }
        if data.len() > 256 {
            return Err(PlanError::Budget);
        }
        let mut cols = vec![];
        let mut args = vec![];
        for (c, v) in data {
            let col = self.column(c, false)?;
            if cols.contains(&col) {
                return Err(PlanError::Shape);
            }
            let Input::Value(v) = v else {
                return Err(PlanError::Missing);
            };
            if !c.nullable && matches!(v, Value::Null(_)) {
                return Err(PlanError::Type);
            }
            cols.push(col);
            args.push(self.typed(c, v)?);
        }
        Ok((cols, args))
    }
}
impl Operation {
    /// Pure, bounded rendering from this owned snapshot. Does not evaluate risk.
    pub fn finalize(&self, dialect: Dialect) -> Result<Plan> {
        let d = &self.0;
        let read = matches!(d.body, Body::Select(_) | Body::Count);
        if !read && (!d.order.is_empty() || d.limit.is_some() || d.table.alias.is_some()) {
            return Err(PlanError::Unsupported);
        }
        if matches!(d.body, Body::Insert(_)) && d.predicate.is_some() {
            return Err(PlanError::Shape);
        }
        if d.order.len() > 256 {
            return Err(PlanError::Budget);
        }
        let mut r = Renderer {
            dialect,
            table: &d.table,
            sql: String::new(),
            args: vec![],
            columns: BTreeMap::new(),
            nodes: 0,
        };
        let table = r.table_sql();
        let from = if let Some(alias) = &d.table.alias {
            format!("{table} AS {}", r.ident(alias))
        } else {
            table.clone()
        };
        let (family, projection) = match &d.body {
            Body::Select(p) => {
                if p.is_empty() {
                    return Err(PlanError::Shape);
                }
                if p.len() > 256 {
                    return Err(PlanError::Budget);
                }
                let mut output_names = Vec::new();
                let cols = p
                    .iter()
                    .map(|p| {
                        let name = p.alias.as_ref().unwrap_or(&p.column.name);
                        if output_names.contains(name) {
                            return Err(PlanError::Shape);
                        }
                        output_names.push(name.clone());
                        let col = r.column(&p.column, true)?;
                        Ok(if let Some(a) = &p.alias {
                            format!("{col} AS {}", r.ident(a))
                        } else {
                            col
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                r.sql = format!("SELECT {} FROM {from}", cols.join(", "));
                (Family::Select, FinalProjection::Columns(p.clone()))
            }
            Body::Count => {
                r.sql = format!("SELECT COUNT(*) FROM {from}");
                (Family::Count, FinalProjection::Count)
            }
            Body::Insert(data) => {
                let (cols, args) = r.data(data)?;
                r.sql = format!(
                    "INSERT INTO {table} ({}) VALUES ({})",
                    cols.join(", "),
                    args.join(", ")
                );
                (Family::Insert, FinalProjection::None)
            }
            Body::Update(data) => {
                let (cols, args) = r.data(data)?;
                r.sql = format!(
                    "UPDATE {table} SET {}",
                    cols.iter()
                        .zip(args)
                        .map(|(c, a)| format!("{c} = {a}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                (Family::Update, FinalProjection::None)
            }
            Body::Delete => {
                r.sql = format!("DELETE FROM {table}");
                (Family::Delete, FinalProjection::None)
            }
        };
        if let Some(p) = &d.predicate {
            let p = r.predicate(p, 0)?;
            r.sql.push_str(&format!(" WHERE {p}"));
        }
        if !d.order.is_empty() {
            let order = d
                .order
                .iter()
                .map(|o| {
                    Ok(format!(
                        "{} {}",
                        r.column(&o.column, true)?,
                        if o.direction == Direction::Asc {
                            "ASC"
                        } else {
                            "DESC"
                        }
                    ))
                })
                .collect::<Result<Vec<_>>>()?;
            r.sql.push_str(&format!(" ORDER BY {}", order.join(", ")));
        }
        if let Some(n) = d.limit {
            let n = i64::try_from(n).map_err(|_| PlanError::Type)?;
            let arg = r.bind(&Value::I64(n))?;
            r.sql.push_str(&format!(" LIMIT {arg}"));
        }
        if r.sql.len() > 131072 {
            return Err(PlanError::Budget);
        }
        Ok(Plan {
            record: Record {
                sql: r.sql,
                args: r.args,
                metadata: Metadata {
                    operation: self.clone(),
                    projection,
                    family,
                },
                dialect,
            },
        })
    }
}

mod inspection;
pub use inspection::*;

mod context;
pub use context::*;
