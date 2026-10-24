//! Test-only source records. Never a trusted context, permit, planner or executor.
use serde::{
    Deserialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use std::{collections::BTreeSet, fmt};

pub const MAX_BYTES: usize = 256 * 1024;
// All numbers in the wire format are strings: integer/decimal lexemes stay exact.
// A separate visitor rejects duplicate keys before typed deserialization (including
// keys in externally tagged enums), floats, and excessive depth/elements/strings.
struct Strict(Value);
impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Strict;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("bounded fixture value without JSON numbers")
            }
            fn visit_bool<E: de::Error>(self, b: bool) -> Result<Strict, E> {
                Ok(Strict(Value::Bool(b)))
            }
            fn visit_str<E: de::Error>(self, s: &str) -> Result<Strict, E> {
                if s.len() > 4096 {
                    return Err(E::custom("string budget"));
                }
                Ok(Strict(Value::String(s.into())))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Strict, A::Error> {
                let mut m = serde_json::Map::new();
                while let Some(k) = a.next_key::<String>()? {
                    if k.len() > 128 || m.len() >= 64 || m.contains_key(&k) {
                        return Err(de::Error::custom("field budget or duplicate"));
                    }
                    m.insert(k, a.next_value::<Strict>()?.0);
                }
                Ok(Strict(Value::Object(m)))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Strict, A::Error> {
                let mut v = Vec::new();
                while let Some(x) = a.next_element::<Strict>()? {
                    if v.len() >= 256 {
                        return Err(de::Error::custom("element budget"));
                    }
                    v.push(x.0);
                }
                Ok(Strict(Value::Array(v)))
            }
        }
        d.deserialize_any(V)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dialect {
    Postgres,
    Mysql,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    Select,
    Update,
    Delete,
    Insert,
    Validate,
    Execute,
    Review,
    Command,
    Manifest,
    Explain,
    Migration,
    Mcp,
    Parse,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Input {
    Missing {
        sql_type: String,
    },
    Null {
        sql_type: String,
    },
    Signed {
        bits: String,
        lexeme: String,
    },
    Unsigned {
        bits: String,
        lexeme: String,
    },
    Decimal {
        precision: String,
        scale: String,
        lexeme: String,
    },
    Text {
        value: String,
    },
    Bool {
        value: bool,
    },
}
impl Input {
    pub fn validate(&self) -> Result<(), String> {
        let bad = || "invalid typed input".to_string();
        match self {
            Self::Signed { bits, lexeme } | Self::Unsigned { bits, lexeme } => {
                let b = bits.parse::<u32>().map_err(|_| bad())?;
                if ![8, 16, 32, 64].contains(&b) {
                    return Err(bad());
                }
                let digits = lexeme.strip_prefix('-').unwrap_or(lexeme);
                if digits.is_empty() || !digits.bytes().all(|c| c.is_ascii_digit()) {
                    return Err(bad());
                }
                if matches!(self, Self::Signed { .. }) {
                    let n = lexeme.parse::<i128>().map_err(|_| bad())?;
                    if n < -(1_i128 << (b - 1)) || n >= (1_i128 << (b - 1)) {
                        return Err(bad());
                    }
                } else {
                    let n = lexeme.parse::<u128>().map_err(|_| bad())?;
                    if n >= (1_u128 << b) {
                        return Err(bad());
                    }
                }
            }
            Self::Decimal {
                precision,
                scale,
                lexeme,
            } => {
                let p = precision.parse::<usize>().map_err(|_| bad())?;
                let s = scale.parse::<usize>().map_err(|_| bad())?;
                let raw = lexeme.strip_prefix('-').unwrap_or(lexeme);
                let parts: Vec<_> = raw.split('.').collect();
                if p == 0
                    || p > 65
                    || s > p
                    || parts.len() > 2
                    || parts
                        .iter()
                        .any(|x| x.is_empty() || !x.bytes().all(|c| c.is_ascii_digit()))
                    || parts.iter().map(|x| x.len()).sum::<usize>() > p
                    || parts.get(1).map_or(0, |x| x.len()) != s
                {
                    return Err(bad());
                }
            }
            Self::Missing { sql_type } | Self::Null { sql_type } => {
                if !["i64", "text", "decimal", "bool"].contains(&sql_type.as_str()) {
                    return Err(bad());
                }
            }
            _ => {}
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "node", rename_all = "snake_case", deny_unknown_fields)]
pub enum Filter {
    Absent,
    And { children: Vec<Filter> },
    Or { children: Vec<Filter> },
    Not { child: Box<Filter> },
    Eq { column: String, value: Input },
    Gt { column: String, value: Input },
    In { column: String, values: Vec<Input> },
    IsNull { column: String },
}
impl Filter {
    fn validate(&self, depth: usize) -> Result<(), String> {
        if depth > 16 {
            return Err("condition depth".into());
        }
        match self {
            Self::And { children } | Self::Or { children } => {
                if children.is_empty() {
                    return Err("empty logical node".into());
                }
                for c in children {
                    c.validate(depth + 1)?;
                }
            }
            Self::Not { child } => child.validate(depth + 1)?,
            Self::Eq { value, .. } | Self::Gt { value, .. } => value.validate()?,
            Self::In { values, .. } => {
                for v in values {
                    v.validate()?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Column {
    pub name: String,
    pub sql_type: String,
    pub nullable: bool,
    pub generated: bool,
    pub readonly: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    pub target: String,
    pub generation: String,
    pub columns: Vec<Column>,
    pub primary_key: Vec<String>,
    pub nullable_unique: Vec<String>,
    pub limits: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Preconditions {
    pub schemas: Vec<Schema>,
    pub policy: String,
    pub tenant_provenance: String,
    pub tenant: Input,
    pub fresh_claim: String,
    pub allow_claim: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shape {
    pub operation: Operation,
    pub projection: Vec<String>,
    pub input_columns: Vec<String>,
    pub filter: Filter,
    pub rows: Vec<Vec<Input>>,
    pub steps: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Risk {
    Low,
    Medium,
    High,
    Destructive,
    Blocked,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Precision {
    Exact,
    Broad,
    Unknown,
    Partial,
    Unsupported,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expected {
    pub diagnostics: Vec<String>,
    pub risk: Risk,
    pub precision: Precision,
    pub dispatch: String,
    pub provider_calls: String,
    pub tx: Vec<String>,
    pub outcome: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub name: String,
    pub shape: Shape,
    pub expected: Expected,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub owners: Vec<String>,
    pub dialects: Vec<Dialect>,
    pub preconditions: Preconditions,
    pub scenarios: Vec<Scenario>,
    pub evidence: Vec<String>,
    pub unverified: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Register {
    pub kind: String,
    pub version: String,
    pub cases: Vec<Case>,
}

pub fn load(bytes: &[u8]) -> Result<Register, String> {
    if bytes.len() > MAX_BYTES {
        return Err("byte budget".into());
    }
    let strict: Strict = serde_json::from_slice(bytes).map_err(|_| "invalid bounded JSON")?;
    fn depth(v: &Value, n: usize) -> Result<(), String> {
        if n > 48 {
            return Err("depth budget".into());
        }
        match v {
            Value::Array(a) => {
                for v in a {
                    depth(v, n + 1)?;
                }
            }
            Value::Object(m) => {
                for v in m.values() {
                    depth(v, n + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    depth(&strict.0, 0)?;
    let r: Register = serde_json::from_value(strict.0).map_err(|_| "invalid fixture fields")?;
    if r.kind != "quent-contract-fixtures" || r.version != "1" || r.cases.len() != 24 {
        return Err("kind/version/coverage".into());
    }
    let mut seen = BTreeSet::new();
    for c in &r.cases {
        if !seen.insert(c.id.clone())
            || !c.evidence.iter().map(String::as_str).eq([
                "documented",
                "fixture_validated",
                "runtime_pending",
            ])
            || c.unverified.is_empty()
        {
            return Err("case identity/evidence".into());
        }
        let owners = owners(&c.id).ok_or("unknown case")?;
        if c.owners.iter().map(String::as_str).collect::<Vec<_>>() != owners {
            return Err("owner coverage".into());
        }
        if c.dialects != [Dialect::Postgres, Dialect::Mysql]
            || c.scenarios.is_empty()
            || c.preconditions.schemas.is_empty()
        {
            return Err("missing prerequisites".into());
        }
        c.preconditions.tenant.validate()?;
        for schema in &c.preconditions.schemas {
            let names: BTreeSet<_> = schema.columns.iter().map(|c| c.name.as_str()).collect();
            if schema.target.is_empty()
                || schema.generation.is_empty()
                || names.len() != schema.columns.len()
                || names.is_empty()
                || schema.primary_key.is_empty()
                || schema
                    .columns
                    .iter()
                    .any(|c| !["i64", "text", "decimal", "bool"].contains(&c.sql_type.as_str()))
                || schema
                    .primary_key
                    .iter()
                    .chain(&schema.nullable_unique)
                    .any(|k| !names.contains(k.as_str()))
            {
                return Err("invalid schema".into());
            }
        }
        for s in &c.scenarios {
            if s.name.is_empty()
                || s.shape.steps.is_empty()
                || s.expected.diagnostics.is_empty()
                || s.expected.outcome.is_empty()
                || !["0", "1", "scenario_dependent"].contains(&s.expected.dispatch.as_str())
                || !["0", "scenario_dependent"].contains(&s.expected.provider_calls.as_str())
                || s.expected.tx.is_empty()
            {
                return Err("incomplete scenario".into());
            }
            s.shape.filter.validate(0)?;
            if s.shape.input_columns.iter().collect::<BTreeSet<_>>().len()
                != s.shape.input_columns.len()
            {
                return Err("duplicate input column".into());
            }
            for row in &s.shape.rows {
                if row.len() != s.shape.input_columns.len() {
                    return Err("row width".into());
                }
                for v in row {
                    v.validate()?;
                }
            }
        }
    }
    Ok(r)
}
// Independent owner register: accidental fixture owner edits cannot silently pass.
pub fn owners(id: &str) -> Option<Vec<&'static str>> {
    Some(match id {
        "QT-C01" => vec!["QT-AI-01", "QT-AI-06", "QT-AI-08"],
        "QT-C02" | "QT-C03" | "QT-C04" | "QT-C05" => vec!["QT-AI-01", "QT-AI-02"],
        "QT-C06" => vec!["QT-AI-02"],
        "QT-C07" => vec!["QT-AI-01"],
        "QT-C08" => vec!["QT-AI-02", "QT-AI-05", "QT-AI-09"],
        "QT-C09" | "QT-C11" => vec!["QT-AI-02", "QT-AI-03"],
        "QT-C10" => vec!["QT-AI-02", "QT-AI-07"],
        "QT-C12" => vec!["QT-AI-03", "QT-AI-05"],
        "QT-C13" | "QT-C16" => vec!["QT-AI-03"],
        "QT-C14" => vec!["QT-AI-03", "QT-AI-09"],
        "QT-C15" => vec!["QT-AI-04", "QT-AI-08", "QT-AI-13"],
        "QT-C17" | "QT-C18" => vec!["QT-AI-10"],
        "QT-C19" => vec!["QT-AI-05"],
        "QT-C20" => vec!["QT-AI-08"],
        "QT-C21" => vec!["QT-AI-12"],
        "QT-C22" => vec!["QT-AI-11"],
        "QT-C23" => vec!["QT-AI-13"],
        "QT-C24" => vec!["QT-AI-05", "QT-AI-06", "QT-AI-13"],
        _ => return None,
    })
}
