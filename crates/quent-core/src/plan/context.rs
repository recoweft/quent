//! Application-only assertions. Nothing here authenticates a caller or a database.
use super::*;

/// Fixed failures, without caller text or source errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextError {
    Identity,
    Generation,
    Binding,
    Policy,
    Type,
    Unsupported,
    Budget,
}
impl fmt::Display for ContextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_CONTEXT_{self:?}")
    }
}
impl std::error::Error for ContextError {}
type ContextResult<T> = std::result::Result<T, ContextError>;

macro_rules! identities {
    ($($name:ident),+) => {$ (
        /// Owned exact application identity, not a digest or external verification.
        #[derive(Clone, PartialEq, Eq)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> ContextResult<Self> {
                let value = value.into();
                if value.is_empty() || value.len() > 128 || !value.bytes().all(|b|
                    b.is_ascii_alphanumeric() || b"_-.:/".contains(&b)) {
                    return Err(ContextError::Identity);
                }
                Ok(Self(value))
            }
        }
        redacted!($name);
    )+};
}
identities!(
    DatabaseIdentity,
    TargetAddress,
    SubjectIdentity,
    SchemaIdentity,
    PolicyIdentity,
    SettingsIdentity
);

/// Explicit application generation. No clock, freshness or global sequence.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Generation(u64);
impl Generation {
    pub fn new(value: u64) -> ContextResult<Self> {
        if value == 0 {
            return Err(ContextError::Generation);
        }
        Ok(Self(value))
    }
}
/// Exact configured target tuple. Address is a nonsecret application routing key,
/// never a credential/DSN or proof of the physical endpoint behind that key.
#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseBinding {
    database: DatabaseIdentity,
    address: TargetAddress,
    catalog: Ident,
    dialect: Dialect,
}
impl DatabaseBinding {
    pub fn application_supplied(
        database: DatabaseIdentity,
        address: TargetAddress,
        catalog: Ident,
        dialect: Dialect,
    ) -> Self {
        Self {
            database,
            address,
            catalog,
            dialect,
        }
    }
}
/// Strict is the only supported setting; no permissive fallback exists.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SettingsMode {
    #[default]
    Strict,
}

