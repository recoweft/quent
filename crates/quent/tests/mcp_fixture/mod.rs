#![allow(dead_code)]
use quent::operation_spec::*;
use quent_core::*;
use serde_json::{Value as Json, json};
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn table() -> Table {
    Table::new(vec![id("canary_catalog"), id("canary_items")], None).unwrap()
}
fn column(name: &str, ty: SqlType, nullable: bool) -> Column {
    table().column(id(name), ty, nullable)
}
fn columns(ty: SqlType, nullable: bool) -> Vec<Column> {
    vec![
        column("id", SqlType::I64, false),
        column("tenant", SqlType::I64, false),
        column("deleted", SqlType::Bool, false),
        column("payload", ty, nullable),
    ]
}
pub fn context(
    d: Dialect,
    ty: SqlType,
    nullable: bool,
    subject: &str,
    extras: Vec<PolicyDeclaration>,
) -> DatabaseContext {
    let b = DatabaseBinding::application_supplied(
        DatabaseIdentity::new("db").unwrap(),
        TargetAddress::new("route").unwrap(),
        id("canary_catalog"),
        d,
    );
    let g = Generation::new(1).unwrap();
    let mut ps = vec![
        PolicyDeclaration::Tenant(column("tenant", SqlType::I64, false)),
        PolicyDeclaration::SoftDelete(column("deleted", SqlType::Bool, false)),
    ];
    ps.extend(extras);
    let facts = ScopeFacts::application_asserted(
        table(),
        d,
        columns(ty, nullable),
        vec![Key::Primary(vec![
            column("tenant", SqlType::I64, false),
            column("id", SqlType::I64, false),
        ])],
        TableContext::OrdinaryBaseTableWithoutEffects,
    )
    .unwrap();
    DatabaseContext::new(
        Settings::new(
            b.clone(),
            SettingsIdentity::new("settings").unwrap(),
            g,
            PolicySet::new(b.clone(), PolicyIdentity::new("policy").unwrap(), g, ps).unwrap(),
        )
        .unwrap(),
        ApplicationContext::application_supplied(
            b.clone(),
            Some(
                SchemaSnapshot::application_supplied(
                    b.clone(),
                    SchemaIdentity::new("schema").unwrap(),
                    g,
                    vec![facts],
                )
                .unwrap(),
            ),
            Some(SubjectIdentity::new(subject).unwrap()),
            Some(
                TenantSnapshot::application_supplied(
                    b,
                    column("tenant", SqlType::I64, false),
                    Value::I64(11),
                )
                .unwrap(),
            ),
        )
        .unwrap(),
    )
    .unwrap()
}
fn registered(ty: SqlType, nullable: bool) -> Vec<TypedColumn> {
    columns(ty, nullable)
        .into_iter()
        .map(|c| {
            TypedColumn::application_supplied(
                c.sensitive_name().sensitive_text(),
                c.sensitive_name().clone(),
                ColumnType::Native(c.sql_type()),
                c.nullable(),
                if c.sensitive_name().sensitive_text() == "tenant" {
                    ColumnRole::Tenant
                } else {
                    ColumnRole::Ordinary
                },
                true,
            )
            .unwrap()
        })
        .collect()
}
pub fn mapping(c: &DatabaseContext, ty: SqlType, nullable: bool) -> TypedModelContract {
    TypedModelContract::application_asserted(
        "canary_items",
        table(),
        registered(ty, nullable),
        ModelCoverage::Complete,
        c,
    )
    .unwrap()
}
pub fn source(filter: Option<Json>) -> SpecSource {
    let mut j = json!({"kind":"quent_operation_spec","version":1,"model":"canary_items","fields":["id","payload"],"order":[{"field":"id","direction":"desc"}],"limit":10});
    if let Some(f) = filter {
        j["filter"] = f;
    }
    SpecSource::parse(&serde_json::to_vec(&j).unwrap()).unwrap()
}
fn cmp(tag: &str, v: Json) -> Json {
    json!({"op":"eq","field":"payload","value":{"type":tag,"value":v}})
}
