//! Finite DB-free walkthrough using explicitly synthetic application assertions.
//! No authentication, provider registration, execution or filesystem apply.
#[path = "../tests/codegen_support/mod.rs"]
mod application_assertions;
#[allow(dead_code)]
mod generated {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/codegen-v2/postgres-generated.rs"
    ));
}
use quent::codegen::{compare, generate};
use quent_core::{Dialect, PolicyDeclaration};
fn migration_review() {
    use quent_core::{Ident, SqlType, Value, migration::*};
    // Separate constraint-free telemetry table. This is independently supplied
    // migration source, never a conversion from manifest Fresh or the keyed model.
    let schema = |after: bool| {
        let mut columns = vec![
            MigrationColumn::new(
                Ident::new("usage").unwrap(),
                Observed::Known(SqlType::I64),
                Observed::Known(false),
                MigrationDefault::Literal(Value::I64(7)),
            )
            .unwrap(),
        ];
        if after {
            columns.push(
                MigrationColumn::new(
                    Ident::new("observed_count").unwrap(),
                    Observed::Known(SqlType::I32),
                    Observed::Known(true),
                    MigrationDefault::Absent,
                )
                .unwrap(),
            );
        }
        let table = MigrationTable::new(
            MigrationName::new(
                Ident::new("quent_dispatch_test").unwrap(),
                Ident::new("tenant_telemetry").unwrap(),
            ),
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
        .unwrap();
        MigrationSchema::new(
            MigrationTarget::new(
                "dedicated_test".into(),
                "dedicated_loopback".into(),
                Ident::new("quent_dispatch_test").unwrap(),
                Dialect::Postgres,
            )
            .unwrap(),
            SchemaAuthority::ApplicationDeclared,
            Knowledge::Complete,
            vec![table],
        )
        .unwrap()
    };
    let plan = plan_migration(&schema(false), &schema(true));
    let phases = plan.phases();
    assert_eq!(phases[0].capability(), PhaseCapability::NullableAppend);
    assert_eq!(phases[0].steps(), 1);
    assert_eq!(phases[1].capability(), PhaseCapability::NoWork);
    assert_eq!(phases[2].capability(), PhaseCapability::NoWork);
    println!("{}", plan.public_view().to_json());
    // No apply: runtime preflight, independent human instruction and current
    // attestations are explicit separate APIs, exercised by dedicated live tests.
}
fn main() {
    let source = include_bytes!("../../../fixtures/codegen-v2/postgres-source.json");
    let config = include_bytes!("../../../fixtures/codegen-v2/postgres-config.json");
    let existing = include_bytes!("../../../fixtures/codegen-v2/postgres-generated.rs");
    let previous = generate(source, config).unwrap();
    let proposed = generate(source, config).unwrap();
    assert_eq!(proposed.sensitive_rust_source().as_bytes(), existing);
    let regeneration = compare(&previous, &proposed, Some(existing));
    println!("{}", regeneration.to_json());
    // In an application these snapshots must come from authenticated server code
    // with correct current schema, physical routing, tenant and policy provenance.
    let columns = application_assertions::columns(Dialect::Postgres, "generated_items");
    let current = application_assertions::context(
        Dialect::Postgres,
        0,
        vec![
            PolicyDeclaration::Readonly(columns[3].clone()),
            PolicyDeclaration::Generated(columns[4].clone()),
        ],
    );
    let mapping = application_assertions::mapping(&current, Dialect::Postgres, "generated_items");
    let repo = generated::generated_items::Repository::bind(&current, &mapping).unwrap();
    // Unchanged small/normal/optional, SetNull big, Set(false) flag, Set("") txt.
    // This retains an inspection record only; there is no patch execute API.
    let patch = generated::generated_items::Patch::new()
        .set_null_big()
        .unwrap()
        .set_flag(false)
        .unwrap()
        .set_txt(String::new())
        .unwrap();
    let compiled = repo
        .compile_patch(generated::generated_items::Key::new(-16, 1), patch)
        .unwrap();
    println!("{}", compiled.public_view().to_json());
    println!(
        "{}",
        quent::review::inspection(&compiled.inspect()).to_json()
    );
    let read = repo
        .select(
            generated::generated_items::Model::compact(),
            Some(
                generated::generated_items::Key::new(-16, 1)
                    .predicate()
                    .unwrap(),
            ),
            vec![],
            1,
        )
        .unwrap();
    println!("{}", quent::review::inspection(&read.inspect()).to_json());
    // Offline domain review cannot establish live freshness or business authority.
    println!("{}", quent::review::manifest(source).unwrap().to_json());
    migration_review();
}
