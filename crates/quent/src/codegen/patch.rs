use super::*;
use std::fmt::Write;

/// Fixed patch failures; no source, values, or driver errors are retained.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PatchError {
    Empty,
    Duplicate,
    Native,
    Budget,
    Binding,
    Policy,
    Gate,
}
impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_PATCH_{self:?}")
    }
}
impl std::error::Error for PatchError {}
#[derive(Debug, Serialize)]
pub struct PatchDiagnostic {
    kind: &'static str,
    version: u8,
    state: &'static str,
    code: PatchError,
}
impl PatchError {
    pub fn public_view(self) -> PatchDiagnostic {
        PatchDiagnostic {
            kind: "quent_patch_diagnostic",
            version: 1,
            state: "refused",
            code: self,
        }
    }
}
impl PatchDiagnostic {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed diagnostic")
    }
}
/// Output only. Counts confer no permission or execution capability.
#[derive(Debug, Serialize)]
pub struct PatchSummary {
    kind: &'static str,
    version: u8,
    state: &'static str,
    assignments: usize,
}
impl PatchSummary {
    #[doc(hidden)]
    pub fn inspection_only(assignments: usize) -> Self {
        Self {
            kind: "quent_patch_summary",
            version: 1,
            state: "inspection_only",
            assignments,
        }
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed summary")
    }
}

