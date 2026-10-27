//! Offline schema differences. Application declarations are never live evidence.
//! No executor, provider, preflight, apply, or public-wire restoration exists.
use crate::{Dialect, Direction, Ident, PlanError, Risk, SqlType, Value};
use std::{collections::BTreeSet, fmt};
type Result<T> = std::result::Result<T, PlanError>;
macro_rules! redacted { ($($t:ty),+ $(,)?) => {$ (impl fmt::Debug for $t {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(concat!(stringify!($t), "([REDACTED])"))
    }
})+}; }
/// Coverage is a declaration about the supplied finite source, never a catalog proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Knowledge {
    Unknown,
    Partial,
    Complete,
}
/// Unknown is retained even when two unobserved values have identical representation.
#[derive(Clone, PartialEq, Eq)]
pub enum Observed<T> {
    Unknown,
    Known(T),
}
impl<T> fmt::Debug for Observed<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Observed([REDACTED])")
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaAuthority {
    UnverifiedSource,
    ApplicationDeclared,
}
#[derive(Clone, PartialEq, Eq)]
pub struct MigrationTarget {
    database: String,
    address: String,
    catalog: Ident,
    dialect: Dialect,
}
impl MigrationTarget {
    pub fn new(
        database: String,
        address: String,
        catalog: Ident,
        dialect: Dialect,
    ) -> Result<Self> {
        if [database.as_str(), address.as_str()]
            .iter()
            .any(|s| s.is_empty() || s.len() > 128 || s.chars().any(char::is_control))
        {
            return Err(PlanError::Shape);
        }
        Ok(Self {
            database,
            address,
            catalog,
            dialect,
        })
    }
}
/// Explicit namespace/table; no search path, aliases, case folding or rename inference.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MigrationName {
    namespace: Ident,
    table: Ident,
}
impl MigrationName {
    pub fn sensitive_parts(&self) -> (&Ident, &Ident) {
        (&self.namespace, &self.table)
    }
    pub fn new(namespace: Ident, table: Ident) -> Self {
        Self { namespace, table }
    }
}
/// Opaque syntax can be compared byte-for-byte but never establishes equivalence.
#[derive(Clone, PartialEq, Eq)]
pub struct OpaqueDefinition(String);
impl OpaqueDefinition {
    pub fn new(text: String) -> Result<Self> {
        if text.len() > 4096 {
            return Err(PlanError::Budget);
        }
        Ok(Self(text))
    }
}
#[derive(Clone, PartialEq, Eq)]
pub enum MigrationDefault {
    Unobserved,
    Unknown,
    Absent,
    Literal(Value),
    Opaque(OpaqueDefinition),
}
#[derive(Clone, PartialEq, Eq)]
pub struct MigrationColumn {
    name: Ident,
    native: Observed<SqlType>,
    nullable: Observed<bool>,
    default: MigrationDefault,
}
impl MigrationColumn {
    pub fn new(
        name: Ident,
        native: Observed<SqlType>,
        nullable: Observed<bool>,
        default: MigrationDefault,
    ) -> Result<Self> {
        if let MigrationDefault::Literal(value) = &default {
            if let Observed::Known(ty) = native
                && value.sql_type() != ty
            {
                return Err(PlanError::Type);
            }
            if let Value::Text(text) = value {
                if text.len() > 4096 {
                    return Err(PlanError::Budget);
                }
                if text.contains('\0') {
                    return Err(PlanError::Type);
                }
            }
        }
        Ok(Self {
            name,
            native,
            nullable,
            default,
        })
    }
}
/// Plain means the application explicitly declares no expression/predicate/include/
/// prefix/collation/opclass/invisibility or other unrepresented index attributes.
#[derive(Clone, PartialEq, Eq)]
pub struct MigrationIndex {
    name: Ident,
    definition: Observed<IndexDefinition>,
}
#[derive(Clone, PartialEq, Eq)]
pub struct IndexDefinition {
    kind: IndexKind,
    columns: Vec<(Ident, Direction)>,
    opaque: Option<OpaqueDefinition>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    NonUnique,
    Unique,
    Primary,
}
impl IndexDefinition {
    /// Unknown expression/partial/other attributes, preserved solely for difference detection.
    pub fn opaque(definition: OpaqueDefinition) -> Self {
        Self {
            kind: IndexKind::NonUnique,
            columns: vec![],
            opaque: Some(definition),
        }
    }
    pub fn plain_btree(kind: IndexKind, columns: Vec<(Ident, Direction)>) -> Result<Self> {
        components(&columns.iter().map(|c| c.0.clone()).collect::<Vec<_>>())?;
        Ok(Self {
            kind,
            columns,
            opaque: None,
        })
    }
}
impl MigrationIndex {
    pub fn new(name: Ident, definition: Observed<IndexDefinition>) -> Self {
        Self { name, definition }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferentialAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}
/// Finite MATCH SIMPLE, nondeferrable, enforced FK; other properties are Unknown.
#[derive(Clone, PartialEq, Eq)]
pub struct ForeignKeyDefinition {
    columns: Vec<Ident>,
    referenced: MigrationName,
    referenced_columns: Vec<Ident>,
    on_delete: ReferentialAction,
    on_update: ReferentialAction,
}
impl ForeignKeyDefinition {
    pub fn simple(
        columns: Vec<Ident>,
        referenced: MigrationName,
        referenced_columns: Vec<Ident>,
        on_delete: ReferentialAction,
        on_update: ReferentialAction,
    ) -> Result<Self> {
        components(&columns)?;
        components(&referenced_columns)?;
        if columns.len() != referenced_columns.len() {
            return Err(PlanError::Shape);
        }
        Ok(Self {
            columns,
            referenced,
            referenced_columns,
            on_delete,
            on_update,
        })
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct MigrationForeignKey {
    name: Ident,
    definition: Observed<ForeignKeyDefinition>,
}
impl MigrationForeignKey {
    pub fn new(name: Ident, definition: Observed<ForeignKeyDefinition>) -> Self {
        Self { name, definition }
    }
}
/// CHECK syntax is deliberately opaque in PR1; even identical bytes stay unknown.
#[derive(Clone, PartialEq, Eq)]
pub struct MigrationCheck {
    name: Ident,
    definition: Observed<OpaqueDefinition>,
}
impl MigrationCheck {
    pub fn new(name: Ident, definition: Observed<OpaqueDefinition>) -> Self {
        Self { name, definition }
    }
}
/// Complete describes enumeration, independently from whether definitions are known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableCoverage {
    pub columns: Knowledge,
    pub indexes: Knowledge,
    pub foreign_keys: Knowledge,
    pub checks: Knowledge,
}
/// Ordinary declares base-table semantics and absence of triggers, RLS, generated/
/// identity columns, partitioning, inheritance, collation and unrepresented effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableEffects {
    Unknown,
    Ordinary,
    Unsupported,
}
#[derive(Clone, PartialEq, Eq)]
pub struct MigrationTable {
    name: MigrationName,
    effects: TableEffects,
    coverage: TableCoverage,
    columns: Vec<MigrationColumn>,
    indexes: Vec<MigrationIndex>,
    foreign_keys: Vec<MigrationForeignKey>,
    checks: Vec<MigrationCheck>,
}
impl MigrationTable {
    pub fn new(
        name: MigrationName,
        effects: TableEffects,
        coverage: TableCoverage,
        columns: Vec<MigrationColumn>,
        indexes: Vec<MigrationIndex>,
        foreign_keys: Vec<MigrationForeignKey>,
        checks: Vec<MigrationCheck>,
    ) -> Result<Self> {
        if columns.len() > 256 || indexes.len() > 16 || foreign_keys.len() > 16 || checks.len() > 16
        {
            return Err(PlanError::Budget);
        }
        if coverage.columns == Knowledge::Complete && columns.is_empty() {
            return Err(PlanError::Shape);
        }
        unique(columns.iter().map(|c| &c.name))?;
        unique(indexes.iter().map(|c| &c.name))?;
        unique(foreign_keys.iter().map(|c| &c.name))?;
        unique(checks.iter().map(|c| &c.name))?;
        let exists = |id: &Ident| columns.iter().any(|c| c.name == *id);
        for i in &indexes {
            if let Observed::Known(d) = &i.definition
                && d.columns.iter().any(|c| !exists(&c.0))
            {
                return Err(PlanError::Shape);
            }
        }
        for fk in &foreign_keys {
            if let Observed::Known(d) = &fk.definition
                && d.columns.iter().any(|c| !exists(c))
            {
                return Err(PlanError::Shape);
            }
        }
        Ok(Self {
            name,
            effects,
            coverage,
            columns,
            indexes,
            foreign_keys,
            checks,
        })
    }
}
/// Owned immutable declarations. Array order is significant; comparison never sorts
/// away a change. Limits bound retained work, not caller allocations before new().
#[derive(Clone, PartialEq, Eq)]
pub struct MigrationSchema {
    target: MigrationTarget,
    authority: SchemaAuthority,
    coverage: Knowledge,
    tables: Vec<MigrationTable>,
}
impl MigrationSchema {
    pub fn new(
        target: MigrationTarget,
        authority: SchemaAuthority,
        coverage: Knowledge,
        tables: Vec<MigrationTable>,
    ) -> Result<Self> {
        if tables.len() > 16 {
            return Err(PlanError::Budget);
        }
        unique(tables.iter().map(|t| &t.name))?;
        if target.dialect == Dialect::Mysql {
            let mut names = BTreeSet::new();
            for t in &tables {
                if !names.insert((
                    t.name.namespace.sensitive_text().to_ascii_lowercase(),
                    t.name.table.sensitive_text().to_ascii_lowercase(),
                )) {
                    return Err(PlanError::Shape);
                }
                for ids in [
                    t.columns.iter().map(|c| &c.name).collect::<Vec<_>>(),
                    t.indexes.iter().map(|c| &c.name).collect(),
                    t.foreign_keys.iter().map(|c| &c.name).collect(),
                    t.checks.iter().map(|c| &c.name).collect(),
                ] {
                    let mut seen = BTreeSet::new();
                    for id in ids {
                        if !seen.insert(id.sensitive_text().to_ascii_lowercase()) {
                            return Err(PlanError::Shape);
                        }
                    }
                }
            }
        }
        for table in &tables {
            if target.dialect == Dialect::Mysql && table.name.namespace != target.catalog {
                return Err(PlanError::Shape);
            }
            for fk in &table.foreign_keys {
                if let Observed::Known(d) = &fk.definition {
                    // References outside the supplied scope require Unknown definitions.
                    let parent = tables
                        .iter()
                        .find(|t| t.name == d.referenced)
                        .ok_or(PlanError::Shape)?;
                    if d.referenced_columns
                        .iter()
                        .any(|c| !parent.columns.iter().any(|p| p.name == *c))
                    {
                        return Err(PlanError::Shape);
                    }
                }
            }
        }
        Ok(Self {
            target,
            authority,
            coverage,
            tables,
        })
    }
}
fn unique<'a, T: Ord + 'a>(items: impl Iterator<Item = &'a T>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for item in items {
        if !seen.insert(item) {
            return Err(PlanError::Shape);
        }
    }
    Ok(())
}
fn components(columns: &[Ident]) -> Result<()> {
    if columns.len() > 16 {
        return Err(PlanError::Budget);
    }
    if columns.is_empty() {
        return Err(PlanError::Shape);
    }
    unique(columns.iter())
}
/// Each area reports detected differences independently from unknown coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationArea {
    Target,
    Tables,
    Effects,
    Columns,
    Types,
    Nullability,
    Defaults,
    Indexes,
    ForeignKeys,
    Checks,
}
const AREAS: [MigrationArea; 10] = [
    MigrationArea::Target,
    MigrationArea::Tables,
    MigrationArea::Effects,
    MigrationArea::Columns,
    MigrationArea::Types,
    MigrationArea::Nullability,
    MigrationArea::Defaults,
    MigrationArea::Indexes,
    MigrationArea::ForeignKeys,
    MigrationArea::Checks,
];
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationSupport {
    Supported,
    Unsupported,
    Unknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DifferenceKind {
    Added,
    Removed,
    Definition,
    Order,
    Authority,
    Coverage,
}
#[derive(Clone, PartialEq, Eq)]
struct Difference {
    area: MigrationArea,
    kind: DifferenceKind,
    support: MigrationSupport,
    table: Option<MigrationName>,
    name: Option<Ident>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AreaView {
    area: MigrationArea,
    changes: usize,
    unknown: bool,
    unsupported: bool,
}
impl AreaView {
    pub fn area(&self) -> MigrationArea {
        self.area
    }
    pub fn changes(&self) -> usize {
        self.changes
    }
    pub fn unknown(&self) -> bool {
        self.unknown
    }
    pub fn unsupported(&self) -> bool {
        self.unsupported
    }
}
/// SqlPlanned means a complete finite offline SQL sequence, never executable authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationStatus {
    NoChange,
    SqlPlanned,
    Refused,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DdlTransaction {
    PostgresTransactional,
    MysqlImplicitCommit,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationView {
    dialect: Dialect,
    status: MigrationStatus,
    risk: Risk,
    areas: [AreaView; 10],
    steps: usize,
    authority: [SchemaAuthority; 2],
}
impl MigrationView {
    pub fn status(&self) -> MigrationStatus {
        self.status
    }
    pub fn risk(&self) -> Risk {
        self.risk
    }
    pub fn areas(&self) -> &[AreaView; 10] {
        &self.areas
    }
    pub fn steps(&self) -> usize {
        self.steps
    }
    pub fn ddl_transaction(&self) -> DdlTransaction {
        if self.dialect == Dialect::Postgres {
            DdlTransaction::PostgresTransactional
        } else {
            DdlTransaction::MysqlImplicitCommit
        }
    }
    /// Output-only fixed codes/counts; no names, SQL, values, types, expressions or error causes.
    pub fn to_json(&self) -> String {
        let mut out = format!(
            "{{\"kind\":\"quent_migration_view\",\"version\":1,\"dialect\":\"{:?}\",\"status\":\"{:?}\",\"risk\":\"{:?}\",\"ddl_transaction\":\"{:?}\",\"authority\":[\"{:?}\",\"{:?}\"],\"live\":\"Unverified\",\"steps\":{},\"areas\":[",
            self.dialect,
            self.status,
            self.risk,
            self.ddl_transaction(),
            self.authority[0],
            self.authority[1],
            self.steps
        );
        for (i, a) in self.areas.iter().enumerate() {
            if i != 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"area\":\"{:?}\",\"changes\":{},\"unknown\":{},\"unsupported\":{}}}",
                a.area, a.changes, a.unknown, a.unsupported
            ));
        }
        out.push_str("]}");
        out
    }
}
impl fmt::Display for MigrationView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
/// Private immutable record with exact input snapshots and final SQL. There is no
/// Serialize/Deserialize, conversion from a view, or CRUD/execution handle.
/// ```compile_fail
/// let plan = quent_core::migration::MigrationPlan { };
/// ```
/// ```compile_fail
/// fn restore(view: quent_core::migration::MigrationView) -> quent_core::migration::MigrationPlan { view.into() }
/// ```
pub struct MigrationPlan {
    before: MigrationSchema,
    after: MigrationSchema,
    differences: Vec<Difference>,
    sql: Vec<String>,
    view: MigrationView,
}
pub struct SensitiveMigration<'a> {
    plan: &'a MigrationPlan,
}
impl MigrationPlan {
    pub fn public_view(&self) -> MigrationView {
        self.view
    }
    pub fn sensitive_source(&self) -> SensitiveMigration<'_> {
        SensitiveMigration { plan: self }
    }
}
impl SensitiveMigration<'_> {
    /// Empty for Refused and NoChange; no partial supported SQL can escape.
    pub fn sql(&self) -> &[String] {
        &self.plan.sql
    }
    pub fn schemas(&self) -> (&MigrationSchema, &MigrationSchema) {
        (&self.plan.before, &self.plan.after)
    }
    pub fn differences(
        &self,
    ) -> impl Iterator<
        Item = (
            MigrationArea,
            DifferenceKind,
            MigrationSupport,
            Option<&MigrationName>,
            Option<&Ident>,
        ),
    > {
        self.plan
            .differences
            .iter()
            .map(|d| (d.area, d.kind, d.support, d.table.as_ref(), d.name.as_ref()))
    }
}
redacted!(
    MigrationTarget,
    MigrationName,
    OpaqueDefinition,
    MigrationDefault,
    MigrationColumn,
    MigrationIndex,
    IndexDefinition,
    ForeignKeyDefinition,
    MigrationForeignKey,
    MigrationCheck,
    MigrationTable,
    MigrationSchema,
    Difference,
    MigrationPlan,
    SensitiveMigration<'_>
);

