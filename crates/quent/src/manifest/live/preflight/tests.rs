use super::*;
use crate::manifest::live::tests::{budget, context};
use quent_core::{Ident, migration::*};
use std::num::NonZeroU64;
fn nz(v: u64) -> NonZeroU64 {
    NonZeroU64::new(v).unwrap()
}
fn plan(
    d: Dialect,
    ns: &str,
    address: &str,
    extra_name: &str,
    before_default: MigrationDefault,
) -> MigrationPlan {
    let col = |name: &str, default| {
        MigrationColumn::new(
            Ident::new(name).unwrap(),
            Observed::Known(SqlType::I64),
            Observed::Known(false),
            default,
        )
        .unwrap()
    };
    let table = |extra| {
        let mut columns = vec![col("a", before_default.clone())];
        if extra {
            columns.push(
                MigrationColumn::new(
                    Ident::new(extra_name).unwrap(),
                    Observed::Known(SqlType::I64),
                    Observed::Known(true),
                    MigrationDefault::Absent,
                )
                .unwrap(),
            );
        }
        MigrationTable::new(
            MigrationName::new(
                Ident::new(ns).unwrap(),
                Ident::new("preflight_probe").unwrap(),
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
        .unwrap()
    };
    let schema = |extra| {
        MigrationSchema::new(
            MigrationTarget::new(
                "probe".into(),
                address.into(),
                Ident::new("quent_dispatch_test").unwrap(),
                if d == Dialect::Postgres {
                    quent_core::Dialect::Postgres
                } else {
                    quent_core::Dialect::Mysql
                },
            )
            .unwrap(),
            SchemaAuthority::ApplicationDeclared,
            Knowledge::Complete,
            vec![table(extra)],
        )
        .unwrap()
    };
    plan_migration(&schema(false), &schema(true))
}
fn two_tables(p: &MigrationPlan, ns: &str) -> MigrationPlan {
    let sensitive = p.sensitive_source();
    let (b, a) = sensitive.schemas();
    let extend = |schema: &MigrationSchema| {
        let mut tables = schema.sensitive_tables().to_vec();
        let old = &tables[0];
        tables.push(
            MigrationTable::new(
                MigrationName::new(
                    Ident::new(ns).unwrap(),
                    Ident::new("preflight_second").unwrap(),
                ),
                TableEffects::Ordinary,
                TableCoverage {
                    columns: Knowledge::Complete,
                    indexes: Knowledge::Complete,
                    foreign_keys: Knowledge::Complete,
                    checks: Knowledge::Complete,
                },
                old.sensitive_columns().to_vec(),
                vec![],
                vec![],
                vec![],
            )
            .unwrap(),
        );
        MigrationSchema::new(
            schema.sensitive_target().clone(),
            schema.authority(),
            schema.coverage(),
            tables,
        )
        .unwrap()
    };
    plan_migration(&extend(b), &extend(a))
}
fn request<'p>(p: &'p MigrationPlan, ns: &str, role: &str) -> PreflightRequest<'p> {
    PreflightRequest::new(
        p,
        MigrationPhase::Expand,
        Scope::new(ns, &["preflight_probe"]).unwrap(),
        Visibility::new(role, nz(1)).unwrap(),
        DdlExclusion::ApplicationDeclared(nz(1)),
    )
    .unwrap()
}
fn extension(ns: &str, role: &str) -> Vec<Vec<Option<String>>> {
    vec![vec![
        Some(ns.into()),
        Some("preflight_probe".into()),
        Some(role.into()),
        Some("0".into()),
        Some("0".into()),
        Some("false".into()),
    ]]
}
#[test]
fn c22_preflight_pure_bounds_claims_and_private_result_refusals() {
    assert!(Visibility::new("", nz(1)).is_err());
    assert!(Visibility::new(&"x".repeat(129), nz(1)).is_err());
    assert!(Visibility::new("\ncanary", nz(1)).is_err());
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let ns = if d == Dialect::Postgres {
            "public"
        } else {
            "quent_dispatch_test"
        };
        let p = plan(
            d,
            ns,
            "host:1",
            "tail",
            MigrationDefault::Literal(Value::I64(7)),
        );
        let r = request(&p, ns, "CANARY_role");
        let expected = r.expected().unwrap();
        let ext = extension(ns, "CANARY_role");
        assert!(matches_result(&r, &expected, &ext).is_ok());
        for phase in [MigrationPhase::Backfill, MigrationPhase::Contract] {
            assert_eq!(
                PreflightRequest::new(
                    &p,
                    phase,
                    r.scope.clone(),
                    r.visibility.clone(),
                    r.exclusion
                )
                .unwrap_err(),
                PreflightError::NoWork
            );
        }
        assert_eq!(
            PreflightRequest::new(
                &p,
                MigrationPhase::Expand,
                r.scope.clone(),
                r.visibility.clone(),
                DdlExclusion::Uncontrolled
            )
            .unwrap_err(),
            PreflightError::Incomplete
        );
        assert_eq!(
            PreflightRequest::new(
                &p,
                MigrationPhase::Expand,
                Scope::new(ns, &["other"]).unwrap(),
                r.visibility.clone(),
                r.exclusion
            )
            .unwrap_err(),
            PreflightError::Mismatch
        );
        let mut variants = vec![vec![], vec![ext[0].clone(), ext[0].clone()], vec![vec![]]];
        for (i, v) in [
            (0, "other_ns"),
            (1, "other_table"),
            (2, "other_role"),
            (3, "1"),
            (3, "garbage"),
            (4, "1"),
            (5, "true"),
        ] {
            let mut changed = ext.clone();
            changed[0][i] = Some(v.into());
            variants.push(changed);
        }
        for changed in variants {
            assert!(matches_result(&r, &expected, &changed).is_err());
        }
        let mut changed = expected.clone();
        changed.tables[0].columns[0].default = Some(DefaultValue::Unknown {});
        assert_eq!(
            matches_result(&r, &changed, &ext),
            Err(PreflightError::Incomplete)
        );
        changed = expected.clone();
        changed.tables[0].columns.reverse();
        let duplicate = changed.tables[0].columns[0].clone();
        changed.tables[0].columns.push(duplicate);
        assert!(matches_result(&r, &changed, &ext).is_err());
        let multi = two_tables(&p, ns);
        let multi_r = PreflightRequest::new(
            &multi,
            MigrationPhase::Expand,
            Scope::new(ns, &["preflight_second", "preflight_probe"]).unwrap(),
            r.visibility.clone(),
            r.exclusion,
        )
        .unwrap();
        let expected_multi = multi_r.expected().unwrap();
        let mut ext_multi = ext.clone();
        let mut second = ext[0].clone();
        second[1] = Some("preflight_second".into());
        ext_multi.push(second);
        assert!(matches_result(&multi_r, &expected_multi, &ext_multi).is_ok());
        ext_multi.reverse();
        assert!(matches_result(&multi_r, &expected_multi, &ext_multi).is_err());
        ext_multi = vec![ext[0].clone(), ext[0].clone()];
        assert!(matches_result(&multi_r, &expected_multi, &ext_multi).is_err());
        let c = context(d, 1, "host:1");
        let id = Arc::new(());
        let now = Instant::now();
        let w = wall().unwrap();
        let mut evidence = PreflightObservation {
            request: r.retained(),
            context: c.clone(),
            identity: id.clone(),
            observed: expected,
            extension: ext,
            started: now,
            completed: now,
            deadline: now + Duration::from_secs(30),
            wall_started_ms: w,
            wall_completed_ms: w,
        };
        assert_eq!(
            evidence.compare(true, &id, &c, &r).state(),
            PreflightState::ConditionsObserved
        );
        assert_eq!(
            evidence.compare(false, &id, &c, &r).state(),
            PreflightState::Unverified
        );
        assert_eq!(
            evidence.compare(true, &Arc::new(()), &c, &r).state(),
            PreflightState::Mismatch
        );
        let mut altered = r.retained();
        altered.phase = MigrationPhase::Contract;
        assert_eq!(
            evidence.compare(true, &id, &c, &altered).state(),
            PreflightState::Mismatch
        );
        altered = r.retained();
        altered.visibility.generation = nz(2);
        assert_eq!(
            evidence.compare(true, &id, &c, &altered).state(),
            PreflightState::Mismatch
        );
        altered = r.retained();
        altered.exclusion = DdlExclusion::ApplicationDeclared(nz(2));
        assert_eq!(
            evidence.compare(true, &id, &c, &altered).state(),
            PreflightState::Mismatch
        );
        evidence.deadline = Instant::now();
        assert_eq!(
            evidence.compare(true, &id, &c, &r).state(),
            PreflightState::Stale
        );
        evidence.deadline = now + Duration::from_secs(30);
        evidence.completed = evidence.deadline;
        assert_eq!(
            evidence.compare(true, &id, &c, &r).state(),
            PreflightState::Stale
        );
        evidence.completed = now;
        evidence.wall_completed_ms = u64::MAX;
        assert_eq!(
            evidence.compare(true, &id, &c, &r).state(),
            PreflightState::Stale
        );
        let json = PreflightView {
            state: PreflightState::ConditionsObserved,
            steps: 1,
            tables: 1,
        }
        .to_json();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            v,
            serde_json::json!({"kind":"quent_migration_preflight_view","version":1,"state":"ConditionsObserved","coverage":"Partial","phase":"Expand","steps":1,"tables":1,"conditions":3,"full_effects":"Unverified","authority":"ApplicationDeclared"})
        );
        assert_eq!(format!("{:?}", r.exclusion), "DdlExclusion([REDACTED])");
        for text in [
            json,
            format!(
                "{r:?} {evidence:?} {:?}",
                Visibility::new("CANARY_role", nz(1)).unwrap()
            ),
        ] {
            assert!(!text.contains("CANARY"));
        }
        assert!(format!("{:?}", r.plan.sensitive_source()).contains("REDACTED"));
        assert!(
            r.visibility.role.contains("CANARY"),
            "canary detecting positive control"
        );
    }
}
fn record(case: &str, trace: &std::sync::Mutex<Vec<&'static str>>) {
    let trace = trace.lock().unwrap();
    let count = |name| trace.iter().filter(|x| **x == name).count();
    eprintln!(
        "C22/{case}: connection={} SELECT_attempt={} SELECT_complete={} BEGIN_attempt={} BEGIN_ACK={} ROLLBACK_attempt={} ROLLBACK_ACK={} provider=0 DDL_DML_apply_COMMIT=0 fixture_termination={}",
        count("connected"),
        count("catalog_select"),
        count("catalog_complete"),
        count("begin_attempt"),
        count("begun"),
        count("rollback_attempt"),
        count("rolled_back"),
        count("fixture_disconnect")
    );
}
macro_rules! live_suite {
    ($name:ident,$reader:ident,$conn:ty,$options:ty,$d:expr,$env:literal,$ns:literal,$role:literal,$create:literal,$drift:literal) => {
        #[test]
        fn $name() {
            use sqlx::{Connection,ConnectOptions,Row};
            assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),Ok("quent_dispatch_test"));
            let opts:$options=std::env::var($env).unwrap().parse().unwrap();
            let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            rt.block_on(async {
                // Fixture-only DDL/DML; observer is an independent connection.
                let mut setup=<$conn>::connect_with(&opts.clone().disable_statement_logging()).await.unwrap();
                sqlx::query($create).execute(&mut setup).await.unwrap();
                sqlx::query("INSERT INTO preflight_probe(a) VALUES (7)").execute(&mut setup).await.unwrap();
                let mut observer=<$conn>::connect_with(&opts.clone().disable_statement_logging()).await.unwrap();
                let address=format!("{}:{}",opts.get_host(),opts.get_port());let c=context($d,1,&address);
                let p=plan($d,$ns,&address,"tail",MigrationDefault::Literal(Value::I64(7)));let r=request(&p,$ns,$role);
                #[cfg(feature="output-tracing")] let capture=crate::output_capture::Capture::default();
                #[cfg(feature="output-tracing")] let guard=tracing::subscriber::set_default(capture.clone());
                let mut reader=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                // Unpolled live request performs no I/O.
                let before=reader.trace.lock().unwrap().clone();drop(reader.observe_migration(&c,&r,budget(),Duration::from_secs(30)));
                assert_eq!(*reader.trace.lock().unwrap(),before);
                // Explicit opt-in: five SELECT ACKs, one BEGIN ACK, one ROLLBACK ACK.
                let e=reader.observe_migration(&c,&r,budget(),Duration::from_secs(30)).await.unwrap();
                record(concat!(stringify!($name),"/positive"), &reader.trace);
                let trace=reader.trace.lock().unwrap().clone();
                for (event,count) in [("connected",1),("begin_attempt",1),("begun",1),("catalog_select",5),("catalog_complete",5),("rollback_attempt",1),("rolled_back",1)] {assert_eq!(trace.iter().filter(|x|**x==event).count(),count,"{event}");}
                assert_eq!(reader.compare_migration(&c,&r,&e).state(),PreflightState::ConditionsObserved);
                assert_eq!(reader.compare_migration(&c,&r,&e).state(),PreflightState::ConditionsObserved,"observational reuse is permitted, not execution reuse");
                assert_eq!(reader.compare_migration(&context($d,2,&address),&r,&e).state(),PreflightState::Mismatch);
                let other_p=plan($d,$ns,&address,"different_tail",MigrationDefault::Literal(Value::I64(7)));let other_r=request(&other_p,$ns,$role);
                assert_eq!(reader.compare_migration(&c,&other_r,&e).state(),PreflightState::Mismatch);
                let wrong_target=plan($d,$ns,"different:1","tail",MigrationDefault::Literal(Value::I64(7)));let wrong_target_r=request(&wrong_target,$ns,$role);
                assert_eq!(reader.compare_migration(&c,&wrong_target_r,&e).state(),PreflightState::Mismatch);
                let mut changed=r.retained();changed.visibility.generation=nz(2);assert_eq!(reader.compare_migration(&c,&changed,&e).state(),PreflightState::Mismatch);
                changed=r.retained();changed.exclusion=DdlExclusion::ApplicationDeclared(nz(2));assert_eq!(reader.compare_migration(&c,&changed,&e).state(),PreflightState::Mismatch);
                changed=r.retained();changed.phase=MigrationPhase::Contract;assert_eq!(reader.compare_migration(&c,&changed,&e).state(),PreflightState::Mismatch);
                let other=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(other.compare_migration(&c,&r,&e).state(),PreflightState::Mismatch);drop(other);
                let calls=reader.trace.lock().unwrap().len();
                assert!(reader.observe_migration(&c,&wrong_target_r,budget(),Duration::from_secs(30)).await.is_err());
                for ttl in [Duration::ZERO,Duration::from_secs(301)] {assert!(reader.observe_migration(&c,&r,budget(),ttl).await.is_err());}
                assert_eq!(reader.trace.lock().unwrap().len(),calls,"before BEGIN refusal");
                let mut expired=reader.observe_migration(&c,&r,budget(),Duration::from_secs(30)).await.unwrap();expired.deadline=Instant::now();
                assert_eq!(reader.compare_migration(&c,&r,&expired).state(),PreflightState::Stale);
                // Real role mismatch, finite budgets, early timeout and downstream error never mint evidence.
                for b in [Budget{queries:2,..budget()},Budget{rows:1,..budget()},Budget{string_bytes:1,..budget()},Budget{timeout:Duration::from_nanos(1),..budget()}] {
                    let mut limited=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                    assert!(limited.observe_migration(&c,&r,b,Duration::from_secs(30)).await.is_err());
                    assert_eq!(limited.compare_migration(&c,&r,&e).state(),PreflightState::Unverified);
                    record(concat!(stringify!($name),"/budget_or_timeout"), &limited.trace);
                }
                let bad_role=request(&p,$ns,"CANARY_wrong_role");let mut limited=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(limited.observe_migration(&c,&bad_role,budget(),Duration::from_secs(30)).await.unwrap_err(),PreflightError::Mismatch);
                record(concat!(stringify!($name),"/role_mismatch"), &limited.trace);
                let mut broken=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();broken.disconnect_after_header=true;
                assert!(broken.observe_migration(&c,&r,budget(),Duration::from_secs(30)).await.is_err());
                assert_eq!(broken.compare_migration(&c,&r,&e).state(),PreflightState::Unverified);
                assert!(!broken.trace.lock().unwrap().contains(&"rolled_back"));
                record(concat!(stringify!($name),"/downstream_error_no_rollback_ACK"), &broken.trace);
                let mut canceled=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                let mut f=Box::pin(canceled.observe_migration(&c,&r,budget(),Duration::from_secs(30)));
                std::future::poll_fn(|cx| {assert!(f.as_mut().poll(cx).is_pending());std::task::Poll::Ready(())}).await;drop(f);
                assert_eq!(canceled.compare_migration(&c,&r,&e).state(),PreflightState::Unverified);
                assert!(!canceled.trace.lock().unwrap().contains(&"rolled_back"));
                record(concat!(stringify!($name),"/cancel"), &canceled.trace);
                assert!(!format!("{e:?} {r:?} {reader:?} {}",reader.compare_migration(&c,&r,&e)).contains("preflight_probe"));
                #[cfg(feature="output-tracing")] {drop(guard);capture.assert_safe(0);}
                // Independent observer confirms no product data/DDL write.
                assert_eq!(sqlx::query_scalar::<_,i64>("SELECT a FROM preflight_probe").fetch_one(&mut observer).await.unwrap(),7);
                assert_eq!(sqlx::query_scalar::<_,i64>("SELECT COUNT(*) FROM information_schema.columns WHERE table_name='preflight_probe'").fetch_one(&mut observer).await.unwrap(),1);
                let independent=sqlx::query("SELECT column_name,data_type,is_nullable,column_default FROM information_schema.columns WHERE table_name='preflight_probe' ORDER BY ordinal_position").fetch_all(&mut observer).await.unwrap();
                assert_eq!(independent.len(),1);
                let independent=&independent[0];
                assert_eq!(independent.get::<String,_>(0),"a");
                assert_eq!(independent.get::<String,_>(1),"bigint");
                assert_eq!(independent.get::<String,_>(2),"NO");
                assert_eq!(independent.get::<Option<String>,_>(3).as_deref(),Some("7"));
                sqlx::query("CREATE TABLE preflight_second(a bigint NOT NULL DEFAULT 7)").execute(&mut setup).await.unwrap();
                let multi=two_tables(&p,$ns);
                let multi_r=PreflightRequest::new(&multi,MigrationPhase::Expand,Scope::new($ns,&["preflight_second","preflight_probe"]).unwrap(),r.visibility.clone(),r.exclusion).unwrap();
                let mut multi_reader=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                let multi_e=multi_reader.observe_migration(&c,&multi_r,budget(),Duration::from_secs(30)).await.unwrap();
                assert_eq!(multi_reader.compare_migration(&c,&multi_r,&multi_e).state(),PreflightState::ConditionsObserved);
                assert_eq!(multi_reader.trace.lock().unwrap().iter().filter(|x|**x=="catalog_complete").count(),9);
                record(concat!(stringify!($name),"/two_tables"), &multi_reader.trace);
                sqlx::query("DROP TABLE preflight_second").execute(&mut setup).await.unwrap();
                // Metadata drift after observation is detected ONLY by a fresh opt-in read.
                sqlx::query($drift).execute(&mut setup).await.unwrap();
                assert_eq!(reader.compare_migration(&c,&r,&e).state(),PreflightState::ConditionsObserved,"recheck does not pretend to continuously monitor DDL");
                let mut fresh=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(fresh.observe_migration(&c,&r,budget(),Duration::from_secs(30)).await.unwrap_err(),PreflightError::Drift);
                record(concat!(stringify!($name),"/drift"), &fresh.trace);
                sqlx::query("ALTER TABLE preflight_probe ALTER COLUMN a DROP DEFAULT").execute(&mut setup).await.unwrap();
                let absent=plan($d,$ns,&address,"tail",MigrationDefault::Absent);let absent_r=request(&absent,$ns,$role);
                let mut unknown=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                if $d==Dialect::Postgres {assert_eq!(unknown.observe_migration(&c,&absent_r,budget(),Duration::from_secs(30)).await.unwrap_err(),PreflightError::Incomplete);}
                else {assert!(unknown.observe_migration(&c,&absent_r,budget(),Duration::from_secs(30)).await.is_ok());}
                record(concat!(stringify!($name),"/default_absence"), &unknown.trace);
                // Nullable no-default and explicit DEFAULT NULL cannot prove source syntax on either DB.
                sqlx::query("DROP TABLE preflight_probe").execute(&mut setup).await.unwrap();
                sqlx::query("CREATE TABLE preflight_probe(a bigint NULL)").execute(&mut setup).await.unwrap();
                // Keep exact nullable source declarations separate from observations.
                let mut null_unknown=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                // Even the non-null source cannot upgrade unknown normalized defaults.
                assert_eq!(null_unknown.observe_migration(&c,&absent_r,budget(),Duration::from_secs(30)).await.unwrap_err(),PreflightError::Incomplete);
                sqlx::query("ALTER TABLE preflight_probe ALTER COLUMN a SET DEFAULT NULL").execute(&mut setup).await.unwrap();
                let mut explicit_null=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(explicit_null.observe_migration(&c,&absent_r,budget(),Duration::from_secs(30)).await.unwrap_err(),PreflightError::Incomplete);
                record(concat!(stringify!($name),"/nullable_default_NULL_unknown"), &explicit_null.trace);
                sqlx::query("DROP TABLE preflight_probe").execute(&mut setup).await.unwrap();
                sqlx::query($create).execute(&mut setup).await.unwrap();
                // Real nonunique index must never disappear into v1's unique-key projection.
                sqlx::query("ALTER TABLE preflight_probe ALTER COLUMN a SET DEFAULT 7").execute(&mut setup).await.unwrap();
                sqlx::query("CREATE INDEX preflight_nonunique ON preflight_probe(a)").execute(&mut setup).await.unwrap();
                let mut indexed=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(indexed.observe_migration(&c,&r,budget(),Duration::from_secs(30)).await.unwrap_err(),PreflightError::Incomplete);
                record(concat!(stringify!($name),"/nonunique_index"), &indexed.trace);
                let mut hidden=$reader::connect(opts.clone().username("quent_manifest_hidden"),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert!(hidden.observe_migration(&c,&r,budget(),Duration::from_secs(30)).await.is_err());
                record(concat!(stringify!($name),"/limited_role"), &hidden.trace);
                sqlx::query("DROP TABLE preflight_probe").execute(&mut setup).await.unwrap();
                sqlx::query("CREATE TABLE preflight_probe(a text)").execute(&mut setup).await.unwrap();
                let mut metadata_type=$reader::connect(opts.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(metadata_type.observe_migration(&c,&r,budget(),Duration::from_secs(30)).await.unwrap_err(),PreflightError::Incomplete);
                record(concat!(stringify!($name),"/unsupported_metadata_type"), &metadata_type.trace);
                sqlx::query("DROP TABLE preflight_probe").execute(&mut setup).await.unwrap();
                observer.close().await.unwrap();setup.close().await.unwrap();
                eprintln!("{}: successful observation SELECT=5/ACK=5 BEGIN attempt=1/ACK=1 ROLLBACK attempt=1/ACK=1; provider=0 DDL/DML/apply/COMMIT=0; independent catalog/data observer verified; fixture DDL/DML separate",stringify!($name));
            });
        }
    }
}
#[cfg(feature = "postgres")]
live_suite!(
    live_preflight_postgres,
    PostgresReader,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    "public",
    "quent_test",
    "CREATE TABLE preflight_probe(a bigint NOT NULL DEFAULT 7)",
    "ALTER TABLE preflight_probe ALTER COLUMN a SET DEFAULT 8"
);
#[cfg(feature = "mysql")]
live_suite!(
    live_preflight_mysql,
    MysqlReader,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "quent_dispatch_test",
    "quent_test@%",
    "CREATE TABLE preflight_probe(a bigint NOT NULL DEFAULT 7) ENGINE=InnoDB",
    "ALTER TABLE preflight_probe ALTER COLUMN a SET DEFAULT 8"
);
