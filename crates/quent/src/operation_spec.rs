//! Strict source syntax, never a schema verdict or executable operation.
//!
//! Explicit sensitive access returns the retained, bounded JSON tree. It must not
//! be logged. No public output trait traverses that tree or the application context.
use quent_core::DatabaseContext;
use serde::Serialize;
use serde_json::{Map, Value};
use std::{collections::BTreeSet, fmt};
pub(crate) mod bounded;

type Result<T> = std::result::Result<T, SpecError>;
/// Closed input-free failures; no source error or path is retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecError {
    Wire,
    Budget,
    Shape,
    Unsupported,
    Range,
    Unverified,
    ReadDenied,
    TenantPredicate,
    SnapshotMismatch,
}
impl fmt::Display for SpecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_SPEC_{self:?}")
    }
}
impl std::error::Error for SpecError {}
macro_rules! redacted {
    ($($t:ty),+) => {$(impl fmt::Debug for $t {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(concat!(stringify!($t), "([REDACTED])"))
        }
    })+};
}
mod typed;
pub use typed::*;

/// Private immutable bounded source. Parsing proves syntax and native ranges only.
pub struct SpecSource {
    tree: Value,
    counts: Counts,
    // Every used symbol, and whether it occurs in a predicate. Retained privately.
    uses: Vec<(String, bool)>,
}
/// Role knowledge supplied by authenticated application code, never wire claims.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnRole {
    Ordinary,
    Tenant,
    Unknown,
}
/// Finite application assertion. Symbols must be unique canonical names, with no
/// aliases for physical columns. `Unknown` is explicit incomplete coverage.
/// Read permission here is an additional refusal gate, never a policy override.
pub struct ModelContract {
    model: String,
    columns: Vec<(String, ColumnRole, bool)>,
    snapshot: DatabaseContext,
}
/// Retains both exact source and exact application contract; still non-executable.
pub struct RoleCheckedSource {
    source: SpecSource,
    contract: ModelContract,
}
redacted!(SpecSource, ModelContract, RoleCheckedSource);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
struct Counts {
    fields: u16,
    order: u16,
    predicates: u16,
    branches: u16,
    literals: u16,
    literal_bytes: u32,
}
/// Output-only. Private fields and Serialize only; no Deserialize/restoration API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SpecSummary {
    kind: &'static str,
    version: u8,
    state: State,
    counts: Counts,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