struct Planner {
    differences: Vec<Difference>,
    areas: [AreaView; 10],
    sql: Vec<String>,
}
impl Planner {
    fn area(&mut self, area: MigrationArea) -> &mut AreaView {
        self.areas
            .iter_mut()
            .find(|a| a.area == area)
            .expect("fixed area")
    }
    fn unknown(&mut self, area: MigrationArea) {
        self.area(area).unknown = true;
    }
    fn unsupported(&mut self, area: MigrationArea) {
        self.area(area).unsupported = true;
    }
    fn change(
        &mut self,
        area: MigrationArea,
        kind: DifferenceKind,
        support: MigrationSupport,
        table: Option<&MigrationName>,
        name: Option<&Ident>,
    ) {
        let a = self.area(area);
        a.changes += 1;
        match support {
            MigrationSupport::Unknown => a.unknown = true,
            MigrationSupport::Unsupported => a.unsupported = true,
            MigrationSupport::Supported => {}
        }
        self.differences.push(Difference {
            area,
            kind,
            support,
            table: table.cloned(),
            name: name.cloned(),
        });
    }
    fn facts(&mut self, schema: &MigrationSchema) {
        if schema.coverage != Knowledge::Complete {
            for area in AREAS {
                self.unknown(area);
            }
        }
        for t in &schema.tables {
            match t.effects {
                TableEffects::Unknown => self.unknown(MigrationArea::Effects),
                TableEffects::Unsupported => self.unsupported(MigrationArea::Effects),
                TableEffects::Ordinary => {}
            }
            for (coverage, areas) in [
                (
                    t.coverage.columns,
                    &[
                        MigrationArea::Columns,
                        MigrationArea::Types,
                        MigrationArea::Nullability,
                        MigrationArea::Defaults,
                    ][..],
                ),
                (t.coverage.indexes, &[MigrationArea::Indexes][..]),
                (t.coverage.foreign_keys, &[MigrationArea::ForeignKeys][..]),
                (t.coverage.checks, &[MigrationArea::Checks][..]),
            ] {
                if coverage != Knowledge::Complete {
                    for &area in areas {
                        self.unknown(area);
                    }
                }
            }
            for c in &t.columns {
                match c.native {
                    Observed::Unknown => self.unknown(MigrationArea::Types),
                    Observed::Known(ty) => {
                        if !native_supported(schema.target.dialect, ty) {
                            self.unsupported(MigrationArea::Types);
                        }
                    }
                }
                if c.nullable == Observed::Unknown {
                    self.unknown(MigrationArea::Nullability);
                }
                match c.default {
                    MigrationDefault::Unknown | MigrationDefault::Unobserved => {
                        self.unknown(MigrationArea::Defaults)
                    }
                    MigrationDefault::Opaque(_) => self.unknown(MigrationArea::Defaults),
                    _ => {}
                }
            }
            if t.indexes.iter().any(|i| {
                matches!(
                    &i.definition,
                    Observed::Unknown
                        | Observed::Known(IndexDefinition {
                            opaque: Some(_),
                            ..
                        })
                )
            }) {
                self.unknown(MigrationArea::Indexes);
            }
            if t.foreign_keys
                .iter()
                .any(|i| i.definition == Observed::Unknown)
            {
                self.unknown(MigrationArea::ForeignKeys);
            }
            // No trusted expression equivalence or CHECK semantics exists in PR1.
            if !t.checks.is_empty() {
                self.unknown(MigrationArea::Checks);
            }
        }
    }
    fn table(&mut self, before: &MigrationTable, after: &MigrationTable, dialect: Dialect) {
        use DifferenceKind::*;
        use MigrationArea::*;
        use MigrationSupport::*;
        let table = Some(&before.name);
        for (area, b, a) in [
            (Columns, before.coverage.columns, after.coverage.columns),
            (Indexes, before.coverage.indexes, after.coverage.indexes),
            (
                ForeignKeys,
                before.coverage.foreign_keys,
                after.coverage.foreign_keys,
            ),
            (Checks, before.coverage.checks, after.coverage.checks),
        ] {
            if b != a {
                self.change(area, Coverage, Unknown, table, None);
            }
        }
        if before.effects != after.effects {
            self.change(Effects, Definition, Unsupported, table, None);
        }
        let old_order: Vec<_> = before
            .columns
            .iter()
            .filter(|c| after.columns.iter().any(|a| a.name == c.name))
            .map(|c| &c.name)
            .collect();
        let new_order: Vec<_> = after
            .columns
            .iter()
            .filter(|c| before.columns.iter().any(|b| b.name == c.name))
            .map(|c| &c.name)
            .collect();
        if old_order != new_order {
            self.change(Columns, Order, Unsupported, table, None);
        }
        for c in &before.columns {
            if let Some(a) = after.columns.iter().find(|a| a.name == c.name) {
                for (area, differs) in [
                    (Types, c.native != a.native),
                    (Nullability, c.nullable != a.nullable),
                    (Defaults, c.default != a.default),
                ] {
                    if differs {
                        self.change(area, Definition, Unsupported, table, Some(&c.name));
                    }
                }
            } else {
                self.change(Columns, Removed, Unsupported, table, Some(&c.name));
            }
        }
        for (position, c) in after.columns.iter().enumerate() {
            if before.columns.iter().any(|b| b.name == c.name) {
                continue;
            }
            let signed = matches!(
                c.native,
                Observed::Known(SqlType::I16 | SqlType::I32 | SqlType::I64)
            ) || (dialect == Dialect::Mysql
                && c.native == Observed::Known(SqlType::I8));
            let empty_constraints = before.indexes.is_empty()
                && after.indexes.is_empty()
                && before.foreign_keys.is_empty()
                && after.foreign_keys.is_empty()
                && before.checks.is_empty()
                && after.checks.is_empty();
            // Only append-only nullable signed integer columns without defaults,
            // under explicitly ordinary, constraint-free table declarations.
            let supported = signed
                && c.nullable == Observed::Known(true)
                && c.default == MigrationDefault::Absent
                && position >= before.columns.len()
                && empty_constraints
                && before.effects == TableEffects::Ordinary
                && after.effects == TableEffects::Ordinary
                && mysql_names(dialect, &after.name, &c.name);
            self.change(
                Columns,
                Added,
                if supported { Supported } else { Unsupported },
                table,
                Some(&c.name),
            );
            if supported {
                let ty = match c.native {
                    Observed::Known(SqlType::I8) => "TINYINT",
                    Observed::Known(SqlType::I16) => "SMALLINT",
                    Observed::Known(SqlType::I32) => "INTEGER",
                    _ => "BIGINT",
                };
                self.sql.push(format!(
                    "ALTER TABLE {}.{} ADD COLUMN {} {ty} NULL",
                    quote(dialect, &after.name.namespace),
                    quote(dialect, &after.name.table),
                    quote(dialect, &c.name)
                ));
            }
        }
        self.named(
            Indexes,
            &before
                .indexes
                .iter()
                .map(|i| (&i.name, &i.definition))
                .collect::<Vec<_>>(),
            &after
                .indexes
                .iter()
                .map(|i| (&i.name, &i.definition))
                .collect::<Vec<_>>(),
            table,
        );
        self.named(
            ForeignKeys,
            &before
                .foreign_keys
                .iter()
                .map(|i| (&i.name, &i.definition))
                .collect::<Vec<_>>(),
            &after
                .foreign_keys
                .iter()
                .map(|i| (&i.name, &i.definition))
                .collect::<Vec<_>>(),
            table,
        );
        self.named(
            Checks,
            &before
                .checks
                .iter()
                .map(|i| (&i.name, &i.definition))
                .collect::<Vec<_>>(),
            &after
                .checks
                .iter()
                .map(|i| (&i.name, &i.definition))
                .collect::<Vec<_>>(),
            table,
        );
    }
    fn named<T: PartialEq>(
        &mut self,
        area: MigrationArea,
        before: &[(&Ident, &T)],
        after: &[(&Ident, &T)],
        table: Option<&MigrationName>,
    ) {
        use DifferenceKind::*;
        use MigrationSupport::Unsupported;
        let old: Vec<_> = before
            .iter()
            .filter(|b| after.iter().any(|a| a.0 == b.0))
            .map(|b| b.0)
            .collect();
        let new: Vec<_> = after
            .iter()
            .filter(|a| before.iter().any(|b| a.0 == b.0))
            .map(|a| a.0)
            .collect();
        if old != new {
            self.change(area, Order, Unsupported, table, None);
        }
        for (name, definition) in before {
            match after.iter().find(|a| a.0 == *name) {
                None => self.change(area, Removed, Unsupported, table, Some(name)),
                Some(a) if a.1 != *definition => {
                    self.change(area, Definition, Unsupported, table, Some(name))
                }
                _ => {}
            }
        }
        for (name, _) in after {
            if !before.iter().any(|b| b.0 == *name) {
                self.change(area, Added, Unsupported, table, Some(name));
            }
        }
    }
}
fn native_supported(dialect: Dialect, ty: SqlType) -> bool {
    dialect == Dialect::Mysql
        || !matches!(
            ty,
            SqlType::I8 | SqlType::U8 | SqlType::U16 | SqlType::U32 | SqlType::U64
        )
}
fn mysql_names(dialect: Dialect, table: &MigrationName, column: &Ident) -> bool {
    dialect != Dialect::Mysql
        || [&table.namespace, &table.table, column].iter().all(|i| {
            i.sensitive_text()
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        })
}
fn quote(dialect: Dialect, id: &Ident) -> String {
    let q = if dialect == Dialect::Postgres {
        '"'
    } else {
        '`'
    };
    format!(
        "{q}{}{q}",
        id.sensitive_text().replace(q, &format!("{q}{q}"))
    )
}
/// Pure bounded comparison. No DB/provider/clock/filesystem/network arguments or
/// calls. Target differences and incomplete coverage refuse the entire SQL plan.
pub fn plan_migration(before: &MigrationSchema, after: &MigrationSchema) -> MigrationPlan {
    use DifferenceKind::*;
    use MigrationArea::*;
    use MigrationSupport::Unsupported;
    let mut p = Planner {
        differences: vec![],
        areas: AREAS.map(|area| AreaView {
            area,
            changes: 0,
            unknown: false,
            unsupported: false,
        }),
        sql: vec![],
    };
    if before.coverage != after.coverage {
        p.change(Tables, Coverage, MigrationSupport::Unknown, None, None);
    }
    p.facts(before);
    p.facts(after);
    if before.target != after.target {
        p.change(Target, Definition, Unsupported, None, None);
    }
    if before.authority != after.authority {
        p.change(Target, Authority, Unsupported, None, None);
    }
    let old: Vec<_> = before
        .tables
        .iter()
        .filter(|b| after.tables.iter().any(|a| a.name == b.name))
        .map(|b| &b.name)
        .collect();
    let new: Vec<_> = after
        .tables
        .iter()
        .filter(|a| before.tables.iter().any(|b| a.name == b.name))
        .map(|a| &a.name)
        .collect();
    if old != new {
        p.change(Tables, Order, Unsupported, None, None);
    }
    for b in &before.tables {
        match after.tables.iter().find(|a| a.name == b.name) {
            Some(a) => p.table(b, a, before.target.dialect),
            None => p.change(Tables, Removed, Unsupported, Some(&b.name), None),
        }
    }
    for a in &after.tables {
        if !before.tables.iter().any(|b| b.name == a.name) {
            p.change(Tables, Added, Unsupported, Some(&a.name), None);
        }
    }
    let refused = p.areas.iter().any(|a| a.unknown || a.unsupported);
    let status = if refused {
        p.sql.clear();
        MigrationStatus::Refused
    } else if p.differences.is_empty() {
        MigrationStatus::NoChange
    } else {
        MigrationStatus::SqlPlanned
    };
    let risk = match status {
        MigrationStatus::Refused => Risk::Blocked,
        MigrationStatus::NoChange => Risk::Low,
        MigrationStatus::SqlPlanned => Risk::High,
    };
    let view = MigrationView {
        dialect: before.target.dialect,
        status,
        risk,
        areas: p.areas,
        steps: p.sql.len(),
        authority: [before.authority, after.authority],
    };
    MigrationPlan {
        before: before.clone(),
        after: after.clone(),
        differences: p.differences,
        sql: p.sql,
        view,
    }
}

#[cfg(test)]
#[path = "migration_tests.rs"]
mod tests;