/// Finite declarations, enforced only by PreparationContext::prepare_operation.
#[derive(Clone, PartialEq, Eq)]
pub enum PolicyDeclaration {
    Tenant(Column),
    SoftDelete(Column),
    Pii(Column),
    RequiredFilter(Column),
    Readonly(Column),
    Generated(Column),
}
impl PolicyDeclaration {
    fn column(&self) -> &Column {
        match self {
            Self::Tenant(c)
            | Self::SoftDelete(c)
            | Self::Pii(c)
            | Self::RequiredFilter(c)
            | Self::Readonly(c)
            | Self::Generated(c) => c,
        }
    }
}
fn supported_column(binding: &DatabaseBinding, column: &Column) -> ContextResult<()> {
    if column.table.alias.is_some() {
        return Err(ContextError::Unsupported);
    }
    if binding.dialect == Dialect::Postgres
        && matches!(
            column.sql_type,
            SqlType::I8 | SqlType::U8 | SqlType::U16 | SqlType::U32 | SqlType::U64
        )
    {
        return Err(ContextError::Unsupported);
    }
    Ok(())
}
fn signed_column(binding: &DatabaseBinding, column: &Column) -> ContextResult<()> {
    supported_column(binding, column)?;
    if column.nullable
        || !matches!(
            column.sql_type,
            SqlType::I8 | SqlType::I16 | SqlType::I32 | SqlType::I64
        )
    {
        return Err(ContextError::Unsupported);
    }
    Ok(())
}
/// Bounded, owned declaration snapshot. Empty means no declarations, never allow.
#[derive(Clone, PartialEq, Eq)]
pub struct PolicySet {
    binding: DatabaseBinding,
    identity: PolicyIdentity,
    generation: Generation,
    declarations: Vec<PolicyDeclaration>,
}
impl PolicySet {
    pub fn new(
        binding: DatabaseBinding,
        identity: PolicyIdentity,
        generation: Generation,
        declarations: Vec<PolicyDeclaration>,
    ) -> ContextResult<Self> {
        if declarations.len() > 256 {
            return Err(ContextError::Budget);
        }
        for (i, declaration) in declarations.iter().enumerate() {
            let c = declaration.column();
            supported_column(&binding, c)?;
            match declaration {
                PolicyDeclaration::Tenant(_) => signed_column(&binding, c)?,
                PolicyDeclaration::SoftDelete(_) if c.nullable || c.sql_type != SqlType::Bool => {
                    return Err(ContextError::Unsupported);
                }
                _ => {}
            }
            for previous in &declarations[..i] {
                let p = previous.column();
                // One declaration per exact column; ambiguous type/alias declarations
                // and multiple tenant columns for the same table are also refused.
                if p.table.path == c.table.path
                    && (p.name == c.name
                        || matches!(
                            (previous, declaration),
                            (PolicyDeclaration::Tenant(_), PolicyDeclaration::Tenant(_))
                        ))
                {
                    return Err(ContextError::Policy);
                }
            }
        }
        Ok(Self {
            binding,
            identity,
            generation,
            declarations,
        })
    }
}
/// Owned immutable configuration, Clone duplicates configuration, not a permit.
/// ```compile_fail
/// fn mutate(s: &mut quent_core::Settings) { s.mode = quent_core::SettingsMode::Strict; }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct Settings {
    binding: DatabaseBinding,
    identity: SettingsIdentity,
    generation: Generation,
    policies: PolicySet,
    mode: SettingsMode,
}
impl Settings {
    pub fn new(
        binding: DatabaseBinding,
        identity: SettingsIdentity,
        generation: Generation,
        policies: PolicySet,
    ) -> ContextResult<Self> {
        if binding != policies.binding {
            return Err(ContextError::Binding);
        }
        Ok(Self {
            binding,
            identity,
            generation,
            policies,
            mode: SettingsMode::Strict,
        })
    }
    pub fn mode(&self) -> SettingsMode {
        self.mode
    }
    /// Same settings identity/target; strictly newer settings and policy generations.
    /// A new snapshot does not update any existing context.
    pub fn next(&self, generation: Generation, policies: PolicySet) -> ContextResult<Self> {
        if generation <= self.generation
            || policies.identity != self.policies.identity
            || policies.generation <= self.policies.generation
        {
            return Err(ContextError::Generation);
        }
        Self::new(
            self.binding.clone(),
            self.identity.clone(),
            generation,
            policies,
        )
    }
}
/// Application assertion only, without a live schema/catalog or ScopeFacts upgrade.
#[derive(Clone, PartialEq, Eq)]
pub struct SchemaSnapshot {
    binding: DatabaseBinding,
    identity: SchemaIdentity,
    generation: Generation,
    facts: Vec<ScopeFacts>,
}
impl SchemaSnapshot {
    pub fn application_supplied(
        binding: DatabaseBinding,
        identity: SchemaIdentity,
        generation: Generation,
        facts: Vec<ScopeFacts>,
    ) -> ContextResult<Self> {
        if facts.len() > 16 {
            return Err(ContextError::Budget);
        }
        for (i, fact) in facts.iter().enumerate() {
            if fact.dialect != binding.dialect || facts[..i].iter().any(|p| p.table == fact.table) {
                return Err(ContextError::Binding);
            }
        }
        Ok(Self {
            binding,
            identity,
            generation,
            facts,
        })
    }
}
/// Exact non-NULL signed tenant value plus declared column/type and target.
#[derive(Clone, PartialEq, Eq)]
pub struct TenantSnapshot {
    binding: DatabaseBinding,
    column: Column,
    value: Value,
}
impl TenantSnapshot {
    pub fn application_supplied(
        binding: DatabaseBinding,
        column: Column,
        value: Value,
    ) -> ContextResult<Self> {
        signed_column(&binding, &column)?;
        if column.sql_type != value.sql_type() {
            return Err(ContextError::Type);
        }
        if !matches!(
            value,
            Value::I8(_) | Value::I16(_) | Value::I32(_) | Value::I64(_)
        ) {
            return Err(ContextError::Unsupported);
        }
        Ok(Self {
            binding,
            column,
            value,
        })
    }
}
/// Supplied by authenticated/authorized server code. Constructors do not perform
/// authentication. No serde, FromStr, JSON, reason, freshness or allow input.
/// ```compile_fail
/// let c: quent_core::ApplicationContext = "{\"tenant\":1,\"allow\":true}".parse().unwrap();
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct ApplicationContext {
    binding: DatabaseBinding,
    schema: Option<SchemaSnapshot>,
    subject: Option<SubjectIdentity>,
    tenant: Option<TenantSnapshot>,
}
impl ApplicationContext {
    pub fn application_supplied(
        binding: DatabaseBinding,
        schema: Option<SchemaSnapshot>,
        subject: Option<SubjectIdentity>,
        tenant: Option<TenantSnapshot>,
    ) -> ContextResult<Self> {
        if schema.as_ref().is_some_and(|s| s.binding != binding)
            || tenant.as_ref().is_some_and(|t| t.binding != binding)
        {
            return Err(ContextError::Binding);
        }
        Ok(Self {
            binding,
            schema,
            subject,
            tenant,
        })
    }
}
/// Presence is only a structural observation, never authentication/permission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextPresence {
    pub schema: bool,
    pub subject: bool,
    pub tenant: bool,
}
/// DB-free carrier, not a connection, executor or transaction.
/// ```compile_fail
/// fn restore(c: quent_core::DatabaseContext) {
///     let restored: quent_core::DatabaseContext = format!("{c:?}").parse().unwrap();
/// }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct DatabaseContext {
    settings: Settings,
    application: ApplicationContext,
}
impl DatabaseContext {
    pub fn new(settings: Settings, application: ApplicationContext) -> ContextResult<Self> {
        if settings.binding != application.binding {
            return Err(ContextError::Binding);
        }
        // When supplied, the tenant must match every declared tenant column exactly.
        // Missing tenant stays missing; declarations are never silently enforced.
        if let Some(tenant) = &application.tenant {
            for p in &settings.policies.declarations {
                if let PolicyDeclaration::Tenant(column) = p
                    && column != &tenant.column
                {
                    return Err(ContextError::Policy);
                }
            }
        }
        if let Some(schema) = &application.schema {
            for fact in &schema.facts {
                for column in settings
                    .policies
                    .declarations
                    .iter()
                    .map(PolicyDeclaration::column)
                    .chain(application.tenant.iter().map(|t| &t.column))
                {
                    if fact.table == column.table && !fact.columns.contains(column) {
                        return Err(ContextError::Type);
                    }
                }
            }
        }
        Ok(Self {
            settings,
            application,
        })
    }
    pub fn presence(&self) -> ContextPresence {
        ContextPresence {
            schema: self.application.schema.is_some(),
            subject: self.application.subject.is_some(),
            tenant: self.application.tenant.is_some(),
        }
    }
    /// Explicit replacement of schema/subject/tenant, preserving the exact target.
    pub fn with_application(&self, application: ApplicationContext) -> ContextResult<Self> {
        Self::new(self.settings.clone(), application)
    }
    /// Explicit new settings generation; existing and sibling snapshots remain owned.
    pub fn with_settings(&self, settings: Settings) -> ContextResult<Self> {
        if settings.identity != self.settings.identity
            || settings.generation <= self.settings.generation
            || settings.binding != self.settings.binding
            || settings.policies.identity != self.settings.policies.identity
            || settings.policies.generation <= self.settings.policies.generation
        {
            return Err(ContextError::Generation);
        }
        Self::new(settings, self.application.clone())
    }
    pub fn prepare(&self) -> PreparationContext {
        PreparationContext {
            snapshot: self.clone(),
        }
    }
    /// Exact content comparison, not a current/live/physical/authentication check.
    pub fn same_snapshot(&self, other: &Self) -> bool {
        self == other
    }
}
/// A snapshot for future preparation work. No begin/commit/rollback/dispatch API.
/// ```compile_fail
/// fn transaction(c: quent_core::PreparationContext) { c.commit(); }
/// ```
#[derive(Clone, PartialEq, Eq)]
pub struct PreparationContext {
    snapshot: DatabaseContext,
}
impl PreparationContext {
    pub fn snapshot(&self) -> &DatabaseContext {
        &self.snapshot
    }
    pub fn prepare(&self) -> Self {
        self.clone()
    }
}
redacted!(
    Generation,
    DatabaseBinding,
    PolicyDeclaration,
    PolicySet,
    Settings,
    SchemaSnapshot,
    TenantSnapshot,
    ApplicationContext,
    DatabaseContext,
    PreparationContext
);

