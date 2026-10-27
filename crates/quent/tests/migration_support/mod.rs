use quent_core::{migration::*, *};
pub fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
pub fn column(name: &str, ty: SqlType) -> MigrationColumn {
    MigrationColumn::new(
        id(name),
        Observed::Known(ty),
        Observed::Known(true),
        MigrationDefault::Absent,
    )
    .unwrap()
}
pub fn schema(d: Dialect, extra: bool) -> MigrationSchema {
    let mut columns = vec![column("a", SqlType::I64)];
    if extra {
        columns.extend([
            column("small", SqlType::I16),
            column("normal", SqlType::I32),
            column("big", SqlType::I64),
        ]);
        if d == Dialect::Mysql {
            columns.push(column("tiny", SqlType::I8));
        }
    }
    MigrationSchema::new(
        MigrationTarget::new(
            "disposable".into(),
            "explicit_loopback".into(),
            id("quent_dispatch_test"),
            d,
        )
        .unwrap(),
        SchemaAuthority::ApplicationDeclared,
        Knowledge::Complete,
        vec![
            MigrationTable::new(
                MigrationName::new(id("quent_dispatch_test"), id("migration_probe")),
                TableEffects::Ordinary,
                TableCoverage {
                    columns: Knowledge::Complete,
                    indexes: Knowledge::Complete,
                    foreign_keys: Knowledge::Complete,
                    checks: Knowledge::Complete,
                },
                columns,
                vec![],
                vec![],
                vec![],
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
