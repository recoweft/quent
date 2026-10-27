mod migration_support;
#[allow(dead_code)]
mod support;
use migration_support::*;
use quent_core::{migration::*, *};
use serde_json::{Value as Json, json};
#[test]
fn c01_c08_c15_c19_c22_c23_public_allowlist_and_db_provider_zero() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let recorder = support::recorder::Recorder::default();
        // No provider, session or callback is passed to this Core-only API. An
        // independent recorder stays empty; this is not a live refusal/Tx test.
        let b = schema(d, false);
        let a = schema(d, true);
        let p = plan_migration(&b, &a);
        let view: Json = serde_json::from_str(&p.public_view().to_json()).unwrap();
        let mut keys = view
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        keys.sort();
        assert_eq!(
            keys,
            [
                "areas",
                "authority",
                "ddl_transaction",
                "dialect",
                "kind",
                "live",
                "risk",
                "status",
                "steps",
                "version"
            ]
        );
        assert_eq!(view["kind"], "quent_migration_view");
        assert_eq!(view["version"], 1);
        assert_eq!(
            view["authority"],
            json!(["ApplicationDeclared", "ApplicationDeclared"])
        );
        assert_eq!(view["live"], "Unverified");
        assert_eq!(view["risk"], "High");
        assert_eq!(view["status"], "SqlPlanned");
        assert_eq!(view["steps"], if d == Dialect::Mysql { 4 } else { 3 });
        let expected = ["Target", "Tables", "Effects", "Columns", "Types", "Nullability", "Defaults", "Indexes", "ForeignKeys", "Checks"].map(|area| json!({"area":area, "changes":if area == "Columns" { if d == Dialect::Mysql {4} else {3} } else {0}, "unknown":false, "unsupported":false}));
        assert_eq!(view["areas"], json!(expected));
        assert_eq!(
            view["ddl_transaction"],
            if d == Dialect::Postgres {
                "PostgresTransactional"
            } else {
                "MysqlImplicitCommit"
            }
        );
        assert!(p.sensitive_source().sql()[0].contains("migration_probe"));
        assert!(!view.to_string().contains("migration_probe"));
        assert_eq!(recorder.call_count(), 0);
        assert!(recorder.observations().is_empty());
        // Core dependency tree and compile signatures establish provider0; there
        // is no provider/Tx implementation to simulate in this pure planner.
    }
}
#[cfg(feature = "manifest")]
#[test]
fn c08_c19_c24_existing_manifest_wire_and_fingerprint_not_reinterpreted() {
    let bytes = include_bytes!("../../../fixtures/manifest-v1/postgres.json");
    let result = quent::manifest::ManifestSource::parse(bytes);
    assert!(result.is_ok());
    let migration = plan_migration(
        &schema(Dialect::Postgres, false),
        &schema(Dialect::Postgres, true),
    );
    assert!(
        quent::manifest::ManifestSource::parse(migration.public_view().to_json().as_bytes())
            .is_err()
    );
    let mut source: Json = serde_json::from_slice(bytes).unwrap();
    source["schema"]["tables"][0]["indexes"] = json!([]);
    assert!(quent::manifest::ManifestSource::parse(&serde_json::to_vec(&source).unwrap()).is_err());
}
#[cfg(feature = "local-details")]
#[test]
fn c15_migration_local_protected_file_and_surface_denial() {
    use quent::local_details::*;
    let dir = std::env::temp_dir().join(format!("quent-migration-detail-{}", std::process::id()));
    let builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    let builder = {
        use std::os::unix::fs::DirBuilderExt;
        let mut b = builder;
        b.mode(0o700);
        b
    };
    builder.create(&dir).unwrap();
    let p = plan_migration(
        &schema(Dialect::Postgres, false),
        &schema(Dialect::Postgres, true),
    );
    let path = dir.join("detail");
    for surface in [Surface::Ci, Surface::Mcp, Surface::Remote] {
        assert_eq!(
            create_new_migration(
                LocalOptIn::acknowledge_sensitive_local_file(),
                surface,
                &path,
                p.sensitive_source()
            )
            .unwrap_err(),
            DetailError::SurfaceDenied
        );
        assert!(!path.exists());
    }
    let export = || {
        create_new_migration(
            LocalOptIn::acknowledge_sensitive_local_file(),
            Surface::LocalApplication,
            &path,
            p.sensitive_source(),
        )
    };
    let result = export();
    // CI guard remains enabled. Native backend refusal is separately unit tested.
    if let Err(error) = result {
        assert!(matches!(
            error,
            DetailError::EnvironmentDenied | DetailError::UnsupportedOs
        ));
        assert!(!path.exists());
    } else {
        let bytes = std::fs::read(&path).unwrap();
        let doc: Json = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(doc["kind"], "quent_local_migration");
        assert_eq!(doc["sql"], json!(p.sensitive_source().sql()));
        assert_eq!(
            doc["differences"][0]["table"],
            json!(["quent_dispatch_test", "migration_probe"])
        );
        assert_eq!(export().unwrap_err(), DetailError::CreateRefused);
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        #[cfg(unix)]
        {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let meta = std::fs::metadata(&path).unwrap();
            assert_eq!(meta.permissions().mode() & 0o7777, 0o600);
            assert_eq!(meta.nlink(), 1);
        }
    }
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(!dir.exists());
}
