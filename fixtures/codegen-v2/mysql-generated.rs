// kind=quent_rust_code version=2 generator=quent-codegen/2; local sensitive data.
// Exact ordered canonical domains, never fingerprints or authority.
const _: &str = "{\"kind\":\"quent_schema\",\"version\":1,\"target\":{\"database\":\"dedicated_test\",\"address\":\"dedicated_loopback\",\"catalog\":\"quent_dispatch_test\",\"dialect\":\"mysql\"},\"coverage\":\"complete\",\"tables\":[{\"path\":[\"quent_dispatch_test\",\"generated_items\"],\"semantics\":\"ordinary\",\"columns_coverage\":\"complete\",\"keys_coverage\":\"complete\",\"columns\":[{\"name\":\"part\",\"native\":\"i16\",\"nullability\":\"nonnull\",\"default\":{\"state\":\"none\"}},{\"name\":\"id\",\"native\":\"i64\",\"nullability\":\"nonnull\",\"default\":{\"state\":\"none\"}},{\"name\":\"tenant\",\"native\":\"i64\",\"nullability\":\"nonnull\",\"default\":{\"state\":\"none\"}},{\"name\":\"small\",\"native\":\"i16\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"normal\",\"native\":\"i32\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"big\",\"native\":\"i64\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"flag\",\"native\":\"bool\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"txt\",\"native\":\"text\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"optional\",\"native\":\"i64\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"tiny\",\"native\":\"i8\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"u_tiny\",\"native\":\"u8\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"u_small\",\"native\":\"u16\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"u_normal\",\"native\":\"u32\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"u_big\",\"native\":\"u64\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}}],\"keys\":[{\"kind\":\"primary\",\"columns\":[\"part\",\"id\"],\"coverage\":\"complete\"}]},{\"path\":[\"quent_dispatch_test\",\"other_items\"],\"semantics\":\"ordinary\",\"columns_coverage\":\"complete\",\"keys_coverage\":\"complete\",\"columns\":[{\"name\":\"part\",\"native\":\"i16\",\"nullability\":\"nonnull\",\"default\":{\"state\":\"none\"}},{\"name\":\"id\",\"native\":\"i64\",\"nullability\":\"nonnull\",\"default\":{\"state\":\"none\"}},{\"name\":\"tenant\",\"native\":\"i64\",\"nullability\":\"nonnull\",\"default\":{\"state\":\"none\"}},{\"name\":\"small\",\"native\":\"i16\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"normal\",\"native\":\"i32\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"big\",\"native\":\"i64\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"flag\",\"native\":\"bool\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"txt\",\"native\":\"text\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"optional\",\"native\":\"i64\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"tiny\",\"native\":\"i8\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"u_tiny\",\"native\":\"u8\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"u_small\",\"native\":\"u16\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"u_normal\",\"native\":\"u32\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}},{\"name\":\"u_big\",\"native\":\"u64\",\"nullability\":\"nullable\",\"default\":{\"state\":\"none\"}}],\"keys\":[{\"kind\":\"primary\",\"columns\":[\"part\",\"id\"],\"coverage\":\"complete\"}]}]}";
const _: &str = "{\"kind\":\"quent_policy\",\"version\":1,\"target\":{\"database\":\"dedicated_test\",\"address\":\"dedicated_loopback\",\"catalog\":\"quent_dispatch_test\",\"dialect\":\"mysql\"},\"coverage\":\"complete\",\"rules\":[{\"kind\":\"tenant\",\"table\":[\"quent_dispatch_test\",\"generated_items\"],\"column\":\"tenant\"},{\"kind\":\"tenant\",\"table\":[\"quent_dispatch_test\",\"other_items\"],\"column\":\"tenant\"},{\"kind\":\"readonly\",\"table\":[\"quent_dispatch_test\",\"other_items\"],\"column\":\"small\"},{\"kind\":\"generated\",\"table\":[\"quent_dispatch_test\",\"other_items\"],\"column\":\"normal\"}]}";
const _: &str = "{\"kind\":\"quent_codegen_config\",\"version\":2,\"generator\":\"quent-codegen/2\",\"models\":[{\"table\":[\"quent_dispatch_test\",\"generated_items\"],\"columns\":[{\"name\":\"part\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"id\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"tenant\",\"role\":\"tenant\",\"read\":true},{\"name\":\"small\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"normal\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"big\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"flag\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"txt\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"optional\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"tiny\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"u_tiny\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"u_small\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"u_normal\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"u_big\",\"role\":\"ordinary\",\"read\":true}],\"projections\":[{\"name\":\"detail\",\"fields\":[\"part\",\"id\",\"small\",\"normal\",\"big\",\"flag\",\"txt\",\"optional\",\"tiny\",\"u_tiny\",\"u_small\",\"u_normal\",\"u_big\"]},{\"name\":\"compact\",\"fields\":[\"optional\",\"id\"]}]},{\"table\":[\"quent_dispatch_test\",\"other_items\"],\"columns\":[{\"name\":\"part\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"id\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"tenant\",\"role\":\"tenant\",\"read\":true},{\"name\":\"small\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"normal\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"big\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"flag\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"txt\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"optional\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"tiny\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"u_tiny\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"u_small\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"u_normal\",\"role\":\"ordinary\",\"read\":true},{\"name\":\"u_big\",\"role\":\"ordinary\",\"read\":true}],\"projections\":[{\"name\":\"detail\",\"fields\":[\"part\",\"id\",\"small\",\"normal\",\"big\",\"flag\",\"txt\",\"optional\",\"tiny\",\"u_tiny\",\"u_small\",\"u_normal\",\"u_big\"]},{\"name\":\"compact\",\"fields\":[\"optional\",\"id\"]}]}]}";
pub mod generated_items {
use quent::operation_spec::{TypedModelContract,TypedColumn,ModelCoverage,ColumnType,ColumnRole,SpecSource,CompiledRead,TypedError};
use quent::codegen::json::{self,json}; use quent_core as core; use std::marker::PhantomData;
macro_rules! redacted { ($t:ty) => { impl std::fmt::Debug for $t { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.write_str("Generated([REDACTED])") } } }; }
pub struct Model; redacted!(Model);
use quent::codegen::{PatchError,PatchSummary};
enum Change<T> { Unchanged, Set(T) } enum NullableChange<T> { Unchanged, SetNull, Set(T) }
pub struct Patch { payload:usize,
small:NullableChange<i16>,
normal:NullableChange<i32>,
big:NullableChange<i64>,
flag:NullableChange<bool>,
txt:NullableChange<String>,
optional:NullableChange<i64>,
tiny:NullableChange<i8>,
u_tiny:NullableChange<u8>,
u_small:NullableChange<u16>,
u_normal:NullableChange<u32>,
u_big:NullableChange<u64>,
} redacted!(Patch); impl Default for Patch {fn default()->Self{Self::new()}} impl Patch {pub fn new()->Self{Self{payload:0,
small:NullableChange::Unchanged,
normal:NullableChange::Unchanged,
big:NullableChange::Unchanged,
flag:NullableChange::Unchanged,
txt:NullableChange::Unchanged,
optional:NullableChange::Unchanged,
tiny:NullableChange::Unchanged,
u_tiny:NullableChange::Unchanged,
u_small:NullableChange::Unchanged,
u_normal:NullableChange::Unchanged,
u_big:NullableChange::Unchanged,
}}
pub fn set_small(mut self,v:i16)->Result<Self,PatchError>{if !matches!(self.small,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.small=NullableChange::Set(v);Ok(self)}
pub fn set_null_small(mut self)->Result<Self,PatchError>{if !matches!(self.small,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.small=NullableChange::SetNull;Ok(self)}
pub fn set_normal(mut self,v:i32)->Result<Self,PatchError>{if !matches!(self.normal,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.normal=NullableChange::Set(v);Ok(self)}
pub fn set_null_normal(mut self)->Result<Self,PatchError>{if !matches!(self.normal,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.normal=NullableChange::SetNull;Ok(self)}
pub fn set_big(mut self,v:i64)->Result<Self,PatchError>{if !matches!(self.big,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.big=NullableChange::Set(v);Ok(self)}
pub fn set_null_big(mut self)->Result<Self,PatchError>{if !matches!(self.big,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.big=NullableChange::SetNull;Ok(self)}
pub fn set_flag(mut self,v:bool)->Result<Self,PatchError>{if !matches!(self.flag,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.flag=NullableChange::Set(v);Ok(self)}
pub fn set_null_flag(mut self)->Result<Self,PatchError>{if !matches!(self.flag,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.flag=NullableChange::SetNull;Ok(self)}
pub fn set_txt(mut self,v:String)->Result<Self,PatchError>{if !matches!(self.txt,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
if v.len()>4096 || v.contains('\0'){return Err(PatchError::Native)}
self.payload+=v.len();if self.payload>65536{return Err(PatchError::Budget)}self.txt=NullableChange::Set(v);Ok(self)}
pub fn set_null_txt(mut self)->Result<Self,PatchError>{if !matches!(self.txt,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.txt=NullableChange::SetNull;Ok(self)}
pub fn set_optional(mut self,v:i64)->Result<Self,PatchError>{if !matches!(self.optional,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.optional=NullableChange::Set(v);Ok(self)}
pub fn set_null_optional(mut self)->Result<Self,PatchError>{if !matches!(self.optional,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.optional=NullableChange::SetNull;Ok(self)}
pub fn set_tiny(mut self,v:i8)->Result<Self,PatchError>{if !matches!(self.tiny,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.tiny=NullableChange::Set(v);Ok(self)}
pub fn set_null_tiny(mut self)->Result<Self,PatchError>{if !matches!(self.tiny,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.tiny=NullableChange::SetNull;Ok(self)}
pub fn set_u_tiny(mut self,v:u8)->Result<Self,PatchError>{if !matches!(self.u_tiny,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_tiny=NullableChange::Set(v);Ok(self)}
pub fn set_null_u_tiny(mut self)->Result<Self,PatchError>{if !matches!(self.u_tiny,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_tiny=NullableChange::SetNull;Ok(self)}
pub fn set_u_small(mut self,v:u16)->Result<Self,PatchError>{if !matches!(self.u_small,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_small=NullableChange::Set(v);Ok(self)}
pub fn set_null_u_small(mut self)->Result<Self,PatchError>{if !matches!(self.u_small,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_small=NullableChange::SetNull;Ok(self)}
pub fn set_u_normal(mut self,v:u32)->Result<Self,PatchError>{if !matches!(self.u_normal,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_normal=NullableChange::Set(v);Ok(self)}
pub fn set_null_u_normal(mut self)->Result<Self,PatchError>{if !matches!(self.u_normal,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_normal=NullableChange::SetNull;Ok(self)}
pub fn set_u_big(mut self,v:u64)->Result<Self,PatchError>{if !matches!(self.u_big,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_big=NullableChange::Set(v);Ok(self)}
pub fn set_null_u_big(mut self)->Result<Self,PatchError>{if !matches!(self.u_big,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_big=NullableChange::SetNull;Ok(self)}
}
pub struct CompiledPatch {prepared:core::PolicyPreparation,mapping:TypedModelContract} redacted!(CompiledPatch); impl CompiledPatch {pub fn inspect(&self)->core::Inspection<'_>{self.prepared.inspect()} pub fn public_view(&self)->PatchSummary{PatchSummary::inspection_only(self.inspect().sensitive_source().metadata().ordered_write_data().len())} pub fn matches_current(&self,current:&core::DatabaseContext,mapping:&TypedModelContract)->bool{self.mapping==*mapping && self.prepared.sensitive_source().context()==current} }
impl Repository<'_> {pub fn compile_patch(&self,key:Key,patch:Patch)->Result<CompiledPatch,PatchError>{let id=|s:&str|core::Ident::new(s).map_err(|_|PatchError::Binding);let table=core::Table::new(vec![id("quent_dispatch_test")?,id("generated_items")?],None).map_err(|_|PatchError::Binding)?;let _=&patch;#[allow(unused_mut)]let mut data=Vec::new();
match patch.small{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("small")?,core::SqlType::I16,true),core::Input::Value(core::Value::I16(v)))),
NullableChange::SetNull=>data.push((table.column(id("small")?,core::SqlType::I16,true),core::Input::Value(core::Value::Null(core::SqlType::I16)))),
}
match patch.normal{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("normal")?,core::SqlType::I32,true),core::Input::Value(core::Value::I32(v)))),
NullableChange::SetNull=>data.push((table.column(id("normal")?,core::SqlType::I32,true),core::Input::Value(core::Value::Null(core::SqlType::I32)))),
}
match patch.big{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("big")?,core::SqlType::I64,true),core::Input::Value(core::Value::I64(v)))),
NullableChange::SetNull=>data.push((table.column(id("big")?,core::SqlType::I64,true),core::Input::Value(core::Value::Null(core::SqlType::I64)))),
}
match patch.flag{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("flag")?,core::SqlType::Bool,true),core::Input::Value(core::Value::Bool(v)))),
NullableChange::SetNull=>data.push((table.column(id("flag")?,core::SqlType::Bool,true),core::Input::Value(core::Value::Null(core::SqlType::Bool)))),
}
match patch.txt{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("txt")?,core::SqlType::Text,true),core::Input::Value(core::Value::Text(v)))),
NullableChange::SetNull=>data.push((table.column(id("txt")?,core::SqlType::Text,true),core::Input::Value(core::Value::Null(core::SqlType::Text)))),
}
match patch.optional{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("optional")?,core::SqlType::I64,true),core::Input::Value(core::Value::I64(v)))),
NullableChange::SetNull=>data.push((table.column(id("optional")?,core::SqlType::I64,true),core::Input::Value(core::Value::Null(core::SqlType::I64)))),
}
match patch.tiny{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("tiny")?,core::SqlType::I8,true),core::Input::Value(core::Value::I8(v)))),
NullableChange::SetNull=>data.push((table.column(id("tiny")?,core::SqlType::I8,true),core::Input::Value(core::Value::Null(core::SqlType::I8)))),
}
match patch.u_tiny{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("u_tiny")?,core::SqlType::U8,true),core::Input::Value(core::Value::U8(v)))),
NullableChange::SetNull=>data.push((table.column(id("u_tiny")?,core::SqlType::U8,true),core::Input::Value(core::Value::Null(core::SqlType::U8)))),
}
match patch.u_small{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("u_small")?,core::SqlType::U16,true),core::Input::Value(core::Value::U16(v)))),
NullableChange::SetNull=>data.push((table.column(id("u_small")?,core::SqlType::U16,true),core::Input::Value(core::Value::Null(core::SqlType::U16)))),
}
match patch.u_normal{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("u_normal")?,core::SqlType::U32,true),core::Input::Value(core::Value::U32(v)))),
NullableChange::SetNull=>data.push((table.column(id("u_normal")?,core::SqlType::U32,true),core::Input::Value(core::Value::Null(core::SqlType::U32)))),
}
match patch.u_big{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("u_big")?,core::SqlType::U64,true),core::Input::Value(core::Value::U64(v)))),
NullableChange::SetNull=>data.push((table.column(id("u_big")?,core::SqlType::U64,true),core::Input::Value(core::Value::Null(core::SqlType::U64)))),
}
if data.is_empty(){return Err(PatchError::Empty)} let payload:usize=data.iter().map(|(_,v)|match v{core::Input::Value(core::Value::Text(s))=>s.len(),_=>8}).sum();if payload>65536{return Err(PatchError::Budget)} let predicates=vec![
core::Predicate::Compare(table.column(id("part")?,core::SqlType::I16,false),core::Compare::Eq,core::Value::I16(key.0)),
core::Predicate::Compare(table.column(id("id")?,core::SqlType::I64,false),core::Compare::Eq,core::Value::I64(key.1)),
];let operation=core::Draft::update(table,data).filter(core::Predicate::And(predicates)).freeze();let prepared=self.current.prepare().prepare_operation(&operation).map_err(|_|PatchError::Policy)?;let inspect=prepared.inspect();let final_payload:usize=inspect.sensitive_source().args().iter().map(|v|match v{core::Value::Text(s)=>s.len(),_=>8}).sum();if final_payload>65536{return Err(PatchError::Budget)}
if inspect.risk()!=core::Risk::Low || inspect.completeness()!=core::Completeness::Complete{return Err(PatchError::Gate)}Ok(CompiledPatch{prepared,mapping:self.mapping.clone()})}}
pub struct Key(i16,i64); redacted!(Key);
impl Key { pub fn new(v0:i16,v1:i64) -> Self { Self(v0,v1) } pub fn sensitive_components(&self) -> (&i16,&i64,) { (&self.0,&self.1,) } }
pub struct Column<T> { name: &'static str, tag: &'static str, marker: PhantomData<T> } impl<T> std::fmt::Debug for Column<T> { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.write_str("Generated([REDACTED])") } }
pub struct Predicate { wire: json::Value, nodes: u16, depth: u8 } redacted!(Predicate);
impl Predicate { fn leaf(wire:json::Value)->Self { Self {wire,nodes:1,depth:0} } pub fn and(self,other:Self)->Result<Self,TypedError> {self.join(other,"and")} pub fn or(self,other:Self)->Result<Self,TypedError> {self.join(other,"or")} fn join(self,other:Self,op:&str)->Result<Self,TypedError> { let nodes=self.nodes+other.nodes+1;let depth=self.depth.max(other.depth)+1;if nodes>128 || depth>8 {return Err(TypedError::NativeRefused)} Ok(Self{wire:json!({"op":op,"args":[self.wire,other.wire]}),nodes,depth}) } }
pub struct Order(json::Value); redacted!(Order); impl<T> Column<T> { pub fn asc(self)->Order {let _=self.tag; Order(json!({"field":self.name,"direction":"asc"}))} pub fn desc(self)->Order {Order(json!({"field":self.name,"direction":"desc"}))} }
impl Model { pub fn part()->Column<i16> {Column{name:"part",tag:"i16",marker:PhantomData}} }
impl Column<i16> { pub fn eq(self,v:i16)->Result<Predicate,TypedError> {
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
}
impl Model { pub fn id()->Column<i64> {Column{name:"id",tag:"i64",marker:PhantomData}} }
impl Column<i64> { pub fn eq(self,v:i64)->Result<Predicate,TypedError> {
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
}
impl Model { pub fn small()->Column<Option<i16>> {Column{name:"small",tag:"i16",marker:PhantomData}} }
impl Column<Option<i16>> { pub fn eq(self,v:Option<i16>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn normal()->Column<Option<i32>> {Column{name:"normal",tag:"i32",marker:PhantomData}} }
impl Column<Option<i32>> { pub fn eq(self,v:Option<i32>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn big()->Column<Option<i64>> {Column{name:"big",tag:"i64",marker:PhantomData}} }
impl Column<Option<i64>> { pub fn eq(self,v:Option<i64>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn flag()->Column<Option<bool>> {Column{name:"flag",tag:"bool",marker:PhantomData}} }
impl Column<Option<bool>> { pub fn eq(self,v:Option<bool>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn txt()->Column<Option<String>> {Column{name:"txt",tag:"text",marker:PhantomData}} }
impl Column<Option<String>> { pub fn eq(self,v:Option<String>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
let text=v.as_ref().unwrap(); if text.len()>4096 || text.contains('\0') {return Err(TypedError::NativeRefused)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn optional()->Column<Option<i64>> {Column{name:"optional",tag:"i64",marker:PhantomData}} }
impl Model { pub fn tiny()->Column<Option<i8>> {Column{name:"tiny",tag:"i8",marker:PhantomData}} }
impl Column<Option<i8>> { pub fn eq(self,v:Option<i8>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn u_tiny()->Column<Option<u8>> {Column{name:"u_tiny",tag:"u8",marker:PhantomData}} }
impl Column<Option<u8>> { pub fn eq(self,v:Option<u8>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn u_small()->Column<Option<u16>> {Column{name:"u_small",tag:"u16",marker:PhantomData}} }
impl Column<Option<u16>> { pub fn eq(self,v:Option<u16>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn u_normal()->Column<Option<u32>> {Column{name:"u_normal",tag:"u32",marker:PhantomData}} }
impl Column<Option<u32>> { pub fn eq(self,v:Option<u32>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn u_big()->Column<Option<u64>> {Column{name:"u_big",tag:"u64",marker:PhantomData}} }
impl Column<Option<u64>> { pub fn eq(self,v:Option<u64>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Key { pub fn predicate(self)->Result<Predicate,TypedError> {
let p0=Model::part().eq(self.0)?;
let p1=Model::id().eq(self.1)?;
let predicate=p0.and(p1)?; Ok(predicate) } }
pub struct Projection<R> { fields: &'static [&'static str], decode: fn(&[core::Value])->Result<R,()>, marker: PhantomData<R> } impl<R> std::fmt::Debug for Projection<R> {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("Generated([REDACTED])")}}
pub struct Read<R> { compiled:CompiledRead, decode:fn(&[core::Value])->Result<R,()> } impl<R> std::fmt::Debug for Read<R> {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("Generated([REDACTED])")}}
impl<R> Read<R> { pub fn inspect(&self)->core::Inspection<'_> {let _=self.decode; self.compiled.inspect()} pub fn public_view(&self)->core::PlanView {self.compiled.public_view()} }
pub struct Rows<R>(Vec<R>); impl<R> Rows<R> {pub fn sensitive_rows(&self)->&[R] {&self.0}} impl<R> std::fmt::Debug for Rows<R> {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("Generated([REDACTED])")}}
pub struct Repository<'a> { current: &'a core::DatabaseContext, mapping: &'a TypedModelContract } redacted!(Repository<'_>);
impl<'a> Repository<'a> { pub fn bind(current:&'a core::DatabaseContext,mapping:&'a TypedModelContract)->Result<Self,TypedError> {
let sensitive=current.sensitive_source(); let target=sensitive.target(); if target.0!="dedicated_test" || target.1!="dedicated_loopback" || target.2.sensitive_text()!="quent_dispatch_test" || target.3!=core::Dialect::Mysql {return Err(TypedError::SnapshotMismatch)}
let id=|s:&str| core::Ident::new(s).map_err(|_|TypedError::Mapping); let table=core::Table::new(vec![id("quent_dispatch_test")?,id("generated_items")?],None).map_err(|_|TypedError::Mapping)?;
let columns=vec![
table.column(id("part")?,core::SqlType::I16,false),
table.column(id("id")?,core::SqlType::I64,false),
table.column(id("tenant")?,core::SqlType::I64,false),
table.column(id("small")?,core::SqlType::I16,true),
table.column(id("normal")?,core::SqlType::I32,true),
table.column(id("big")?,core::SqlType::I64,true),
table.column(id("flag")?,core::SqlType::Bool,true),
table.column(id("txt")?,core::SqlType::Text,true),
table.column(id("optional")?,core::SqlType::I64,true),
table.column(id("tiny")?,core::SqlType::I8,true),
table.column(id("u_tiny")?,core::SqlType::U8,true),
table.column(id("u_small")?,core::SqlType::U16,true),
table.column(id("u_normal")?,core::SqlType::U32,true),
table.column(id("u_big")?,core::SqlType::U64,true),
]; let expected=TypedModelContract::application_asserted("generated_items",table.clone(),vec![
TypedColumn::application_supplied("part",columns[0].sensitive_name().clone(),ColumnType::Native(columns[0].sql_type()),columns[0].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("id",columns[1].sensitive_name().clone(),ColumnType::Native(columns[1].sql_type()),columns[1].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("tenant",columns[2].sensitive_name().clone(),ColumnType::Native(columns[2].sql_type()),columns[2].nullable(),ColumnRole::Tenant,true)?,
TypedColumn::application_supplied("small",columns[3].sensitive_name().clone(),ColumnType::Native(columns[3].sql_type()),columns[3].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("normal",columns[4].sensitive_name().clone(),ColumnType::Native(columns[4].sql_type()),columns[4].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("big",columns[5].sensitive_name().clone(),ColumnType::Native(columns[5].sql_type()),columns[5].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("flag",columns[6].sensitive_name().clone(),ColumnType::Native(columns[6].sql_type()),columns[6].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("txt",columns[7].sensitive_name().clone(),ColumnType::Native(columns[7].sql_type()),columns[7].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("optional",columns[8].sensitive_name().clone(),ColumnType::Native(columns[8].sql_type()),columns[8].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("tiny",columns[9].sensitive_name().clone(),ColumnType::Native(columns[9].sql_type()),columns[9].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("u_tiny",columns[10].sensitive_name().clone(),ColumnType::Native(columns[10].sql_type()),columns[10].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("u_small",columns[11].sensitive_name().clone(),ColumnType::Native(columns[11].sql_type()),columns[11].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("u_normal",columns[12].sensitive_name().clone(),ColumnType::Native(columns[12].sql_type()),columns[12].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("u_big",columns[13].sensitive_name().clone(),ColumnType::Native(columns[13].sql_type()),columns[13].nullable(),ColumnRole::Ordinary,true)?,
],ModelCoverage::Complete,current)?; if expected!=*mapping {return Err(TypedError::Mapping)}
let facts=core::ScopeFacts::application_asserted(table.clone(),core::Dialect::Mysql,columns.clone(),vec![
core::Key::Primary(vec![columns[0].clone(),columns[1].clone()]),
],core::TableContext::OrdinaryBaseTableWithoutEffects).map_err(|_|TypedError::Mapping)?; if !current.sensitive_source().schema().is_some_and(|(_,_,facts_list)|facts_list.contains(&facts)) {return Err(TypedError::Mapping)}
let policies:Vec<core::PolicyDeclaration>=vec![
core::PolicyDeclaration::Tenant(columns[2].clone()),
]; let actual:Vec<_>=current.sensitive_source().declarations().iter().filter(|p|{let c=match p { core::PolicyDeclaration::Tenant(c)|core::PolicyDeclaration::SoftDelete(c)|core::PolicyDeclaration::Pii(c)|core::PolicyDeclaration::RequiredFilter(c)|core::PolicyDeclaration::Readonly(c)|core::PolicyDeclaration::Generated(c)=>c };c.sensitive_table()==&table}).cloned().collect(); if policies!=actual {return Err(TypedError::Role)} Ok(Self{current,mapping}) }
pub fn select<R>(&self,projection:Projection<R>,filter:Option<Predicate>,order:Vec<Order>,limit:u16)->Result<Read<R>,TypedError> { if order.len()>32 || limit==0 || limit>256 {return Err(TypedError::NativeRefused)} let mut wire=json!({"kind":"quent_operation_spec","version":1,"model":"generated_items","fields":projection.fields,"order":order.into_iter().map(|o|o.0).collect::<Vec<_>>(),"limit":limit});if let Some(p)=filter{wire["filter"]=p.wire;} let bytes=json::to_vec(&wire).map_err(|_|TypedError::NativeRefused)?;let compiled=SpecSource::parse(&bytes).map_err(|_|TypedError::NativeRefused)?.compile(self.mapping,self.current)?;Ok(Read{compiled,decode:projection.decode}) } }
pub struct DetailRow {
part: i16,
id: i64,
small: Option<i16>,
normal: Option<i32>,
big: Option<i64>,
flag: Option<bool>,
txt: Option<String>,
optional: Option<i64>,
tiny: Option<i8>,
u_tiny: Option<u8>,
u_small: Option<u16>,
u_normal: Option<u32>,
u_big: Option<u64>,
} redacted!(DetailRow); impl DetailRow {
pub fn sensitive_part(&self)->&i16 {&self.part}
pub fn sensitive_id(&self)->&i64 {&self.id}
pub fn sensitive_small(&self)->&Option<i16> {&self.small}
pub fn sensitive_normal(&self)->&Option<i32> {&self.normal}
pub fn sensitive_big(&self)->&Option<i64> {&self.big}
pub fn sensitive_flag(&self)->&Option<bool> {&self.flag}
pub fn sensitive_txt(&self)->&Option<String> {&self.txt}
pub fn sensitive_optional(&self)->&Option<i64> {&self.optional}
pub fn sensitive_tiny(&self)->&Option<i8> {&self.tiny}
pub fn sensitive_u_tiny(&self)->&Option<u8> {&self.u_tiny}
pub fn sensitive_u_small(&self)->&Option<u16> {&self.u_small}
pub fn sensitive_u_normal(&self)->&Option<u32> {&self.u_normal}
pub fn sensitive_u_big(&self)->&Option<u64> {&self.u_big}
} impl Model {pub fn detail()->Projection<DetailRow> {Projection{fields:&["part", "id", "small", "normal", "big", "flag", "txt", "optional", "tiny", "u_tiny", "u_small", "u_normal", "u_big"],decode:decode_detail,marker:PhantomData}} }
fn decode_detail(values:&[core::Value])->Result<DetailRow,()> { if values.len()!=13 {return Err(())} Ok(DetailRow{
part:match &values[0] {core::Value::I16(v)=>*v,_=>return Err(())},
id:match &values[1] {core::Value::I64(v)=>*v,_=>return Err(())},
small:match &values[2] {core::Value::I16(v)=>Some(*v),core::Value::Null(core::SqlType::I16)=>None,_=>return Err(())},
normal:match &values[3] {core::Value::I32(v)=>Some(*v),core::Value::Null(core::SqlType::I32)=>None,_=>return Err(())},
big:match &values[4] {core::Value::I64(v)=>Some(*v),core::Value::Null(core::SqlType::I64)=>None,_=>return Err(())},
flag:match &values[5] {core::Value::Bool(v)=>Some(*v),core::Value::Null(core::SqlType::Bool)=>None,_=>return Err(())},
txt:match &values[6] {core::Value::Text(v)=>Some(v.clone()),core::Value::Null(core::SqlType::Text)=>None,_=>return Err(())},
optional:match &values[7] {core::Value::I64(v)=>Some(*v),core::Value::Null(core::SqlType::I64)=>None,_=>return Err(())},
tiny:match &values[8] {core::Value::I8(v)=>Some(*v),core::Value::Null(core::SqlType::I8)=>None,_=>return Err(())},
u_tiny:match &values[9] {core::Value::U8(v)=>Some(*v),core::Value::Null(core::SqlType::U8)=>None,_=>return Err(())},
u_small:match &values[10] {core::Value::U16(v)=>Some(*v),core::Value::Null(core::SqlType::U16)=>None,_=>return Err(())},
u_normal:match &values[11] {core::Value::U32(v)=>Some(*v),core::Value::Null(core::SqlType::U32)=>None,_=>return Err(())},
u_big:match &values[12] {core::Value::U64(v)=>Some(*v),core::Value::Null(core::SqlType::U64)=>None,_=>return Err(())},
}) }
pub struct CompactRow {
optional: Option<i64>,
id: i64,
} redacted!(CompactRow); impl CompactRow {
pub fn sensitive_optional(&self)->&Option<i64> {&self.optional}
pub fn sensitive_id(&self)->&i64 {&self.id}
} impl Model {pub fn compact()->Projection<CompactRow> {Projection{fields:&["optional", "id"],decode:decode_compact,marker:PhantomData}} }
fn decode_compact(values:&[core::Value])->Result<CompactRow,()> { if values.len()!=2 {return Err(())} Ok(CompactRow{
optional:match &values[0] {core::Value::I64(v)=>Some(*v),core::Value::Null(core::SqlType::I64)=>None,_=>return Err(())},
id:match &values[1] {core::Value::I64(v)=>*v,_=>return Err(())},
}) }
#[cfg(all(feature="runtime-tokio",feature="postgres"))] impl<R> Read<R> { pub async fn execute_postgres(self,session:&mut quent::PostgresSession,current:&core::DatabaseContext,mapping:&TypedModelContract)->Result<Rows<R>,quent::ExecutionError> {let result=session.execute_compiled_read(current,mapping,self.compiled).await?;let decoded=result.sensitive_rows().iter().map(|row|(self.decode)(row)).collect::<Result<Vec<_>,_>>();match decoded {Ok(rows)=>Ok(Rows(rows)),Err(())=>{session.quarantine_generated_decode();Err(quent::ExecutionError::Decode)}} } }
#[cfg(all(feature="runtime-tokio",feature="mysql"))] impl<R> Read<R> { pub async fn execute_mysql(self,session:&mut quent::MysqlSession,current:&core::DatabaseContext,mapping:&TypedModelContract)->Result<Rows<R>,quent::ExecutionError> {let result=session.execute_compiled_read(current,mapping,self.compiled).await?;let decoded=result.sensitive_rows().iter().map(|row|(self.decode)(row)).collect::<Result<Vec<_>,_>>();match decoded {Ok(rows)=>Ok(Rows(rows)),Err(())=>{session.quarantine_generated_decode();Err(quent::ExecutionError::Decode)}} } }
}
pub mod other_items {
use quent::operation_spec::{TypedModelContract,TypedColumn,ModelCoverage,ColumnType,ColumnRole,SpecSource,CompiledRead,TypedError};
use quent::codegen::json::{self,json}; use quent_core as core; use std::marker::PhantomData;
macro_rules! redacted { ($t:ty) => { impl std::fmt::Debug for $t { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.write_str("Generated([REDACTED])") } } }; }
pub struct Model; redacted!(Model);
use quent::codegen::{PatchError,PatchSummary};
enum Change<T> { Unchanged, Set(T) } enum NullableChange<T> { Unchanged, SetNull, Set(T) }
pub struct Patch { payload:usize,
big:NullableChange<i64>,
flag:NullableChange<bool>,
txt:NullableChange<String>,
optional:NullableChange<i64>,
tiny:NullableChange<i8>,
u_tiny:NullableChange<u8>,
u_small:NullableChange<u16>,
u_normal:NullableChange<u32>,
u_big:NullableChange<u64>,
} redacted!(Patch); impl Default for Patch {fn default()->Self{Self::new()}} impl Patch {pub fn new()->Self{Self{payload:0,
big:NullableChange::Unchanged,
flag:NullableChange::Unchanged,
txt:NullableChange::Unchanged,
optional:NullableChange::Unchanged,
tiny:NullableChange::Unchanged,
u_tiny:NullableChange::Unchanged,
u_small:NullableChange::Unchanged,
u_normal:NullableChange::Unchanged,
u_big:NullableChange::Unchanged,
}}
pub fn set_big(mut self,v:i64)->Result<Self,PatchError>{if !matches!(self.big,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.big=NullableChange::Set(v);Ok(self)}
pub fn set_null_big(mut self)->Result<Self,PatchError>{if !matches!(self.big,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.big=NullableChange::SetNull;Ok(self)}
pub fn set_flag(mut self,v:bool)->Result<Self,PatchError>{if !matches!(self.flag,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.flag=NullableChange::Set(v);Ok(self)}
pub fn set_null_flag(mut self)->Result<Self,PatchError>{if !matches!(self.flag,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.flag=NullableChange::SetNull;Ok(self)}
pub fn set_txt(mut self,v:String)->Result<Self,PatchError>{if !matches!(self.txt,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
if v.len()>4096 || v.contains('\0'){return Err(PatchError::Native)}
self.payload+=v.len();if self.payload>65536{return Err(PatchError::Budget)}self.txt=NullableChange::Set(v);Ok(self)}
pub fn set_null_txt(mut self)->Result<Self,PatchError>{if !matches!(self.txt,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.txt=NullableChange::SetNull;Ok(self)}
pub fn set_optional(mut self,v:i64)->Result<Self,PatchError>{if !matches!(self.optional,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.optional=NullableChange::Set(v);Ok(self)}
pub fn set_null_optional(mut self)->Result<Self,PatchError>{if !matches!(self.optional,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.optional=NullableChange::SetNull;Ok(self)}
pub fn set_tiny(mut self,v:i8)->Result<Self,PatchError>{if !matches!(self.tiny,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.tiny=NullableChange::Set(v);Ok(self)}
pub fn set_null_tiny(mut self)->Result<Self,PatchError>{if !matches!(self.tiny,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.tiny=NullableChange::SetNull;Ok(self)}
pub fn set_u_tiny(mut self,v:u8)->Result<Self,PatchError>{if !matches!(self.u_tiny,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_tiny=NullableChange::Set(v);Ok(self)}
pub fn set_null_u_tiny(mut self)->Result<Self,PatchError>{if !matches!(self.u_tiny,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_tiny=NullableChange::SetNull;Ok(self)}
pub fn set_u_small(mut self,v:u16)->Result<Self,PatchError>{if !matches!(self.u_small,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_small=NullableChange::Set(v);Ok(self)}
pub fn set_null_u_small(mut self)->Result<Self,PatchError>{if !matches!(self.u_small,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_small=NullableChange::SetNull;Ok(self)}
pub fn set_u_normal(mut self,v:u32)->Result<Self,PatchError>{if !matches!(self.u_normal,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_normal=NullableChange::Set(v);Ok(self)}
pub fn set_null_u_normal(mut self)->Result<Self,PatchError>{if !matches!(self.u_normal,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_normal=NullableChange::SetNull;Ok(self)}
pub fn set_u_big(mut self,v:u64)->Result<Self,PatchError>{if !matches!(self.u_big,NullableChange::Unchanged){return Err(PatchError::Duplicate)}
self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_big=NullableChange::Set(v);Ok(self)}
pub fn set_null_u_big(mut self)->Result<Self,PatchError>{if !matches!(self.u_big,NullableChange::Unchanged){return Err(PatchError::Duplicate)}self.payload+=8;if self.payload>65536{return Err(PatchError::Budget)}self.u_big=NullableChange::SetNull;Ok(self)}
}
pub struct CompiledPatch {prepared:core::PolicyPreparation,mapping:TypedModelContract} redacted!(CompiledPatch); impl CompiledPatch {pub fn inspect(&self)->core::Inspection<'_>{self.prepared.inspect()} pub fn public_view(&self)->PatchSummary{PatchSummary::inspection_only(self.inspect().sensitive_source().metadata().ordered_write_data().len())} pub fn matches_current(&self,current:&core::DatabaseContext,mapping:&TypedModelContract)->bool{self.mapping==*mapping && self.prepared.sensitive_source().context()==current} }
impl Repository<'_> {pub fn compile_patch(&self,key:Key,patch:Patch)->Result<CompiledPatch,PatchError>{let id=|s:&str|core::Ident::new(s).map_err(|_|PatchError::Binding);let table=core::Table::new(vec![id("quent_dispatch_test")?,id("other_items")?],None).map_err(|_|PatchError::Binding)?;let _=&patch;#[allow(unused_mut)]let mut data=Vec::new();
match patch.big{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("big")?,core::SqlType::I64,true),core::Input::Value(core::Value::I64(v)))),
NullableChange::SetNull=>data.push((table.column(id("big")?,core::SqlType::I64,true),core::Input::Value(core::Value::Null(core::SqlType::I64)))),
}
match patch.flag{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("flag")?,core::SqlType::Bool,true),core::Input::Value(core::Value::Bool(v)))),
NullableChange::SetNull=>data.push((table.column(id("flag")?,core::SqlType::Bool,true),core::Input::Value(core::Value::Null(core::SqlType::Bool)))),
}
match patch.txt{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("txt")?,core::SqlType::Text,true),core::Input::Value(core::Value::Text(v)))),
NullableChange::SetNull=>data.push((table.column(id("txt")?,core::SqlType::Text,true),core::Input::Value(core::Value::Null(core::SqlType::Text)))),
}
match patch.optional{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("optional")?,core::SqlType::I64,true),core::Input::Value(core::Value::I64(v)))),
NullableChange::SetNull=>data.push((table.column(id("optional")?,core::SqlType::I64,true),core::Input::Value(core::Value::Null(core::SqlType::I64)))),
}
match patch.tiny{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("tiny")?,core::SqlType::I8,true),core::Input::Value(core::Value::I8(v)))),
NullableChange::SetNull=>data.push((table.column(id("tiny")?,core::SqlType::I8,true),core::Input::Value(core::Value::Null(core::SqlType::I8)))),
}
match patch.u_tiny{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("u_tiny")?,core::SqlType::U8,true),core::Input::Value(core::Value::U8(v)))),
NullableChange::SetNull=>data.push((table.column(id("u_tiny")?,core::SqlType::U8,true),core::Input::Value(core::Value::Null(core::SqlType::U8)))),
}
match patch.u_small{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("u_small")?,core::SqlType::U16,true),core::Input::Value(core::Value::U16(v)))),
NullableChange::SetNull=>data.push((table.column(id("u_small")?,core::SqlType::U16,true),core::Input::Value(core::Value::Null(core::SqlType::U16)))),
}
match patch.u_normal{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("u_normal")?,core::SqlType::U32,true),core::Input::Value(core::Value::U32(v)))),
NullableChange::SetNull=>data.push((table.column(id("u_normal")?,core::SqlType::U32,true),core::Input::Value(core::Value::Null(core::SqlType::U32)))),
}
match patch.u_big{NullableChange::Unchanged=>(),NullableChange::Set(v)=>data.push((table.column(id("u_big")?,core::SqlType::U64,true),core::Input::Value(core::Value::U64(v)))),
NullableChange::SetNull=>data.push((table.column(id("u_big")?,core::SqlType::U64,true),core::Input::Value(core::Value::Null(core::SqlType::U64)))),
}
if data.is_empty(){return Err(PatchError::Empty)} let payload:usize=data.iter().map(|(_,v)|match v{core::Input::Value(core::Value::Text(s))=>s.len(),_=>8}).sum();if payload>65536{return Err(PatchError::Budget)} let predicates=vec![
core::Predicate::Compare(table.column(id("part")?,core::SqlType::I16,false),core::Compare::Eq,core::Value::I16(key.0)),
core::Predicate::Compare(table.column(id("id")?,core::SqlType::I64,false),core::Compare::Eq,core::Value::I64(key.1)),
];let operation=core::Draft::update(table,data).filter(core::Predicate::And(predicates)).freeze();let prepared=self.current.prepare().prepare_operation(&operation).map_err(|_|PatchError::Policy)?;let inspect=prepared.inspect();let final_payload:usize=inspect.sensitive_source().args().iter().map(|v|match v{core::Value::Text(s)=>s.len(),_=>8}).sum();if final_payload>65536{return Err(PatchError::Budget)}
if inspect.risk()!=core::Risk::Low || inspect.completeness()!=core::Completeness::Complete{return Err(PatchError::Gate)}Ok(CompiledPatch{prepared,mapping:self.mapping.clone()})}}
pub struct Key(i16,i64); redacted!(Key);
impl Key { pub fn new(v0:i16,v1:i64) -> Self { Self(v0,v1) } pub fn sensitive_components(&self) -> (&i16,&i64,) { (&self.0,&self.1,) } }
pub struct Column<T> { name: &'static str, tag: &'static str, marker: PhantomData<T> } impl<T> std::fmt::Debug for Column<T> { fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result { f.write_str("Generated([REDACTED])") } }
pub struct Predicate { wire: json::Value, nodes: u16, depth: u8 } redacted!(Predicate);
impl Predicate { fn leaf(wire:json::Value)->Self { Self {wire,nodes:1,depth:0} } pub fn and(self,other:Self)->Result<Self,TypedError> {self.join(other,"and")} pub fn or(self,other:Self)->Result<Self,TypedError> {self.join(other,"or")} fn join(self,other:Self,op:&str)->Result<Self,TypedError> { let nodes=self.nodes+other.nodes+1;let depth=self.depth.max(other.depth)+1;if nodes>128 || depth>8 {return Err(TypedError::NativeRefused)} Ok(Self{wire:json!({"op":op,"args":[self.wire,other.wire]}),nodes,depth}) } }
pub struct Order(json::Value); redacted!(Order); impl<T> Column<T> { pub fn asc(self)->Order {let _=self.tag; Order(json!({"field":self.name,"direction":"asc"}))} pub fn desc(self)->Order {Order(json!({"field":self.name,"direction":"desc"}))} }
impl Model { pub fn part()->Column<i16> {Column{name:"part",tag:"i16",marker:PhantomData}} }
impl Column<i16> { pub fn eq(self,v:i16)->Result<Predicate,TypedError> {
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
}
impl Model { pub fn id()->Column<i64> {Column{name:"id",tag:"i64",marker:PhantomData}} }
impl Column<i64> { pub fn eq(self,v:i64)->Result<Predicate,TypedError> {
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
}
impl Model { pub fn small()->Column<Option<i16>> {Column{name:"small",tag:"i16",marker:PhantomData}} }
impl Column<Option<i16>> { pub fn eq(self,v:Option<i16>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn normal()->Column<Option<i32>> {Column{name:"normal",tag:"i32",marker:PhantomData}} }
impl Column<Option<i32>> { pub fn eq(self,v:Option<i32>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn big()->Column<Option<i64>> {Column{name:"big",tag:"i64",marker:PhantomData}} }
impl Column<Option<i64>> { pub fn eq(self,v:Option<i64>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn flag()->Column<Option<bool>> {Column{name:"flag",tag:"bool",marker:PhantomData}} }
impl Column<Option<bool>> { pub fn eq(self,v:Option<bool>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn txt()->Column<Option<String>> {Column{name:"txt",tag:"text",marker:PhantomData}} }
impl Column<Option<String>> { pub fn eq(self,v:Option<String>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
let text=v.as_ref().unwrap(); if text.len()>4096 || text.contains('\0') {return Err(TypedError::NativeRefused)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn optional()->Column<Option<i64>> {Column{name:"optional",tag:"i64",marker:PhantomData}} }
impl Model { pub fn tiny()->Column<Option<i8>> {Column{name:"tiny",tag:"i8",marker:PhantomData}} }
impl Column<Option<i8>> { pub fn eq(self,v:Option<i8>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn u_tiny()->Column<Option<u8>> {Column{name:"u_tiny",tag:"u8",marker:PhantomData}} }
impl Column<Option<u8>> { pub fn eq(self,v:Option<u8>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn u_small()->Column<Option<u16>> {Column{name:"u_small",tag:"u16",marker:PhantomData}} }
impl Column<Option<u16>> { pub fn eq(self,v:Option<u16>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn u_normal()->Column<Option<u32>> {Column{name:"u_normal",tag:"u32",marker:PhantomData}} }
impl Column<Option<u32>> { pub fn eq(self,v:Option<u32>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Model { pub fn u_big()->Column<Option<u64>> {Column{name:"u_big",tag:"u64",marker:PhantomData}} }
impl Column<Option<u64>> { pub fn eq(self,v:Option<u64>)->Result<Predicate,TypedError> {
if v.is_none() {return Err(TypedError::NullComparison)}
Ok(Predicate::leaf(json!({"op":"eq","field":self.name,"value":{"type":self.tag,"value":v}}))) }
pub fn is_null(&self)->Predicate { Predicate::leaf(json!({"op":"is_null","field":self.name})) }
}
impl Key { pub fn predicate(self)->Result<Predicate,TypedError> {
let p0=Model::part().eq(self.0)?;
let p1=Model::id().eq(self.1)?;
let predicate=p0.and(p1)?; Ok(predicate) } }
pub struct Projection<R> { fields: &'static [&'static str], decode: fn(&[core::Value])->Result<R,()>, marker: PhantomData<R> } impl<R> std::fmt::Debug for Projection<R> {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("Generated([REDACTED])")}}
pub struct Read<R> { compiled:CompiledRead, decode:fn(&[core::Value])->Result<R,()> } impl<R> std::fmt::Debug for Read<R> {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("Generated([REDACTED])")}}
impl<R> Read<R> { pub fn inspect(&self)->core::Inspection<'_> {let _=self.decode; self.compiled.inspect()} pub fn public_view(&self)->core::PlanView {self.compiled.public_view()} }
pub struct Rows<R>(Vec<R>); impl<R> Rows<R> {pub fn sensitive_rows(&self)->&[R] {&self.0}} impl<R> std::fmt::Debug for Rows<R> {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{f.write_str("Generated([REDACTED])")}}
pub struct Repository<'a> { current: &'a core::DatabaseContext, mapping: &'a TypedModelContract } redacted!(Repository<'_>);
impl<'a> Repository<'a> { pub fn bind(current:&'a core::DatabaseContext,mapping:&'a TypedModelContract)->Result<Self,TypedError> {
let sensitive=current.sensitive_source(); let target=sensitive.target(); if target.0!="dedicated_test" || target.1!="dedicated_loopback" || target.2.sensitive_text()!="quent_dispatch_test" || target.3!=core::Dialect::Mysql {return Err(TypedError::SnapshotMismatch)}
let id=|s:&str| core::Ident::new(s).map_err(|_|TypedError::Mapping); let table=core::Table::new(vec![id("quent_dispatch_test")?,id("other_items")?],None).map_err(|_|TypedError::Mapping)?;
let columns=vec![
table.column(id("part")?,core::SqlType::I16,false),
table.column(id("id")?,core::SqlType::I64,false),
table.column(id("tenant")?,core::SqlType::I64,false),
table.column(id("small")?,core::SqlType::I16,true),
table.column(id("normal")?,core::SqlType::I32,true),
table.column(id("big")?,core::SqlType::I64,true),
table.column(id("flag")?,core::SqlType::Bool,true),
table.column(id("txt")?,core::SqlType::Text,true),
table.column(id("optional")?,core::SqlType::I64,true),
table.column(id("tiny")?,core::SqlType::I8,true),
table.column(id("u_tiny")?,core::SqlType::U8,true),
table.column(id("u_small")?,core::SqlType::U16,true),
table.column(id("u_normal")?,core::SqlType::U32,true),
table.column(id("u_big")?,core::SqlType::U64,true),
]; let expected=TypedModelContract::application_asserted("other_items",table.clone(),vec![
TypedColumn::application_supplied("part",columns[0].sensitive_name().clone(),ColumnType::Native(columns[0].sql_type()),columns[0].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("id",columns[1].sensitive_name().clone(),ColumnType::Native(columns[1].sql_type()),columns[1].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("tenant",columns[2].sensitive_name().clone(),ColumnType::Native(columns[2].sql_type()),columns[2].nullable(),ColumnRole::Tenant,true)?,
TypedColumn::application_supplied("small",columns[3].sensitive_name().clone(),ColumnType::Native(columns[3].sql_type()),columns[3].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("normal",columns[4].sensitive_name().clone(),ColumnType::Native(columns[4].sql_type()),columns[4].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("big",columns[5].sensitive_name().clone(),ColumnType::Native(columns[5].sql_type()),columns[5].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("flag",columns[6].sensitive_name().clone(),ColumnType::Native(columns[6].sql_type()),columns[6].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("txt",columns[7].sensitive_name().clone(),ColumnType::Native(columns[7].sql_type()),columns[7].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("optional",columns[8].sensitive_name().clone(),ColumnType::Native(columns[8].sql_type()),columns[8].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("tiny",columns[9].sensitive_name().clone(),ColumnType::Native(columns[9].sql_type()),columns[9].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("u_tiny",columns[10].sensitive_name().clone(),ColumnType::Native(columns[10].sql_type()),columns[10].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("u_small",columns[11].sensitive_name().clone(),ColumnType::Native(columns[11].sql_type()),columns[11].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("u_normal",columns[12].sensitive_name().clone(),ColumnType::Native(columns[12].sql_type()),columns[12].nullable(),ColumnRole::Ordinary,true)?,
TypedColumn::application_supplied("u_big",columns[13].sensitive_name().clone(),ColumnType::Native(columns[13].sql_type()),columns[13].nullable(),ColumnRole::Ordinary,true)?,
],ModelCoverage::Complete,current)?; if expected!=*mapping {return Err(TypedError::Mapping)}
let facts=core::ScopeFacts::application_asserted(table.clone(),core::Dialect::Mysql,columns.clone(),vec![
core::Key::Primary(vec![columns[0].clone(),columns[1].clone()]),
],core::TableContext::OrdinaryBaseTableWithoutEffects).map_err(|_|TypedError::Mapping)?; if !current.sensitive_source().schema().is_some_and(|(_,_,facts_list)|facts_list.contains(&facts)) {return Err(TypedError::Mapping)}
let policies:Vec<core::PolicyDeclaration>=vec![
core::PolicyDeclaration::Tenant(columns[2].clone()),
core::PolicyDeclaration::Readonly(columns[3].clone()),
core::PolicyDeclaration::Generated(columns[4].clone()),
]; let actual:Vec<_>=current.sensitive_source().declarations().iter().filter(|p|{let c=match p { core::PolicyDeclaration::Tenant(c)|core::PolicyDeclaration::SoftDelete(c)|core::PolicyDeclaration::Pii(c)|core::PolicyDeclaration::RequiredFilter(c)|core::PolicyDeclaration::Readonly(c)|core::PolicyDeclaration::Generated(c)=>c };c.sensitive_table()==&table}).cloned().collect(); if policies!=actual {return Err(TypedError::Role)} Ok(Self{current,mapping}) }
pub fn select<R>(&self,projection:Projection<R>,filter:Option<Predicate>,order:Vec<Order>,limit:u16)->Result<Read<R>,TypedError> { if order.len()>32 || limit==0 || limit>256 {return Err(TypedError::NativeRefused)} let mut wire=json!({"kind":"quent_operation_spec","version":1,"model":"other_items","fields":projection.fields,"order":order.into_iter().map(|o|o.0).collect::<Vec<_>>(),"limit":limit});if let Some(p)=filter{wire["filter"]=p.wire;} let bytes=json::to_vec(&wire).map_err(|_|TypedError::NativeRefused)?;let compiled=SpecSource::parse(&bytes).map_err(|_|TypedError::NativeRefused)?.compile(self.mapping,self.current)?;Ok(Read{compiled,decode:projection.decode}) } }
pub struct DetailRow {
part: i16,
id: i64,
small: Option<i16>,
normal: Option<i32>,
big: Option<i64>,
flag: Option<bool>,
txt: Option<String>,
optional: Option<i64>,
tiny: Option<i8>,
u_tiny: Option<u8>,
u_small: Option<u16>,
u_normal: Option<u32>,
u_big: Option<u64>,
} redacted!(DetailRow); impl DetailRow {
pub fn sensitive_part(&self)->&i16 {&self.part}
pub fn sensitive_id(&self)->&i64 {&self.id}
pub fn sensitive_small(&self)->&Option<i16> {&self.small}
pub fn sensitive_normal(&self)->&Option<i32> {&self.normal}
pub fn sensitive_big(&self)->&Option<i64> {&self.big}
pub fn sensitive_flag(&self)->&Option<bool> {&self.flag}
pub fn sensitive_txt(&self)->&Option<String> {&self.txt}
pub fn sensitive_optional(&self)->&Option<i64> {&self.optional}
pub fn sensitive_tiny(&self)->&Option<i8> {&self.tiny}
pub fn sensitive_u_tiny(&self)->&Option<u8> {&self.u_tiny}
pub fn sensitive_u_small(&self)->&Option<u16> {&self.u_small}
pub fn sensitive_u_normal(&self)->&Option<u32> {&self.u_normal}
pub fn sensitive_u_big(&self)->&Option<u64> {&self.u_big}
} impl Model {pub fn detail()->Projection<DetailRow> {Projection{fields:&["part", "id", "small", "normal", "big", "flag", "txt", "optional", "tiny", "u_tiny", "u_small", "u_normal", "u_big"],decode:decode_detail,marker:PhantomData}} }
fn decode_detail(values:&[core::Value])->Result<DetailRow,()> { if values.len()!=13 {return Err(())} Ok(DetailRow{
part:match &values[0] {core::Value::I16(v)=>*v,_=>return Err(())},
id:match &values[1] {core::Value::I64(v)=>*v,_=>return Err(())},
small:match &values[2] {core::Value::I16(v)=>Some(*v),core::Value::Null(core::SqlType::I16)=>None,_=>return Err(())},
normal:match &values[3] {core::Value::I32(v)=>Some(*v),core::Value::Null(core::SqlType::I32)=>None,_=>return Err(())},
big:match &values[4] {core::Value::I64(v)=>Some(*v),core::Value::Null(core::SqlType::I64)=>None,_=>return Err(())},
flag:match &values[5] {core::Value::Bool(v)=>Some(*v),core::Value::Null(core::SqlType::Bool)=>None,_=>return Err(())},
txt:match &values[6] {core::Value::Text(v)=>Some(v.clone()),core::Value::Null(core::SqlType::Text)=>None,_=>return Err(())},
optional:match &values[7] {core::Value::I64(v)=>Some(*v),core::Value::Null(core::SqlType::I64)=>None,_=>return Err(())},
tiny:match &values[8] {core::Value::I8(v)=>Some(*v),core::Value::Null(core::SqlType::I8)=>None,_=>return Err(())},
u_tiny:match &values[9] {core::Value::U8(v)=>Some(*v),core::Value::Null(core::SqlType::U8)=>None,_=>return Err(())},
u_small:match &values[10] {core::Value::U16(v)=>Some(*v),core::Value::Null(core::SqlType::U16)=>None,_=>return Err(())},
u_normal:match &values[11] {core::Value::U32(v)=>Some(*v),core::Value::Null(core::SqlType::U32)=>None,_=>return Err(())},
u_big:match &values[12] {core::Value::U64(v)=>Some(*v),core::Value::Null(core::SqlType::U64)=>None,_=>return Err(())},
}) }
pub struct CompactRow {
optional: Option<i64>,
id: i64,
} redacted!(CompactRow); impl CompactRow {
pub fn sensitive_optional(&self)->&Option<i64> {&self.optional}
pub fn sensitive_id(&self)->&i64 {&self.id}
} impl Model {pub fn compact()->Projection<CompactRow> {Projection{fields:&["optional", "id"],decode:decode_compact,marker:PhantomData}} }
fn decode_compact(values:&[core::Value])->Result<CompactRow,()> { if values.len()!=2 {return Err(())} Ok(CompactRow{
optional:match &values[0] {core::Value::I64(v)=>Some(*v),core::Value::Null(core::SqlType::I64)=>None,_=>return Err(())},
id:match &values[1] {core::Value::I64(v)=>*v,_=>return Err(())},
}) }
#[cfg(all(feature="runtime-tokio",feature="postgres"))] impl<R> Read<R> { pub async fn execute_postgres(self,session:&mut quent::PostgresSession,current:&core::DatabaseContext,mapping:&TypedModelContract)->Result<Rows<R>,quent::ExecutionError> {let result=session.execute_compiled_read(current,mapping,self.compiled).await?;let decoded=result.sensitive_rows().iter().map(|row|(self.decode)(row)).collect::<Result<Vec<_>,_>>();match decoded {Ok(rows)=>Ok(Rows(rows)),Err(())=>{session.quarantine_generated_decode();Err(quent::ExecutionError::Decode)}} } }
#[cfg(all(feature="runtime-tokio",feature="mysql"))] impl<R> Read<R> { pub async fn execute_mysql(self,session:&mut quent::MysqlSession,current:&core::DatabaseContext,mapping:&TypedModelContract)->Result<Rows<R>,quent::ExecutionError> {let result=session.execute_compiled_read(current,mapping,self.compiled).await?;let decoded=result.sensitive_rows().iter().map(|row|(self.decode)(row)).collect::<Result<Vec<_>,_>>();match decoded {Ok(rows)=>Ok(Rows(rows)),Err(())=>{session.quarantine_generated_decode();Err(quent::ExecutionError::Decode)}} } }
}
