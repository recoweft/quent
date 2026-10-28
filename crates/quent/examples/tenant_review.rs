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
}
