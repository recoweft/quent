#![allow(dead_code)]
use quent::operation_spec::*;
use quent_core::*;
pub mod pg {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/codegen-v1/postgres-generated.rs"
    ));
}
pub mod my {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/codegen-v1/mysql-generated.rs"
    ));
}
pub fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
pub fn table(name: &str) -> Table {
    Table::new(vec![id("quent_dispatch_test"), id(name)], None).unwrap()
}
pub fn columns(d: Dialect, name: &str) -> Vec<Column> {
    let table = table(name);
    let mut declarations = vec![
        ("part", SqlType::I16, false),
        ("id", SqlType::I64, false),
        ("tenant", SqlType::I64, false),
        ("small", SqlType::I16, true),
        ("normal", SqlType::I32, true),
        ("big", SqlType::I64, true),
        ("flag", SqlType::Bool, true),
        ("txt", SqlType::Text, true),
        ("optional", SqlType::I64, true),
    ];
    if d == Dialect::Mysql {
        declarations.extend([
            ("tiny", SqlType::I8, true),
            ("u_tiny", SqlType::U8, true),
            ("u_small", SqlType::U16, true),
            ("u_normal", SqlType::U32, true),
            ("u_big", SqlType::U64, true),
        ]);
    }
    declarations
        .into_iter()
        .map(|(n, t, null)| table.column(id(n), t, null))
        .collect()
}
pub fn mapping(c: &DatabaseContext, d: Dialect, name: &str) -> TypedModelContract {
    TypedModelContract::application_asserted(
        name,
        table(name),
        columns(d, name)
            .iter()
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
            .collect(),
        ModelCoverage::Complete,
        c,
    )
    .unwrap()
}
// Independent application assertions, never reconstructed from source/config JSON.
pub fn context(d: Dialect, variant: u8, extra: Vec<PolicyDeclaration>) -> DatabaseContext {
    let binding = DatabaseBinding::application_supplied(
        DatabaseIdentity::new(if variant == 1 {
            "other"
        } else {
            "dedicated_test"
        })
        .unwrap(),
        TargetAddress::new(if variant == 2 {
            "other"
        } else {
            "dedicated_loopback"
        })
        .unwrap(),
        id("quent_dispatch_test"),
        d,
    );
    let g = Generation::new(1).unwrap();
    let mut facts = vec![];
    let mut policies = vec![];
    for n in ["generated_items", "other_items"] {
        let mut cols = columns(d, n);
        if variant == 5 {
            cols.reverse();
        }
        if variant == 9 {
            cols.pop();
        }
        let component = |name: &str| {
            columns(d, n)
                .into_iter()
                .find(|c| c.sensitive_name().sensitive_text() == name)
                .unwrap()
        };
        let mut keys = vec![component("part"), component("id")];
        if variant == 6 {
            keys.reverse();
        }
        facts.push(
            ScopeFacts::application_asserted(
                table(n),
                d,
                cols,
                vec![Key::Primary(keys)],
                if variant == 10 {
                    TableContext::Unsupported
                } else {
                    TableContext::OrdinaryBaseTableWithoutEffects
                },
            )
            .unwrap(),
        );
        if n == "generated_items" {
            policies.push(PolicyDeclaration::Tenant(component("tenant")));
        }
    }
    policies.extend(extra);
    DatabaseContext::new(
        Settings::new(
            binding.clone(),
            SettingsIdentity::new(if variant == 7 { "changed" } else { "settings" }).unwrap(),
            g,
            PolicySet::new(
                binding.clone(),
                PolicyIdentity::new("policy").unwrap(),
                g,
                policies,
            )
            .unwrap(),
        )
        .unwrap(),
        ApplicationContext::application_supplied(
            binding.clone(),
            Some(
                SchemaSnapshot::application_supplied(
                    binding.clone(),
                    SchemaIdentity::new(if variant == 8 { "changed" } else { "schema" }).unwrap(),
                    g,
                    facts,
                )
                .unwrap(),
            ),
            Some(SubjectIdentity::new(if variant == 3 { "changed" } else { "subject" }).unwrap()),
            Some(
                TenantSnapshot::application_supplied(
                    binding,
                    table("generated_items").column(id("tenant"), SqlType::I64, false),
                    Value::I64(if variant == 4 { 22 } else { 11 }),
                )
                .unwrap(),
            ),
        )
        .unwrap(),
    )
    .unwrap()
}
