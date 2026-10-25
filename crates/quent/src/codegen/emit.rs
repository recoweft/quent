use super::*;
use std::fmt::Write;
struct Output(String);
impl Write for Output {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.0.len().saturating_add(s.len()) > 524_288 {
            return Err(fmt::Error);
        }
        self.0.push_str(s);
        Ok(())
    }
}
// At most 16 primary components, preserving order without exhausting the public
// eight-edge composition budget on a left-deep expression.
fn key_predicate(parts: &[String]) -> String {
    if parts.len() == 1 {
        return parts[0].clone();
    }
    let mid = parts.len() / 2;
    format!(
        "{}.and({})?",
        key_predicate(&parts[..mid]),
        key_predicate(&parts[mid..])
    )
}
pub(super) fn emit(
    cfg: &Config,
    schema: &Value,
    policy: &Value,
    schema_bytes: &[u8],
    policy_bytes: &[u8],
    cfg_bytes: &str,
) -> Result<String, CodegenError> {
    let mut out = Output(String::new());
    macro_rules! w { ($($t:tt)*) => { writeln!(out, $($t)*).map_err(|_| CodegenError::Budget)? }; }
    w!("// kind=quent_rust_code version=1 generator=quent-codegen/1; local sensitive data.");
    w!("// Exact ordered canonical domains, never fingerprints or authority.");
    w!(
        "const _: &str = {:?};",
        std::str::from_utf8(schema_bytes).unwrap()
    );
    w!(
        "const _: &str = {:?};",
        std::str::from_utf8(policy_bytes).unwrap()
    );
    w!("const _: &str = {cfg_bytes:?};");
    for (m, t) in cfg.models.iter().zip(schema["tables"].as_array().unwrap()) {
        let module = &m.table[1];
        let cs = t["columns"].as_array().unwrap();
        let keys = t["keys"].as_array().unwrap();
        let primary = keys.iter().find(|k| k["kind"] == "primary").unwrap()["columns"]
            .as_array()
            .unwrap();
        let find = |n: &Value| cs.iter().position(|c| c["name"] == *n).unwrap();
        w!("pub mod {module} {{");
        w!(
            "use quent::operation_spec::{{TypedModelContract,TypedColumn,ModelCoverage,ColumnType,ColumnRole,SpecSource,CompiledRead,TypedError}};"
        );
        w!(
            "use quent::codegen::json::{{self,json}}; use quent_core as core; use std::marker::PhantomData;"
        );
        w!(
            "macro_rules! redacted {{ ($t:ty) => {{ impl std::fmt::Debug for $t {{ fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {{ f.write_str(\"Generated([REDACTED])\") }} }} }}; }}"
        );
        w!("pub struct Model; redacted!(Model);");
        let key_types = primary
            .iter()
            .map(|n| rust_ty(&cs[find(n)]))
            .collect::<Vec<_>>();
        w!("pub struct Key({}); redacted!(Key);", key_types.join(","));
        w!(
            "impl Key {{ pub fn new({}) -> Self {{ Self({}) }} pub fn sensitive_components(&self) -> ({}) {{ ({}) }} }}",
            key_types
                .iter()
                .enumerate()
                .map(|(i, t)| format!("v{i}:{t}"))
                .collect::<Vec<_>>()
                .join(","),
            (0..key_types.len())
                .map(|i| format!("v{i}"))
                .collect::<Vec<_>>()
                .join(","),
            key_types
                .iter()
                .map(|t| format!("&{t},"))
                .collect::<String>(),
            (0..key_types.len())
                .map(|i| format!("&self.{i},"))
                .collect::<String>()
        );
        w!(
            "pub struct Column<T> {{ name: &'static str, tag: &'static str, marker: PhantomData<T> }} impl<T> std::fmt::Debug for Column<T> {{ fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {{ f.write_str(\"Generated([REDACTED])\") }} }}"
        );
        w!(
            "pub struct Predicate {{ wire: json::Value, nodes: u16, depth: u8 }} redacted!(Predicate);"
        );
        w!(
            "impl Predicate {{ fn leaf(wire:json::Value)->Self {{ Self {{wire,nodes:1,depth:0}} }} pub fn and(self,other:Self)->Result<Self,TypedError> {{self.join(other,\"and\")}} pub fn or(self,other:Self)->Result<Self,TypedError> {{self.join(other,\"or\")}} fn join(self,other:Self,op:&str)->Result<Self,TypedError> {{ let nodes=self.nodes+other.nodes+1;let depth=self.depth.max(other.depth)+1;if nodes>128 || depth>8 {{return Err(TypedError::NativeRefused)}} Ok(Self{{wire:json!({{\"op\":op,\"args\":[self.wire,other.wire]}}),nodes,depth}}) }} }}"
        );
        w!(
            "pub struct Order(json::Value); redacted!(Order); impl<T> Column<T> {{ pub fn asc(self)->Order {{let _=self.tag; Order(json!({{\"field\":self.name,\"direction\":\"asc\"}}))}} pub fn desc(self)->Order {{Order(json!({{\"field\":self.name,\"direction\":\"desc\"}}))}} }}"
        );
        let mut types = BTreeSet::new();
        for (c, decl) in cs.iter().zip(&m.columns) {
            if !decl.read || decl.role == Role::Tenant {
                continue;
            }
            let n = &decl.name;
            let ty = rust_ty(c);
            let tag = native(c)?;
            w!(
                "impl Model {{ pub fn {n}()->Column<{ty}> {{Column{{name:{n:?},tag:{tag:?},marker:PhantomData}}}} }}"
            );
            if types.insert(ty.clone()) {
                w!("impl Column<{ty}> {{ pub fn eq(self,v:{ty})->Result<Predicate,TypedError> {{");
                let nullable = c["nullability"] == "nullable";
                if nullable {
                    w!("if v.is_none() {{return Err(TypedError::NullComparison)}}");
                }
                if tag == "text" {
                    let v = if nullable {
                        "v.as_ref().unwrap()"
                    } else {
                        "&v"
                    };
                    w!(
                        "let text={v}; if text.len()>4096 || text.contains('\\0') {{return Err(TypedError::NativeRefused)}}"
                    );
                }
                w!(
                    "Ok(Predicate::leaf(json!({{\"op\":\"eq\",\"field\":self.name,\"value\":{{\"type\":self.tag,\"value\":v}}}}))) }}"
                );
                if nullable {
                    w!(
                        "pub fn is_null(&self)->Predicate {{ Predicate::leaf(json!({{\"op\":\"is_null\",\"field\":self.name}})) }}"
                    );
                }
                w!("}}");
            }
        }
        if primary.iter().all(|n| {
            let d = &m.columns[find(n)];
            d.role == Role::Ordinary && d.read
        }) {
            w!("impl Key {{ pub fn predicate(self)->Result<Predicate,TypedError> {{");
            for (i, n) in primary.iter().enumerate() {
                w!("let p{i}=Model::{}().eq(self.{i})?;", n.as_str().unwrap());
            }
            w!(
                "let predicate={}; Ok(predicate) }} }}",
                key_predicate(
                    &(0..primary.len())
                        .map(|i| format!("p{i}"))
                        .collect::<Vec<_>>()
                )
            );
        }
        w!(
            "pub struct Projection<R> {{ fields: &'static [&'static str], decode: fn(&[core::Value])->Result<R,()>, marker: PhantomData<R> }} impl<R> std::fmt::Debug for Projection<R> {{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{{f.write_str(\"Generated([REDACTED])\")}}}}"
        );
        w!(
            "pub struct Read<R> {{ compiled:CompiledRead, decode:fn(&[core::Value])->Result<R,()> }} impl<R> std::fmt::Debug for Read<R> {{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{{f.write_str(\"Generated([REDACTED])\")}}}}"
        );
        w!(
            "impl<R> Read<R> {{ pub fn inspect(&self)->core::Inspection<'_> {{let _=self.decode; self.compiled.inspect()}} pub fn public_view(&self)->core::PlanView {{self.compiled.public_view()}} }}"
        );
        w!(
            "pub struct Rows<R>(Vec<R>); impl<R> Rows<R> {{pub fn sensitive_rows(&self)->&[R] {{&self.0}}}} impl<R> std::fmt::Debug for Rows<R> {{fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{{f.write_str(\"Generated([REDACTED])\")}}}}"
        );
        w!(
            "pub struct Repository<'a> {{ current: &'a core::DatabaseContext, mapping: &'a TypedModelContract }} redacted!(Repository<'_>);"
        );
        w!(
            "impl<'a> Repository<'a> {{ pub fn bind(current:&'a core::DatabaseContext,mapping:&'a TypedModelContract)->Result<Self,TypedError> {{"
        );
        let target = &schema["target"];
        let dialect = if target["dialect"] == "postgres" {
            "Postgres"
        } else {
            "Mysql"
        };
        w!(
            "let sensitive=current.sensitive_source(); let target=sensitive.target(); if target.0!={:?} || target.1!={:?} || target.2.sensitive_text()!={:?} || target.3!=core::Dialect::{dialect} {{return Err(TypedError::SnapshotMismatch)}}",
            target["database"].as_str().unwrap(),
            target["address"].as_str().unwrap(),
            target["catalog"].as_str().unwrap()
        );
        w!(
            "let id=|s:&str| core::Ident::new(s).map_err(|_|TypedError::Mapping); let table=core::Table::new(vec![id({:?})?,id({:?})?],None).map_err(|_|TypedError::Mapping)?;",
            m.table[0],
            m.table[1]
        );
        w!("let columns=vec![");
        for c in cs {
            w!(
                "table.column(id({:?})?,core::SqlType::{},{}),",
                c["name"].as_str().unwrap(),
                variant(c),
                c["nullability"] == "nullable"
            );
        }
        w!(
            "]; let expected=TypedModelContract::application_asserted({module:?},table.clone(),vec!["
        );
        for (i, d) in m.columns.iter().enumerate() {
            let role = if d.role == Role::Tenant {
                "Tenant"
            } else {
                "Ordinary"
            };
            w!(
                "TypedColumn::application_supplied({:?},columns[{i}].sensitive_name().clone(),ColumnType::Native(columns[{i}].sql_type()),columns[{i}].nullable(),ColumnRole::{role},{})?,",
                d.name,
                d.read
            );
        }
        w!(
            "],ModelCoverage::Complete,current)?; if expected!=*mapping {{return Err(TypedError::Mapping)}}"
        );
        w!(
            "let facts=core::ScopeFacts::application_asserted(table.clone(),core::Dialect::{dialect},columns.clone(),vec!["
        );
        for k in keys {
            let kind = if k["kind"] == "primary" {
                "Primary"
            } else {
                "Unique"
            };
            w!(
                "core::Key::{kind}(vec![{}]),",
                k["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|n| format!("columns[{}].clone()", find(n)))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }
        w!(
            "],core::TableContext::OrdinaryBaseTableWithoutEffects).map_err(|_|TypedError::Mapping)?; if !current.sensitive_source().schema().is_some_and(|(_,_,facts_list)|facts_list.contains(&facts)) {{return Err(TypedError::Mapping)}}"
        );
        w!("let policies:Vec<core::PolicyDeclaration>=vec![");
        for r in policy["rules"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["table"] == t["path"])
        {
            let kind = match r["kind"].as_str().unwrap() {
                "tenant" => "Tenant",
                "soft_delete" => "SoftDelete",
                "pii" => "Pii",
                "required_filter" => "RequiredFilter",
                "readonly" => "Readonly",
                "generated" => "Generated",
                _ => return Err(CodegenError::Unsupported),
            };
            w!(
                "core::PolicyDeclaration::{kind}(columns[{}].clone()),",
                find(&r["column"])
            );
        }
        w!(
            "]; let actual:Vec<_>=current.sensitive_source().declarations().iter().filter(|p|{{let c=match p {{ core::PolicyDeclaration::Tenant(c)|core::PolicyDeclaration::SoftDelete(c)|core::PolicyDeclaration::Pii(c)|core::PolicyDeclaration::RequiredFilter(c)|core::PolicyDeclaration::Readonly(c)|core::PolicyDeclaration::Generated(c)=>c }};c.sensitive_table()==&table}}).cloned().collect(); if policies!=actual {{return Err(TypedError::Role)}} Ok(Self{{current,mapping}}) }}"
        );
        w!(
            "pub fn select<R>(&self,projection:Projection<R>,filter:Option<Predicate>,order:Vec<Order>,limit:u16)->Result<Read<R>,TypedError> {{ if order.len()>32 || limit==0 || limit>256 {{return Err(TypedError::NativeRefused)}} let mut wire=json!({{\"kind\":\"quent_operation_spec\",\"version\":1,\"model\":{module:?},\"fields\":projection.fields,\"order\":order.into_iter().map(|o|o.0).collect::<Vec<_>>(),\"limit\":limit}});if let Some(p)=filter{{wire[\"filter\"]=p.wire;}} let bytes=json::to_vec(&wire).map_err(|_|TypedError::NativeRefused)?;let compiled=SpecSource::parse(&bytes).map_err(|_|TypedError::NativeRefused)?.compile(self.mapping,self.current)?;Ok(Read{{compiled,decode:projection.decode}}) }} }}"
        );
        for p in &m.projections {
            let row = format!("{}Row", pascal(&p.name));
            w!("pub struct {row} {{");
            for f in &p.fields {
                w!(
                    "{f}: {},",
                    rust_ty(&cs[cs.iter().position(|c| c["name"] == *f).unwrap()])
                );
            }
            w!("}} redacted!({row}); impl {row} {{");
            for f in &p.fields {
                w!(
                    "pub fn sensitive_{f}(&self)->&{} {{&self.{f}}}",
                    rust_ty(&cs[cs.iter().position(|c| c["name"] == *f).unwrap()])
                );
            }
            w!(
                "}} impl Model {{pub fn {}()->Projection<{row}> {{Projection{{fields:&{:?},decode:decode_{},marker:PhantomData}}}} }}",
                p.name,
                p.fields,
                p.name
            );
            w!(
                "fn decode_{}(values:&[core::Value])->Result<{row},()> {{ if values.len()!={} {{return Err(())}} Ok({row}{{",
                p.name,
                p.fields.len()
            );
            for (i, f) in p.fields.iter().enumerate() {
                let c = &cs[cs.iter().position(|c| c["name"] == *f).unwrap()];
                let v = variant(c);
                let payload = if v == "Text" { "v.clone()" } else { "*v" };
                if c["nullability"] == "nullable" {
                    w!(
                        "{f}:match &values[{i}] {{core::Value::{v}(v)=>Some({payload}),core::Value::Null(core::SqlType::{v})=>None,_=>return Err(())}},"
                    );
                } else {
                    w!(
                        "{f}:match &values[{i}] {{core::Value::{v}(v)=>{payload},_=>return Err(())}},"
                    );
                }
            }
            w!("}}) }}");
        }
        for (feature, session) in [("postgres", "PostgresSession"), ("mysql", "MysqlSession")] {
            w!(
                "#[cfg(all(feature=\"runtime-tokio\",feature={feature:?}))] impl<R> Read<R> {{ pub async fn execute_{feature}(self,session:&mut quent::{session},current:&core::DatabaseContext,mapping:&TypedModelContract)->Result<Rows<R>,quent::ExecutionError> {{let result=session.execute_compiled_read(current,mapping,self.compiled).await?;let decoded=result.sensitive_rows().iter().map(|row|(self.decode)(row)).collect::<Result<Vec<_>,_>>();match decoded {{Ok(rows)=>Ok(Rows(rows)),Err(())=>{{session.quarantine_generated_decode();Err(quent::ExecutionError::Decode)}}}} }} }}"
            );
        }
        w!("}}");
    }
    Ok(out.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn code_artifact_bound_never_accepts_a_truncated_write() {
        let mut out = Output(String::new());
        out.write_str(&"x".repeat(524_288)).unwrap();
        assert!(out.write_str("y").is_err());
        assert_eq!(out.0.len(), 524_288);
    }
}
