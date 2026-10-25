//! Pure bounded generator. Source declarations are untrusted code data, not facts.
//! Generated repositories require an independently supplied application mapping
//! and current context. See specs/typed-codegen-v1.md for the finite contract.
use crate::manifest::{ArtifactKind, ManifestSource, SourceCoverage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, fmt};
mod emit;
mod patch;
mod regenerate;
pub use patch::{PatchDiagnostic, PatchError, PatchSummary};
pub use regenerate::{Regeneration, RegenerationAction, RegenerationState, compare};
/// Implementation dependency for generated source, pinned by this workspace.
/// This reexport supplies JSON data only, never an executor or authority builder.
#[doc(hidden)]
pub use serde_json as json;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CodegenError {
    Wire,
    Version,
    Budget,
    Unsupported,
    Naming,
    Coverage,
    Mapping,
}
impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_CODEGEN_{self:?}")
    }
}
impl std::error::Error for CodegenError {}
#[derive(Debug, Serialize)]
pub struct Diagnostic {
    kind: &'static str,
    version: u8,
    state: &'static str,
    code: CodegenError,
}
impl CodegenError {
    pub fn public_view(self) -> Diagnostic {
        Diagnostic {
            kind: "quent_codegen_diagnostic",
            version: 1,
            state: "refused",
            code: self,
        }
    }
}
impl Diagnostic {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed diagnostic")
    }
}
#[derive(Debug, Serialize)]
pub struct Summary {
    kind: &'static str,
    version: u8,
    state: &'static str,
    models: usize,
    projections: usize,
    code_bytes: usize,
}
impl Summary {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed summary")
    }
}
/// Local code artifact only. No Display/serde, filesystem I/O or execution API.
pub struct CodeArtifact {
    revision: u32,
    schema: Vec<u8>,
    policy: Vec<u8>,
    config: String,
    source: String,
    models: usize,
    projections: usize,
}
impl fmt::Debug for CodeArtifact {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("CodeArtifact([REDACTED])")
    }
}
impl CodeArtifact {
    pub fn sensitive_rust_source(&self) -> &str {
        &self.source
    }
    pub fn public_view(&self) -> Summary {
        Summary {
            kind: "quent_codegen_summary",
            version: 1,
            state: "unverified_code",
            models: self.models,
            projections: self.projections,
            code_bytes: self.source.len(),
        }
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Config {
    kind: ConfigKind,
    version: u32,
    generator: String,
    models: Vec<Model>,
}
#[derive(Deserialize, Serialize)]
enum ConfigKind {
    #[serde(rename = "quent_codegen_config")]
    Config,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Model {
    table: Vec<String>,
    columns: Vec<Column>,
    projections: Vec<Projection>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Column {
    name: String,
    role: Role,
    read: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum Role {
    Ordinary,
    Tenant,
    Unknown,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Projection {
    name: String,
    fields: Vec<String>,
}

/// No case folding, aliasing, normalization or lossy identifier substitution.
fn name(s: &str) -> bool {
    const RESERVED: &[&str] = &[
        "as",
        "async",
        "await",
        "break",
        "const",
        "continue",
        "crate",
        "dyn",
        "else",
        "enum",
        "extern",
        "false",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "ref",
        "return",
        "self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
        "abstract",
        "become",
        "box",
        "do",
        "final",
        "macro",
        "override",
        "priv",
        "typeof",
        "unsized",
        "virtual",
        "yield",
        "try",
        "gen",
        "union",
        "raw",
        "safe",
        "key",
        "model",
        "column",
        "predicate",
        "order",
        "projection",
        "repository",
        "read",
        "result",
        "bind",
        "select",
        "inspect",
        "public_view",
        "sensitive_rows",
        "new",
        "eq",
        "is_null",
        "asc",
        "desc",
        "json",
        "core",
        "std",
        "quent",
        "quent_core",
    ];
    !s.is_empty()
        && s.len() <= 63
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        && !s.ends_with('_')
        && !s.contains("__")
        && !RESERVED.contains(&s)
}
fn pascal(s: &str) -> String {
    s.split('_')
        .map(|part| {
            let mut b = part.as_bytes().to_vec();
            b[0] = b[0].to_ascii_uppercase();
            String::from_utf8(b).expect("ASCII")
        })
        .collect()
}
fn native(c: &Value) -> Result<&str, CodegenError> {
    let n = c["native"].as_str().ok_or(CodegenError::Wire)?;
    if matches!(
        n,
        "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "bool" | "text"
    ) {
        Ok(n)
    } else {
        Err(CodegenError::Unsupported)
    }
}
fn rust_ty(c: &Value) -> String {
    let n = native(c).expect("validated native");
    let t = if n == "text" { "String" } else { n };
    if c["nullability"] == "nullable" {
        format!("Option<{t}>")
    } else {
        t.into()
    }
}
fn variant(c: &Value) -> String {
    let n = native(c).expect("validated native");
    let mut s = n.to_owned();
    s[..1].make_ascii_uppercase();
    s
}
/// Manifest source v1 + explicit matched config/generator revision 1 or 2.
/// Revision 1 output stays byte-identical; revision 2 additionally emits patches.
/// The only inputs are the supplied bytes; no environment, clock, provider or I/O.
pub fn generate(manifest: &[u8], config: &[u8]) -> Result<CodeArtifact, CodegenError> {
    use CodegenError as E;
    let source = ManifestSource::parse(manifest).map_err(|_| E::Wire)?;
    crate::manifest::bounded::check(config).map_err(|_| E::Wire)?;
    let cfg: Config = serde_json::from_slice(config).map_err(|_| E::Wire)?;
    if !matches!(
        (cfg.version, cfg.generator.as_str()),
        (1, "quent-codegen/1") | (2, "quent-codegen/2")
    ) {
        return Err(E::Version);
    }
    if cfg.version == 2
        && cfg.models.iter().any(|m| {
            m.columns.iter().any(|c| {
                matches!(
                    c.name.as_str(),
                    "patch"
                        | "payload"
                        | "compiled_patch"
                        | "change"
                        | "nullable_change"
                        | "compile_patch"
                        | "matches_current"
                )
            })
        })
    {
        return Err(E::Naming);
    }
    let summaries = source.public_view();
    for summary in &summaries.artifacts()[..2] {
        match summary.coverage() {
            SourceCoverage::Complete => {}
            SourceCoverage::Unsupported => return Err(E::Unsupported),
            _ => return Err(E::Coverage),
        }
    }
    let canonical = source.sensitive_source().canonical_source();
    let tree: Value = serde_json::from_slice(&canonical).map_err(|_| E::Wire)?;
    if !tree["generated_code"].is_null() || !tree["live_observation"].is_null() {
        return Err(E::Unsupported);
    }
    let schema = &tree["schema"];
    let policy = &tree["policy"];
    if schema["target"] != policy["target"] {
        return Err(E::Mapping);
    }
    let tables = schema["tables"].as_array().ok_or(E::Wire)?;
    if cfg.models.is_empty() || cfg.models.len() > 8 || cfg.models.len() != tables.len() {
        return Err(E::Budget);
    }
    let rules = policy["rules"].as_array().ok_or(E::Wire)?;
    // Reject ambiguous or dangling policy declarations, including duplicate kinds.
    let mut rule_names = BTreeSet::new();
    for r in rules {
        if !rule_names.insert((r["table"].to_string(), r["column"].to_string()))
            || !tables.iter().any(|t| {
                t["path"] == r["table"]
                    && t["columns"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|c| c["name"] == r["column"])
            })
        {
            return Err(E::Mapping);
        }
    }
    let mut modules = BTreeSet::new();
    let mut items = 0;
    let mut total_columns = 0;
    for (m, t) in cfg.models.iter().zip(tables) {
        if m.table.len() != 2 || m.table.iter().any(|s| !name(s)) || !modules.insert(&m.table[1]) {
            return Err(E::Naming);
        }
        if serde_json::to_value(&m.table).unwrap() != t["path"] {
            return Err(E::Mapping);
        }
        let cs = t["columns"].as_array().unwrap();
        total_columns += cs.len();
        if cs.is_empty()
            || cs.len() > 32
            || total_columns > 128
            || m.columns.len() != cs.len()
            || m.projections.is_empty()
            || m.projections.len() > 8
        {
            return Err(E::Budget);
        }
        let keys = t["keys"].as_array().unwrap();
        if keys.iter().filter(|k| k["kind"] == "primary").count() != 1 || keys.len() > 8 {
            return Err(E::Unsupported);
        }
        for (c, decl) in cs.iter().zip(&m.columns) {
            native(c)?;
            if !name(&decl.name) {
                return Err(E::Naming);
            }
            if c["name"] != decl.name || decl.role == Role::Unknown {
                return Err(E::Mapping);
            }
            let tenant = rules.iter().any(|r| {
                r["table"] == t["path"] && r["column"] == decl.name && r["kind"] == "tenant"
            });
            if tenant != (decl.role == Role::Tenant) {
                return Err(E::Mapping);
            }
        }
        let mut projections = BTreeSet::new();
        for p in &m.projections {
            if !name(&p.name)
                || m.columns.iter().any(|c| c.name == p.name)
                || !projections.insert(pascal(&p.name))
            {
                return Err(E::Naming);
            }
            let mut fields = BTreeSet::new();
            if p.fields.is_empty() || p.fields.len() > 32 {
                return Err(E::Budget);
            }
            for f in &p.fields {
                if !fields.insert(f) || !m.columns.iter().any(|c| c.name == *f && c.read) {
                    return Err(E::Mapping);
                }
            }
            items += p.fields.len() + 1;
        }
        items += cs.len() * 8 + keys.len();
        if items > 2048 {
            return Err(E::Budget);
        }
    }
    if cfg.version == 2 {
        for (m, t) in cfg.models.iter().zip(tables) {
            let mut methods = BTreeSet::new();
            for (d, c) in m.columns.iter().zip(t["columns"].as_array().unwrap()) {
                if !methods.insert(format!("set_{}", d.name))
                    || (c["nullability"] == "nullable"
                        && !methods.insert(format!("set_null_{}", d.name)))
                {
                    return Err(E::Naming);
                }
            }
        }
    }
    let cfg_canonical = serde_json::to_string(&cfg).expect("closed config");
    let schema_bytes = source
        .sensitive_source()
        .canonical_artifact(ArtifactKind::Schema)
        .unwrap();
    let policy_bytes = source
        .sensitive_source()
        .canonical_artifact(ArtifactKind::Policy)
        .unwrap();
    let output = emit::emit(
        &cfg,
        schema,
        policy,
        &schema_bytes,
        &policy_bytes,
        &cfg_canonical,
    )?;
    if output.len() > 524_288 {
        return Err(E::Budget);
    }
    Ok(CodeArtifact {
        revision: cfg.version,
        schema: schema_bytes,
        policy: policy_bytes,
        config: cfg_canonical,
        source: output,
        models: cfg.models.len(),
        projections: cfg.models.iter().map(|m| m.projections.len()).sum(),
    })
}
