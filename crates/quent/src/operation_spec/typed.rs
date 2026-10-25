//! Application assertions and DB-free compilation through the sole Core planner.
use super::*;
use quent_core::{
    Column, Compare, Dialect, Direction, Draft, Ident, Inspection, Order, PlanView,
    PolicyDeclaration, PolicyPreparation, Predicate, Projection, SqlType, Table,
    Value as NativeValue,
};

/// No inference from a type name, manifest, observation or fingerprint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeFamily {
    Decimal,
    Uuid,
    Date,
    Time,
    Bytes,
    Float,
    Other,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnType {
    Native(SqlType),
    Unsupported(NativeFamily),
    Unobserved,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelCoverage {
    Complete,
    Unknown,
}
/// An authenticated application must supply accurate physical identity and roles.
#[derive(Clone, PartialEq, Eq)]
pub struct TypedColumn {
    symbol: String,
    physical: Ident,
    ty: ColumnType,
    nullable: bool,
    role: ColumnRole,
    read: bool,
}
impl TypedColumn {
    pub fn application_supplied(
        symbol: &str,
        physical: Ident,
        ty: ColumnType,
        nullable: bool,
        role: ColumnRole,
        read: bool,
    ) -> TypedResult<Self> {
        symbol_str(symbol).map_err(|_| TypedError::Mapping)?;
        symbol_str(physical.sensitive_text()).map_err(|_| TypedError::Mapping)?;
        Ok(Self {
            symbol: symbol.into(),
            physical,
            ty,
            nullable,
            role,
            read,
        })
    }
}
/// Immutable, complete exact-content mapping tied to an application snapshot.
/// Construction validates structure, not live schema or application authentication.
#[derive(Clone, PartialEq, Eq)]
pub struct TypedModelContract {
    model: String,
    table: Table,
    columns: Vec<TypedColumn>,
    coverage: ModelCoverage,
    snapshot: DatabaseContext,
}
impl TypedModelContract {
    pub fn application_asserted(
        model: &str,
        table: Table,
        columns: Vec<TypedColumn>,
        coverage: ModelCoverage,
        current: &DatabaseContext,
    ) -> TypedResult<Self> {
        symbol_str(model).map_err(|_| TypedError::Mapping)?;
        if table.sensitive_alias().is_some()
            || table.sensitive_path().len() != 2
            || table
                .sensitive_path()
                .iter()
                .any(|p| symbol_str(p.sensitive_text()).is_err())
            || columns.is_empty()
            || columns.len() > 256
        {
            return Err(TypedError::Mapping);
        }
        let mut symbols = BTreeSet::new();
        let mut physical = BTreeSet::new();
        for c in &columns {
            if !symbols.insert(&c.symbol) || !physical.insert(c.physical.sensitive_text()) {
                return Err(TypedError::Mapping);
            }
            // An ordinary source symbol may never disguise a physical tenant column.
            for p in current.sensitive_source().declarations() {
                if let PolicyDeclaration::Tenant(tenant) = p
                    && tenant.sensitive_table() == &table
                    && tenant.sensitive_name() == &c.physical
                    && (c.role != ColumnRole::Tenant
                        || c.ty != ColumnType::Native(tenant.sql_type())
                        || c.nullable != tenant.nullable())
                {
                    return Err(TypedError::Mapping);
                }
            }
        }
        Ok(Self {
            model: model.into(),
            table,
            columns,
            coverage,
            snapshot: current.clone(),
        })
    }
    fn column(&self, symbol: &str, dialect: Dialect) -> TypedResult<Column> {
        let c = self
            .columns
            .iter()
            .find(|c| c.symbol == symbol)
            .ok_or(TypedError::Mapping)?;
        let ty = match c.ty {
            ColumnType::Native(ty) => ty,
            ColumnType::Unsupported(_) => return Err(TypedError::UnsupportedFamily),
            ColumnType::Unobserved => return Err(TypedError::Unobserved),
        };
        if dialect == Dialect::Postgres
            && matches!(
                ty,
                SqlType::I8 | SqlType::U8 | SqlType::U16 | SqlType::U32 | SqlType::U64
            )
        {
            return Err(TypedError::Dialect);
        }
        Ok(self.table.column(c.physical.clone(), ty, c.nullable))
    }
}
/// Private read-only owned record; no SQL, Operation or handle replacement API.
pub struct CompiledRead {
    source: RoleCheckedSource,
    mapping: TypedModelContract,
    prepared: PolicyPreparation,
}
redacted!(TypedColumn, TypedModelContract, CompiledRead);
impl CompiledRead {
    pub fn inspect(&self) -> Inspection<'_> {
        self.prepared.inspect()
    }
    pub fn public_view(&self) -> PlanView {
        self.prepared.public_view()
    }
    pub fn matches_current(&self, current: &DatabaseContext, mapping: &TypedModelContract) -> bool {
        self.mapping == *mapping && self.source.matches_snapshot(current)
    }
    #[cfg(all(
        feature = "runtime-tokio",
        any(feature = "postgres", feature = "mysql")
    ))]
    pub(crate) fn into_prepared(
        self,
        current: &DatabaseContext,
        mapping: &TypedModelContract,
    ) -> TypedResult<PolicyPreparation> {
        if !self.matches_current(current, mapping) {
            return Err(TypedError::SnapshotMismatch);
        }
        Ok(self.prepared)
    }
}
type TypedResult<T> = std::result::Result<T, TypedError>;
/// Fixed failures independent of source strings and native error messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TypedError {
    Mapping,
    UnknownCoverage,
    Unobserved,
    UnsupportedFamily,
    Dialect,
    Role,
    ReadDenied,
    TenantPredicate,
    SnapshotMismatch,
    TypeMismatch,
    NullComparison,
    NonNullableNull,
    Operator,
    PolicyRefused,
    NativeRefused,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RepairAction {
    CheckApplicationRegistration,
    CheckNativeSupport,
    UseExplicitNullPredicate,
    CheckLiteralType,
    CheckReadPolicy,
    RecompileCurrentSnapshot,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct TypedDiagnostic {
    kind: &'static str,
    version: u8,
    state: &'static str,
    code: TypedError,
    action: RepairAction,
}
impl TypedError {
    pub fn public_view(self) -> TypedDiagnostic {
        let action = match self {
            Self::NullComparison => RepairAction::UseExplicitNullPredicate,
            Self::TypeMismatch | Self::NonNullableNull => RepairAction::CheckLiteralType,
            Self::UnsupportedFamily | Self::Dialect | Self::Operator | Self::NativeRefused => {
                RepairAction::CheckNativeSupport
            }
            Self::ReadDenied | Self::TenantPredicate | Self::PolicyRefused => {
                RepairAction::CheckReadPolicy
            }
            Self::SnapshotMismatch => RepairAction::RecompileCurrentSnapshot,
            _ => RepairAction::CheckApplicationRegistration,
        };
        TypedDiagnostic {
            kind: "quent_typed_spec_diagnostic",
            version: 1,
            state: "refused",
            code: self,
            action,
        }
    }
}
impl TypedDiagnostic {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("fixed typed diagnostic")
    }
}
impl fmt::Display for TypedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_TYPED_SPEC_{self:?}")
    }
}
impl std::error::Error for TypedError {}
impl SpecSource {
    /// DB/provider/catalog/clock-free. Success is type/preparation, never permission.
    pub fn compile(
        self,
        mapping: &TypedModelContract,
        current: &DatabaseContext,
    ) -> TypedResult<CompiledRead> {
        if !mapping.snapshot.same_snapshot(current) {
            return Err(TypedError::SnapshotMismatch);
        }
        if mapping.coverage != ModelCoverage::Complete {
            return Err(TypedError::UnknownCoverage);
        }
        if mapping
            .columns
            .iter()
            .any(|c| c.role == ColumnRole::Unknown)
        {
            return Err(TypedError::Role);
        }
        let dialect = current.sensitive_source().target().3;
        // Never treat unobserved or unsupported registered coverage as typed success.
        let columns = mapping
            .columns
            .iter()
            .map(|c| mapping.column(&c.symbol, dialect))
            .collect::<TypedResult<Vec<_>>>()?;
        if !current
            .sensitive_source()
            .matches_model_columns(&mapping.table, &columns)
        {
            return Err(TypedError::Mapping);
        }
        let roles = ModelContract::application_asserted(
            &mapping.model,
            mapping
                .columns
                .iter()
                .map(|c| (c.symbol.clone(), c.role, c.read))
                .collect(),
            current,
        )
        .map_err(|_| TypedError::Mapping)?;
        let source = self.check_roles(roles, current).map_err(|e| match e {
            SpecError::TenantPredicate => TypedError::TenantPredicate,
            SpecError::ReadDenied => TypedError::ReadDenied,
            _ => TypedError::Role,
        })?;
        let tree = source.sensitive_source();
        let projections = tree["fields"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                Ok(Projection::new(
                    mapping.column(f.as_str().unwrap(), dialect)?,
                    None,
                ))
            })
            .collect::<TypedResult<Vec<_>>>()?;
        let mut draft = Draft::select(mapping.table.clone(), projections);
        if let Some(p) = tree.get("filter") {
            draft = draft.filter(compile_predicate(p, mapping, dialect)?);
        }
        let order = tree["order"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| {
                Ok(Order {
                    column: mapping.column(o["field"].as_str().unwrap(), dialect)?,
                    direction: if o["direction"] == "asc" {
                        Direction::Asc
                    } else {
                        Direction::Desc
                    },
                })
            })
            .collect::<TypedResult<Vec<_>>>()?;
        let operation = draft
            .order(order)
            .limit(tree["limit"].as_u64().unwrap())
            .freeze();
        let prepared = current
            .prepare()
            .prepare_operation(&operation)
            .map_err(|_| TypedError::PolicyRefused)?;
        Ok(CompiledRead {
            source,
            mapping: mapping.clone(),
            prepared,
        })
    }
}
fn compile_predicate(
    p: &Value,
    mapping: &TypedModelContract,
    dialect: Dialect,
) -> TypedResult<Predicate> {
    let op = p["op"].as_str().unwrap();
    match op {
        "and" | "or" => {
            let args = p["args"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| compile_predicate(p, mapping, dialect))
                .collect::<TypedResult<Vec<_>>>()?;
            Ok(if op == "and" {
                Predicate::And(args)
            } else {
                Predicate::Or(args)
            })
        }
        "not" => Ok(Predicate::Not(Box::new(compile_predicate(
            &p["arg"], mapping, dialect,
        )?))),
        _ => {
            let c = mapping.column(p["field"].as_str().unwrap(), dialect)?;
            if op == "is_null" {
                return Ok(Predicate::IsNull(c));
            }
            if op == "in" {
                let values = p["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| native_literal(v, &c))
                    .collect::<TypedResult<Vec<_>>>()?;
                return Ok(Predicate::In(c, values));
            }
            let value = native_literal(&p["value"], &c)?;
            if matches!(value, NativeValue::Null(_)) {
                return Err(TypedError::NullComparison);
            }
            if !matches!(op, "eq" | "ne") && matches!(c.sql_type(), SqlType::Bool | SqlType::Text) {
                return Err(TypedError::Operator);
            }
            let compare = match op {
                "eq" => Compare::Eq,
                "ne" => Compare::Ne,
                "lt" => Compare::Lt,
                "le" => Compare::Le,
                "gt" => Compare::Gt,
                "ge" => Compare::Ge,
                _ => return Err(TypedError::Operator),
            };
            Ok(Predicate::Compare(c, compare, value))
        }
    }
}
fn native_literal(lit: &Value, c: &Column) -> TypedResult<NativeValue> {
    let ty = match lit["type"].as_str().unwrap() {
        "i8" => SqlType::I8,
        "i16" => SqlType::I16,
        "i32" => SqlType::I32,
        "i64" => SqlType::I64,
        "u8" => SqlType::U8,
        "u16" => SqlType::U16,
        "u32" => SqlType::U32,
        "u64" => SqlType::U64,
        "bool" => SqlType::Bool,
        "text" => SqlType::Text,
        _ => return Err(TypedError::UnsupportedFamily),
    };
    if ty != c.sql_type() {
        return Err(TypedError::TypeMismatch);
    }
    let v = &lit["value"];
    if v.is_null() {
        if !c.nullable() {
            return Err(TypedError::NonNullableNull);
        }
        return Ok(NativeValue::Null(ty));
    }
    macro_rules! checked {
        ($variant:ident, $get:ident) => {
            NativeValue::$variant(
                v.$get()
                    .ok_or(TypedError::TypeMismatch)?
                    .try_into()
                    .map_err(|_| TypedError::TypeMismatch)?,
            )
        };
    }
    Ok(match ty {
        SqlType::I8 => checked!(I8, as_i64),
        SqlType::I16 => checked!(I16, as_i64),
        SqlType::I32 => checked!(I32, as_i64),
        SqlType::I64 => checked!(I64, as_i64),
        SqlType::U8 => checked!(U8, as_u64),
        SqlType::U16 => checked!(U16, as_u64),
        SqlType::U32 => checked!(U32, as_u64),
        SqlType::U64 => checked!(U64, as_u64),
        SqlType::Bool => NativeValue::Bool(v.as_bool().ok_or(TypedError::TypeMismatch)?),
        SqlType::Text => {
            let s = v.as_str().ok_or(TypedError::TypeMismatch)?;
            if s.contains('\0') {
                return Err(TypedError::NativeRefused);
            }
            NativeValue::Text(s.into())
        }
    })
}