pub(super) fn emit(
    out: &mut impl Write,
    m: &Model,
    t: &Value,
    policy: &Value,
) -> Result<(), CodegenError> {
    macro_rules! w { ($($t:tt)*) => { writeln!(out, $($t)*).map_err(|_| CodegenError::Budget)? }; }
    let cs = t["columns"].as_array().unwrap();
    let primary = t["keys"]
        .as_array()
        .unwrap()
        .iter()
        .find(|k| k["kind"] == "primary")
        .unwrap()["columns"]
        .as_array()
        .unwrap();
    // Config/read is a reduction only. Bind independently checks application
    // mapping and the complete policy; common preparation checks protected writes.
    let writable: Vec<_> = cs
        .iter()
        .zip(&m.columns)
        .filter(|(c, d)| {
            d.read
                && d.role == Role::Ordinary
                && !primary.contains(&c["name"])
                && !policy["rules"].as_array().unwrap().iter().any(|r| {
                    r["table"] == t["path"]
                        && r["column"] == c["name"]
                        && matches!(
                            r["kind"].as_str(),
                            Some("tenant" | "readonly" | "generated" | "pii")
                        )
                })
        })
        .collect();
    w!("use quent::codegen::{{PatchError,PatchSummary}};");
    w!(
        "enum Change<T> {{ Unchanged, Set(T) }} enum NullableChange<T> {{ Unchanged, SetNull, Set(T) }}"
    );
    w!("pub struct Patch {{ payload:usize,");
    for (c, d) in &writable {
        let ty = native(c)?;
        let ty = if ty == "text" { "String" } else { ty };
        let change = if c["nullability"] == "nullable" {
            "NullableChange"
        } else {
            "Change"
        };
        w!("{}:{change}<{ty}>,", d.name);
    }
    w!(
        "}} redacted!(Patch); impl Default for Patch {{fn default()->Self{{Self::new()}}}} impl Patch {{pub fn new()->Self{{Self{{payload:0,"
    );
    for (c, d) in &writable {
        let change = if c["nullability"] == "nullable" {
            "NullableChange"
        } else {
            "Change"
        };
        w!("{}:{change}::Unchanged,", d.name);
    }
    w!("}}}}");
    for (c, d) in &writable {
        let n = &d.name;
        let ty = native(c)?;
        let ty = if ty == "text" { "String" } else { ty };
        let change = if c["nullability"] == "nullable" {
            "NullableChange"
        } else {
            "Change"
        };
        w!(
            "pub fn set_{n}(mut self,v:{ty})->Result<Self,PatchError>{{if !matches!(self.{n},{change}::Unchanged){{return Err(PatchError::Duplicate)}}"
        );
        if ty == "String" {
            w!("if v.len()>4096 || v.contains('\\0'){{return Err(PatchError::Native)}}");
        }
        let size = if ty == "String" { "v.len()" } else { "8" };
        w!(
            "self.payload+={size};if self.payload>65536{{return Err(PatchError::Budget)}}self.{n}={change}::Set(v);Ok(self)}}"
        );
        if change == "NullableChange" {
            w!(
                "pub fn set_null_{n}(mut self)->Result<Self,PatchError>{{if !matches!(self.{n},NullableChange::Unchanged){{return Err(PatchError::Duplicate)}}self.payload+=8;if self.payload>65536{{return Err(PatchError::Budget)}}self.{n}=NullableChange::SetNull;Ok(self)}}"
            );
        }
    }
    w!("}}");
    w!(
        "pub struct CompiledPatch {{prepared:core::PolicyPreparation,mapping:TypedModelContract}} redacted!(CompiledPatch); impl CompiledPatch {{pub fn inspect(&self)->core::Inspection<'_>{{self.prepared.inspect()}} pub fn public_view(&self)->PatchSummary{{PatchSummary::inspection_only(self.inspect().sensitive_source().metadata().ordered_write_data().len())}} pub fn matches_current(&self,current:&core::DatabaseContext,mapping:&TypedModelContract)->bool{{self.mapping==*mapping && self.prepared.sensitive_source().context()==current}} }}"
    );
    w!(
        "impl Repository<'_> {{pub fn compile_patch(&self,key:Key,patch:Patch)->Result<CompiledPatch,PatchError>{{let id=|s:&str|core::Ident::new(s).map_err(|_|PatchError::Binding);let table=core::Table::new(vec![id({:?})?,id({:?})?],None).map_err(|_|PatchError::Binding)?;let _=&patch;#[allow(unused_mut)]let mut data=Vec::new();",
        m.table[0],
        m.table[1]
    );
    for (c, d) in &writable {
        let n = &d.name;
        let v = variant(c);
        let nullable = c["nullability"] == "nullable";
        let change = if nullable { "NullableChange" } else { "Change" };
        w!(
            "match patch.{n}{{{change}::Unchanged=>(),{change}::Set(v)=>data.push((table.column(id({n:?})?,core::SqlType::{v},{nullable}),core::Input::Value(core::Value::{v}(v)))),"
        );
        if nullable {
            w!(
                "NullableChange::SetNull=>data.push((table.column(id({n:?})?,core::SqlType::{v},true),core::Input::Value(core::Value::Null(core::SqlType::{v})))),"
            );
        }
        w!("}}");
    }
    w!(
        "if data.is_empty(){{return Err(PatchError::Empty)}} let payload:usize=data.iter().map(|(_,v)|match v{{core::Input::Value(core::Value::Text(s))=>s.len(),_=>8}}).sum();if payload>65536{{return Err(PatchError::Budget)}} let predicates=vec!["
    );
    for (i, n) in primary.iter().enumerate() {
        let c = cs.iter().find(|c| c["name"] == *n).unwrap();
        let v = variant(c);
        w!(
            "core::Predicate::Compare(table.column(id({:?})?,core::SqlType::{v},false),core::Compare::Eq,core::Value::{v}(key.{i})),",
            n.as_str().unwrap()
        );
    }
    w!(
        "];let operation=core::Draft::update(table,data).filter(core::Predicate::And(predicates)).freeze();let prepared=self.current.prepare().prepare_operation(&operation).map_err(|_|PatchError::Policy)?;let inspect=prepared.inspect();let final_payload:usize=inspect.sensitive_source().args().iter().map(|v|match v{{core::Value::Text(s)=>s.len(),_=>8}}).sum();if final_payload>65536{{return Err(PatchError::Budget)}}\nif inspect.risk()!=core::Risk::Low || inspect.completeness()!=core::Completeness::Complete{{return Err(PatchError::Gate)}}Ok(CompiledPatch{{prepared,mapping:self.mapping.clone()}})}}}}"
    );
    Ok(())
}