/// Explicit local sensitive inspection, like Plan::sensitive_source. Never log or
/// serialize its getter results into public summaries. It has no restoration API.
pub struct SensitiveContextSource<'a> {
    snapshot: &'a DatabaseContext,
}
impl DatabaseContext {
    pub fn sensitive_source(&self) -> SensitiveContextSource<'_> {
        SensitiveContextSource { snapshot: self }
    }
}
impl SensitiveContextSource<'_> {
    pub fn target(&self) -> (&str, &str, &Ident, Dialect) {
        let b = &self.snapshot.settings.binding;
        (&b.database.0, &b.address.0, &b.catalog, b.dialect)
    }
    pub fn settings_version(&self) -> (&str, u64) {
        let s = &self.snapshot.settings;
        (&s.identity.0, s.generation.0)
    }
    pub fn policy_version(&self) -> (&str, u64) {
        let p = &self.snapshot.settings.policies;
        (&p.identity.0, p.generation.0)
    }
    pub fn declarations(&self) -> &[PolicyDeclaration] {
        &self.snapshot.settings.policies.declarations
    }
    pub fn schema(&self) -> Option<(&str, u64, &[ScopeFacts])> {
        self.snapshot
            .application
            .schema
            .as_ref()
            .map(|s| (s.identity.0.as_str(), s.generation.0, s.facts.as_slice()))
    }
    pub fn subject(&self) -> Option<&str> {
        self.snapshot
            .application
            .subject
            .as_ref()
            .map(|s| s.0.as_str())
    }
    pub fn tenant(&self) -> Option<(&Column, &Value)> {
        self.snapshot
            .application
            .tenant
            .as_ref()
            .map(|t| (&t.column, &t.value))
    }
}
redacted!(SensitiveContextSource<'_>);

mod policy;
pub use policy::*;