enum State {
    SyntaxOnly,
    ApplicationRolesChecked,
}
impl SpecSummary {
    pub fn to_json(&self) -> String {
        // Fixed bounded fields, serialization cannot fail.
        serde_json::to_string(self).expect("fixed spec summary")
    }
}
/// Output-only finite diagnostic, independent kind/version from source/summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct SpecDiagnostic {
    kind: &'static str,
    version: u8,
    code: SpecError,
}
impl SpecError {
    pub fn public_view(self) -> SpecDiagnostic {
        SpecDiagnostic {
            kind: "quent_operation_spec_diagnostic",
            version: 1,
            code: self,
        }
    }
}
impl SpecDiagnostic {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed spec diagnostic")
    }
}
impl SpecSource {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        // Checks depth/nodes/arrays/decoded duplicate keys BEFORE allocating Value.
        bounded::check(bytes)?;
        let tree: Value = serde_json::from_slice(bytes).map_err(|_| SpecError::Wire)?;
        let root = object(
            &tree,
            &[
                "kind", "version", "model", "fields", "filter", "order", "limit",
            ],
            &["kind", "version", "model", "fields", "order", "limit"],
        )?;
        if string(get(root, "kind")?)? != "quent_operation_spec"
            || get(root, "version")?.as_u64() != Some(1)
        {
            return Err(SpecError::Unsupported);
        }
        symbol(get(root, "model")?)?;
        let fields = array(get(root, "fields")?, 1, 64)?;
        let mut uses = Vec::new();
        let mut unique = BTreeSet::new();
        for f in fields {
            let name = symbol(f)?;
            if !unique.insert(name) {
                return Err(SpecError::Shape);
            }
            uses.push((name.to_owned(), false));
        }
        let order = array(get(root, "order")?, 0, 16)?;
        unique.clear();
        for entry in order {
            let o = object(entry, &["field", "direction"], &["field", "direction"])?;
            let name = symbol(get(o, "field")?)?;
            if !unique.insert(name) {
                return Err(SpecError::Shape);
            }
            if !matches!(string(get(o, "direction")?)?, "asc" | "desc") {
                return Err(SpecError::Unsupported);
            }
            uses.push((name.to_owned(), false));
        }
        let limit = get(root, "limit")?.as_u64().ok_or(SpecError::Shape)?;
        if !(1..=10_000).contains(&limit) {
            return Err(SpecError::Range);
        }
        let mut counts = Counts {
            fields: fields.len() as u16,
            order: order.len() as u16,
            ..Counts::default()
        };
        if let Some(filter) = root.get("filter") {
            predicate(filter, &mut counts, &mut uses)?;
        }
        Ok(Self { tree, counts, uses })
    }
    pub fn public_view(&self) -> SpecSummary {
        SpecSummary {
            kind: "quent_operation_spec_summary",
            version: 1,
            state: State::SyntaxOnly,
            counts: self.counts,
        }
    }
    /// Explicit sensitive inspection. Not a public summary, SQL or registered IR.
    pub fn sensitive_source(&self) -> &Value {
        &self.tree
    }
    /// Consumes the bounded source, checks every occurrence including dead branches.
    /// The caller must supply current context explicitly. No ambient lookup occurs.
    pub fn check_roles(
        self,
        contract: ModelContract,
        current: &DatabaseContext,
    ) -> Result<RoleCheckedSource> {
        if !contract.snapshot.same_snapshot(current) {
            return Err(SpecError::SnapshotMismatch);
        }
        if self.tree["model"].as_str() != Some(contract.model.as_str()) {
            return Err(SpecError::Unverified);
        }
        for (name, predicate) in &self.uses {
            let (_, role, read) = contract
                .columns
                .iter()
                .find(|(n, _, _)| n == name)
                .ok_or(SpecError::Unverified)?;
            if *role == ColumnRole::Unknown {
                return Err(SpecError::Unverified);
            }
            if !read {
                return Err(SpecError::ReadDenied);
            }
            if *predicate && *role == ColumnRole::Tenant {
                return Err(SpecError::TenantPredicate);
            }
        }
        Ok(RoleCheckedSource {
            source: self,
            contract,
        })
    }
}
impl ModelContract {
    /// Caller asserts complete role knowledge for each registered symbol, unique
    /// physical column mapping (no aliases), accurate read restrictions and exact
    /// target/snapshot. Construction does not authenticate these assertions.
    /// Missing symbols/Unknown roles refuse. No manifest/fingerprint import exists.
    pub fn application_asserted(
        model: &str,
        columns: Vec<(String, ColumnRole, bool)>,
        current: &DatabaseContext,
    ) -> Result<Self> {
        symbol_str(model)?;
        if columns.is_empty() || columns.len() > 256 {
            return Err(SpecError::Budget);
        }
        let mut seen = BTreeSet::new();
        for (name, _, _) in &columns {
            symbol_str(name)?;
            if !seen.insert(name) {
                return Err(SpecError::Shape);
            }
        }
        Ok(Self {
            model: model.to_owned(),
            columns,
            snapshot: current.clone(),
        })
    }
}
impl RoleCheckedSource {
    pub fn public_view(&self) -> SpecSummary {
        SpecSummary {
            state: State::ApplicationRolesChecked,
            ..self.source.public_view()
        }
    }
    pub fn sensitive_source(&self) -> &Value {
        self.source.sensitive_source()
    }
    /// Exact retained context comparison, not freshness or physical authentication.
    pub fn matches_snapshot(&self, current: &DatabaseContext) -> bool {
        self.contract.snapshot.same_snapshot(current)
    }
}
fn get<'a>(m: &'a Map<String, Value>, k: &str) -> Result<&'a Value> {
    m.get(k).ok_or(SpecError::Shape)
}
fn object<'a>(v: &'a Value, allowed: &[&str], required: &[&str]) -> Result<&'a Map<String, Value>> {
    let m = v.as_object().ok_or(SpecError::Shape)?;
    if m.keys().any(|k| !allowed.contains(&k.as_str()))
        || required.iter().any(|k| !m.contains_key(*k))
    {
        return Err(SpecError::Shape);
    }
    Ok(m)
}
fn string(v: &Value) -> Result<&str> {
    v.as_str().ok_or(SpecError::Shape)
}
fn symbol(v: &Value) -> Result<&str> {
    let s = string(v)?;
    symbol_str(s)?;
    Ok(s)
}
fn symbol_str(s: &str) -> Result<()> {
    if s.is_empty()
        || s.len() > 63
        || !s
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err(SpecError::Shape);
    }
    Ok(())
}
fn array(v: &Value, min: usize, max: usize) -> Result<&[Value]> {
    let a = v.as_array().ok_or(SpecError::Shape)?;
    if a.len() < min {
        return Err(SpecError::Shape);
    }
    if a.len() > max {
        return Err(SpecError::Budget);
    }
    Ok(a)
}
fn predicate(v: &Value, counts: &mut Counts, uses: &mut Vec<(String, bool)>) -> Result<()> {
    counts.predicates += 1;
    if counts.predicates > 256 {
        return Err(SpecError::Budget);
    }
    let base = v.as_object().ok_or(SpecError::Shape)?;
    let op = string(get(base, "op")?)?;
    match op {
        "and" | "or" => {
            let m = object(v, &["op", "args"], &["op", "args"])?;
            let a = array(get(m, "args")?, 1, 32)?;
            counts.branches += a.len() as u16;
            if counts.branches > 256 {
                return Err(SpecError::Budget);
            }
            for child in a {
                predicate(child, counts, uses)?;
            }
        }
        "not" => {
            let m = object(v, &["op", "arg"], &["op", "arg"])?;
            predicate(get(m, "arg")?, counts, uses)?;
        }
        "eq" | "ne" | "lt" | "le" | "gt" | "ge" | "in" | "is_null" => {
            let keys: &[&str] = match op {
                "in" => &["op", "field", "values"],
                "is_null" => &["op", "field"],
                _ => &["op", "field", "value"],
            };
            let m = object(v, keys, keys)?;
            uses.push((symbol(get(m, "field")?)?.to_owned(), true));
            match op {
                "in" => {
                    for item in array(get(m, "values")?, 1, 64)? {
                        literal(item, counts)?;
                    }
                }
                "is_null" => (),
                _ => literal(get(m, "value")?, counts)?,
            }
        }
        _ => return Err(SpecError::Unsupported),
    }
    Ok(())
}
fn literal(v: &Value, counts: &mut Counts) -> Result<()> {
    let m = object(v, &["type", "value"], &["type", "value"])?;
    let tag = string(get(m, "type")?)?;
    if !matches!(
        tag,
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "bool" | "text"
    ) {
        return Err(SpecError::Unsupported);
    }
    let value = get(m, "value")?;
    counts.literals += 1;
    if counts.literals > 512 {
        return Err(SpecError::Budget);
    }
    // Native JSON integer tokens are checked without float or string coercion.
    if !value.is_null() {
        let valid = match tag {
            "i8" => value.as_i64().is_some_and(|n| i8::try_from(n).is_ok()),
            "i16" => value.as_i64().is_some_and(|n| i16::try_from(n).is_ok()),
            "i32" => value.as_i64().is_some_and(|n| i32::try_from(n).is_ok()),
            "i64" => value.as_i64().is_some(),
            "u8" => value.as_u64().is_some_and(|n| u8::try_from(n).is_ok()),
            "u16" => value.as_u64().is_some_and(|n| u16::try_from(n).is_ok()),
            "u32" => value.as_u64().is_some_and(|n| u32::try_from(n).is_ok()),
            "u64" => value.as_u64().is_some(),
            "bool" => value.is_boolean(),
            "text" => value.is_string(),
            _ => false,
        };
        if !valid {
            return Err(SpecError::Range);
        }
    }
    counts.literal_bytes += match value {
        Value::String(s) => s.len() as u32,
        Value::Null => 0,
        Value::Bool(_) => 1,
        _ => 8,
    };
    if counts.literal_bytes > 16_384 {
        return Err(SpecError::Budget);
    }
    Ok(())
}
