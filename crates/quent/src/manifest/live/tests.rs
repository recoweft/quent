use super::*;
use quent_core::{
    ApplicationContext, DatabaseBinding, DatabaseIdentity, Generation, Ident, PolicyIdentity,
    PolicySet, Settings, SettingsIdentity, TargetAddress,
};
use serde_json::json;
pub(super) fn context(d: Dialect, g: u64, address: &str) -> DatabaseContext {
    context_at(d, g, address, "quent_dispatch_test", None)
}
pub(super) fn context_at(
    d: Dialect,
    g: u64,
    address: &str,
    catalog: &str,
    subject: Option<&str>,
) -> DatabaseContext {
    let binding = DatabaseBinding::application_supplied(
        DatabaseIdentity::new("probe").unwrap(),
        TargetAddress::new(address).unwrap(),
        Ident::new(catalog).unwrap(),
        match d {
            Dialect::Postgres => quent_core::Dialect::Postgres,
            Dialect::Mysql => quent_core::Dialect::Mysql,
        },
    );
    let generation = Generation::new(g).unwrap();
    let policy = PolicySet::new(
        binding.clone(),
        PolicyIdentity::new("probe_policy").unwrap(),
        generation,
        vec![],
    )
    .unwrap();
    DatabaseContext::new(
        Settings::new(
            binding.clone(),
            SettingsIdentity::new("probe_settings").unwrap(),
            generation,
            policy,
        )
        .unwrap(),
        ApplicationContext::application_supplied(
            binding,
            None,
            subject.map(|s| quent_core::SubjectIdentity::new(s).unwrap()),
            None,
        )
        .unwrap(),
    )
    .unwrap()
}
fn source(d: Dialect, ns: &str, address: &str) -> ManifestSource {
    ManifestSource::parse(&serde_json::to_vec(&json!({"kind":"quent_manifest_source","version":1,"schema":{"kind":"quent_schema","version":1,"target":{"database":"probe","address":address,"catalog":"quent_dispatch_test","dialect":if d==Dialect::Postgres {"postgres"}else{"mysql"}},"coverage":"complete","tables":[{"path":[ns,"manifest_probe"],"semantics":"ordinary","columns_coverage":"complete","keys_coverage":"complete","columns":[{"name":"a","native":"i64","nullability":"nonnull","default":{"state":"literal","value":{"type":"i64","value":0}}},{"name":"b","native":"i32","nullability":"nonnull","default":{"state":"literal","value":{"type":"i32","value":7}}}],"keys":[{"kind":"primary","columns":["a","b"],"coverage":"complete"}]}]}})).unwrap()).unwrap()
}
pub(super) fn budget() -> Budget {
    Budget {
        queries: 65,
        rows: 8192,
        bytes: 1_048_576,
        string_bytes: 4096,
        tables: 16,
        columns: 256,
        keys: 16,
        branches: 256,
        timeout: Duration::from_secs(10),
    }
}
#[test]
fn finite_limits_defaults_and_clock() {
    let b = budget();
    assert!(b.check().is_ok());
    for bad in [
        Budget { queries: 66, ..b },
        Budget { rows: 0, ..b },
        Budget {
            string_bytes: 4097,
            ..b
        },
        Budget {
            timeout: Duration::MAX,
            ..b
        },
        Budget { branches: 257, ..b },
    ] {
        assert!(bad.check().is_err());
    }
    let now = Instant::now();
    assert!(expiry(now, Duration::ZERO).is_err());
    assert!(expiry(now, Duration::from_secs(301)).is_err());
    assert!(expiry(now, Duration::from_secs(300)).is_ok());
    assert!(now.checked_add(Duration::MAX).is_none());
    assert!(integer_default(Native::I16, Some("32768"), true, false).is_none());
    assert!(integer_default(Native::I32, Some("1+1"), true, false).is_none());
    assert!(matches!(
        integer_default(
            Native::I64,
            Some("'-9223372036854775808'::bigint"),
            true,
            false
        ),
        Some(DefaultValue::Literal {
            value: Literal::I64(i64::MIN)
        })
    ));
    assert!(matches!(
        integer_default(Native::I8, None, false, true),
        Some(DefaultValue::Unknown {})
    ));
    assert!(Scope::new("public", &["x", "x"]).is_err());
    assert!(Scope::new("CANARY", &["x"]).is_err());
    let mut m = Meter {
        b: Budget {
            string_bytes: 3,
            ..b
        },
        queries: 0,
        rows: 0,
        bytes: 0,
        trace: Arc::default(),
        disconnect_after_header: false,
    };
    assert!(m.row("[\"four\"]").is_err());
}
macro_rules! suite {
    ($name:ident,$reader:ident,$connection:ty,$options:ty,$d:expr,$env:literal,$ns:literal,$create:literal,$alter:literal,$extra:literal,$alternate:expr) => {
        #[test]
        fn $name() {
            use sqlx::{Connection,ConnectOptions};
            assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),Ok("quent_dispatch_test"));
            let options:$options=std::env::var($env).unwrap().parse().unwrap();
            let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            runtime.block_on(async {
                let mut setup=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                sqlx::query("DROP TABLE IF EXISTS manifest_probe").execute(&mut setup).await.unwrap();
                sqlx::query($create).execute(&mut setup).await.unwrap();
                let address=format!("{}:{}",options.get_host(),options.get_port());let c=context($d,1,&address);let source=source($d,$ns,&address);let scope=Scope::new($ns,&["manifest_probe"]).unwrap();
                #[cfg(feature="output-tracing")] let capture=crate::output_capture::Capture::default();
                #[cfg(feature="output-tracing")] let capture_guard=tracing::subscriber::set_default(capture.clone());
                let failed=$reader::connect(options.clone().username("CANARY_DRIVER_USER").password("CANARY_SECRET_PASSWORD"),c.clone(),Duration::from_secs(5)).await.unwrap_err();
                assert_eq!(failed,VerifyError::Connect);assert!(std::error::Error::source(&failed).is_none());assert!(!format!("{failed:?} {failed}").contains("CANARY"));
                let mut reader=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                let observed=reader.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap();
                assert_eq!(reader.compare(&c,&scope,&source,&observed).verdict(),Verdict::Matched);
                assert_eq!(*reader.trace.lock().unwrap(),vec!["connected","begin_attempt","begun","catalog_select","catalog_complete","catalog_select","catalog_complete","catalog_select","catalog_complete","catalog_select","catalog_complete","rollback_attempt","rolled_back"]);
                assert_eq!(reader.compare(&context($d,2,&address),&scope,&source,&observed).verdict(),Verdict::Unverified);
                assert_eq!(reader.compare(&context_at($d,1,&address,"quent_dispatch_test",Some("changed_subject")),&scope,&source,&observed).verdict(),Verdict::Unverified);
                let mut changed=source.clone();changed.source.schema.as_mut().unwrap().tables[0].columns[1].default=Some(DefaultValue::Literal{value:Literal::I32(8)});let changed=ManifestSource::parse(&changed.sensitive_source().canonical_source()).unwrap();
                assert_eq!(reader.compare(&c,&scope,&changed,&observed).verdict(),Verdict::Unverified);
                let other=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(other.compare(&c,&scope,&source,&observed).verdict(),Verdict::Unverified);
                assert!(!format!("{observed:?} {reader:?} {}",reader.compare(&c,&scope,&source,&observed)).contains("manifest_probe"));
                drop(other);
                #[cfg(feature="output-tracing")] {drop(capture_guard);capture.assert_safe(0);}
                let namespace=if $d==Dialect::Postgres {"manifest_other"}else{"quent_manifest_other"};
                let catalog=if $d==Dialect::Postgres {"quent_dispatch_test"}else{"quent_manifest_other"};
                let alternate_options=($alternate)(options.clone());
                let other_context=context_at($d,1,&address,catalog,None);
                let mut other_source=source.clone();let schema=other_source.source.schema.as_mut().unwrap();schema.target.catalog=catalog.into();schema.tables[0].path[0]=namespace.into();
                let other_scope=Scope::new(namespace,&["manifest_probe"]).unwrap();
                let mut alternate=$reader::connect(alternate_options,other_context.clone(),Duration::from_secs(5)).await.unwrap();
                let alternate_observation=alternate.observe(&other_context,&other_scope,&other_source,budget(),Duration::from_secs(30)).await.unwrap();
                assert_eq!(alternate.compare(&other_context,&other_scope,&other_source,&alternate_observation).verdict(),Verdict::Drift);
                assert_eq!(alternate.compare(&other_context,&other_scope,&other_source,&alternate_observation).counts()[3],1);
                assert_eq!(reader.compare(&other_context,&other_scope,&other_source,&alternate_observation).verdict(),Verdict::Unverified);
                let fake=ManifestSource::parse(include_bytes!("../../../../../fixtures/manifest-v1/postgres.json")).unwrap();
                let mut claimed=source.clone();claimed.source.policy=fake.source.policy;claimed.source.generated_code=fake.source.generated_code;claimed.source.live_observation=fake.source.live_observation;
                let o=reader.observe(&c,&scope,&claimed,budget(),Duration::from_secs(30)).await.unwrap();
                let summary=reader.compare(&c,&scope,&claimed,&o);assert_eq!(summary.verdict(),Verdict::Matched);
                let public:serde_json::Value=serde_json::from_str(&summary.to_json()).unwrap();for field in ["policy","generated_code","external_live"] {assert_eq!(public[field],"Unverified");}
                sqlx::query("CREATE TABLE manifest_probe_two(a bigint NOT NULL DEFAULT 0,b integer NOT NULL DEFAULT 7,PRIMARY KEY(a,b))").execute(&mut setup).await.unwrap();
                let mut multiple=source.clone();let schema=multiple.source.schema.as_mut().unwrap();let mut second=schema.tables[0].clone();second.path[1]="manifest_probe_two".into();schema.tables.insert(0,second);
                let multiple_scope=Scope::new($ns,&["manifest_probe_two","manifest_probe"]).unwrap();
                let o=reader.observe(&c,&multiple_scope,&multiple,budget(),Duration::from_secs(30)).await.unwrap();assert_eq!(reader.compare(&c,&multiple_scope,&multiple,&o).verdict(),Verdict::Matched);
                let before=reader.trace.lock().unwrap().len();
                for b in [Budget{tables:1,..budget()},Budget{branches:1,..budget()}] {assert_eq!(reader.observe(&c,&multiple_scope,&multiple,b,Duration::from_secs(30)).await.unwrap_err(),VerifyError::Budget);}
                let mut wrong=source.clone();wrong.source.schema.as_mut().unwrap().target.catalog="wrong_catalog".into();
                assert_eq!(reader.observe(&c,&scope,&wrong,budget(),Duration::from_secs(30)).await.unwrap_err(),VerifyError::Target);
                wrong.source.schema=None;assert_eq!(reader.observe(&c,&scope,&wrong,budget(),Duration::from_secs(30)).await.unwrap_err(),VerifyError::Missing);
                assert_eq!(reader.observe(&c,&scope,&source,budget(),Duration::ZERO).await.unwrap_err(),VerifyError::Expired);
                assert_eq!(reader.trace.lock().unwrap().len(),before,"invalid scope/source/TTL sends no catalog/Tx query");
                sqlx::query("DROP TABLE manifest_probe_two").execute(&mut setup).await.unwrap();
                let mut boundary=reader.observe(&c,&scope,&source,budget(),Duration::from_secs(300)).await.unwrap();
                boundary.deadline=boundary.completed;
                assert_eq!(reader.compare(&c,&scope,&source,&boundary).verdict(),Verdict::Stale);
                boundary.deadline=Instant::now()+Duration::from_secs(1);boundary.wall_completed_ms=u64::MAX;
                assert_eq!(reader.compare(&c,&scope,&source,&boundary).verdict(),Verdict::Stale);
                let changes:Vec<(Vec<&str>,Verdict)>=if $d==Dialect::Postgres { vec![
                    (vec!["ALTER TABLE manifest_probe ALTER COLUMN b TYPE bigint"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe ALTER COLUMN b SET DEFAULT 8"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe ALTER COLUMN b DROP DEFAULT"],Verdict::Unknown),
                    (vec!["ALTER TABLE manifest_probe DROP CONSTRAINT manifest_probe_pkey","ALTER TABLE manifest_probe ADD PRIMARY KEY(b,a)"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe ADD UNIQUE(b,a)"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe DROP CONSTRAINT manifest_probe_pkey","ALTER TABLE manifest_probe ALTER COLUMN b DROP NOT NULL"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe DROP COLUMN b CASCADE"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe ENABLE ROW LEVEL SECURITY"],Verdict::Unsupported),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u numeric"],Verdict::Unsupported),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u integer DEFAULT random()"],Verdict::Unsupported),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u integer DEFAULT NULL"],Verdict::Unknown),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u integer UNIQUE"],Verdict::Unsupported),
                    (vec!["CREATE UNIQUE INDEX manifest_expr ON manifest_probe((a+1))"],Verdict::Unsupported),
                    (vec!["CREATE UNIQUE INDEX manifest_partial ON manifest_probe(a) WHERE b>0"],Verdict::Unsupported),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u text"],Verdict::Unsupported),
                    (vec!["CREATE FUNCTION manifest_probe_fn() RETURNS trigger LANGUAGE plpgsql AS 'BEGIN RAISE EXCEPTION ''CANARY_default_must_not_execute''; END'","CREATE TRIGGER manifest_trigger BEFORE INSERT ON manifest_probe FOR EACH ROW EXECUTE FUNCTION manifest_probe_fn()"],Verdict::Unsupported),
                ]} else {vec![
                    (vec!["ALTER TABLE manifest_probe MODIFY b bigint NOT NULL DEFAULT 7"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe MODIFY b int unsigned NOT NULL DEFAULT 7"],Verdict::Unsupported),
                    (vec!["ALTER TABLE manifest_probe ALTER b SET DEFAULT 8"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe ALTER b DROP DEFAULT"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe DROP PRIMARY KEY, ADD PRIMARY KEY(b,a)"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe ADD UNIQUE(b,a)"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe DROP PRIMARY KEY, MODIFY b integer NULL DEFAULT 7"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe DROP PRIMARY KEY, DROP COLUMN b"],Verdict::Drift),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u decimal(12,2)"],Verdict::Unsupported),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u integer"],Verdict::Unknown),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u integer DEFAULT NULL"],Verdict::Unknown),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u integer DEFAULT (1+2)"],Verdict::Unsupported),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u integer UNIQUE DEFAULT 0"],Verdict::Unsupported),
                    (vec!["CREATE UNIQUE INDEX manifest_expr ON manifest_probe((a+1))"],Verdict::Unsupported),
                    (vec!["ALTER TABLE manifest_probe ADD COLUMN u text"],Verdict::Unsupported),

                ]};
                for (case,(ddls,verdict)) in changes.into_iter().enumerate() {
                    sqlx::query("DROP TABLE manifest_probe").execute(&mut setup).await.unwrap();
                    sqlx::query($create).execute(&mut setup).await.unwrap();
                    for ddl in ddls {sqlx::raw_sql(sqlx::AssertSqlSafe(ddl.to_owned())).execute(&mut setup).await.unwrap();}
                    let o=reader.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap();
                    assert_eq!(reader.compare(&c,&scope,&source,&o).verdict(),verdict,"case {case}");
                }
                sqlx::query("DROP TABLE manifest_probe").execute(&mut setup).await.unwrap();
                sqlx::query($create).execute(&mut setup).await.unwrap();
                if $d==Dialect::Postgres {sqlx::query("DROP FUNCTION manifest_probe_fn()").execute(&mut setup).await.unwrap();}
                if $d==Dialect::Mysql {
                    let mut triggered=source.clone();triggered.source.schema.as_mut().unwrap().tables[0].path[1]="manifest_trigger_probe".into();
                    let trigger_scope=Scope::new($ns,&["manifest_trigger_probe"]).unwrap();
                    let o=reader.observe(&c,&trigger_scope,&triggered,budget(),Duration::from_secs(30)).await.unwrap();
                    assert_eq!(reader.compare(&c,&trigger_scope,&triggered,&o).verdict(),Verdict::Unsupported);
                }
                // Independent expected native widths/signs/literals through the parser.
                let natives:Vec<(&str,Native,Literal)>=if $d==Dialect::Postgres {vec![
                    ("smallint NOT NULL DEFAULT -32768",Native::I16,Literal::I16(i16::MIN)),
                    ("integer NOT NULL DEFAULT -2147483648",Native::I32,Literal::I32(i32::MIN)),
                    ("bigint NOT NULL DEFAULT '-9223372036854775808'::bigint",Native::I64,Literal::I64(i64::MIN)),
                    ("boolean NOT NULL DEFAULT false",Native::Bool,Literal::Bool(false)),
                ]}else{vec![
                    ("tinyint NOT NULL DEFAULT -128",Native::I8,Literal::I8(i8::MIN)),
                    ("smallint NOT NULL DEFAULT -32768",Native::I16,Literal::I16(i16::MIN)),
                    ("int NOT NULL DEFAULT -2147483648",Native::I32,Literal::I32(i32::MIN)),
                    ("bigint NOT NULL DEFAULT -9223372036854775808",Native::I64,Literal::I64(i64::MIN)),
                    ("tinyint unsigned NOT NULL DEFAULT 255",Native::U8,Literal::U8(u8::MAX)),
                    ("smallint unsigned NOT NULL DEFAULT 65535",Native::U16,Literal::U16(u16::MAX)),
                    ("int unsigned NOT NULL DEFAULT 4294967295",Native::U32,Literal::U32(u32::MAX)),
                    ("bigint unsigned NOT NULL DEFAULT 18446744073709551615",Native::U64,Literal::U64(u64::MAX)),
                ]};
                for (definition,native,value) in natives {
                    sqlx::query(sqlx::AssertSqlSafe(format!("ALTER TABLE manifest_probe ADD COLUMN c {definition}"))).execute(&mut setup).await.unwrap();
                    let mut expected=source.clone();expected.source.schema.as_mut().unwrap().tables[0].columns.push(Column{name:"c".into(),native,nullability:Nullability::Nonnull,default:Some(DefaultValue::Literal{value})});
                    let expected=ManifestSource::parse(&expected.sensitive_source().canonical_source()).unwrap();
                    let o=reader.observe(&c,&scope,&expected,budget(),Duration::from_secs(30)).await.unwrap();
                    assert_eq!(reader.compare(&c,&scope,&expected,&o).verdict(),Verdict::Matched,"native {native:?}");
                    sqlx::query("ALTER TABLE manifest_probe DROP COLUMN c").execute(&mut setup).await.unwrap();
                }
                sqlx::query("CREATE UNIQUE INDEX manifest_finite_unique ON manifest_probe(b,a)").execute(&mut setup).await.unwrap();
                let mut keyed=source.clone();keyed.source.schema.as_mut().unwrap().tables[0].keys.push(Key{kind:KeyKind::Unique,columns:vec!["b".into(),"a".into()],coverage:Coverage::Complete});
                let o=reader.observe(&c,&scope,&keyed,budget(),Duration::from_secs(30)).await.unwrap();assert_eq!(reader.compare(&c,&scope,&keyed,&o).verdict(),Verdict::Matched);
                let drop_index=if $d==Dialect::Postgres {"DROP INDEX manifest_finite_unique"}else{"DROP INDEX manifest_finite_unique ON manifest_probe"};
                sqlx::query(sqlx::AssertSqlSafe(drop_index.to_owned())).execute(&mut setup).await.unwrap();
                let long_default=format!("ALTER TABLE manifest_probe ADD COLUMN c text DEFAULT ('{}')", "CANARY".repeat(700));
                sqlx::query(sqlx::AssertSqlSafe(long_default)).execute(&mut setup).await.unwrap();
                let mut long=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                if $d==Dialect::Postgres {
                    let o=long.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap();assert_eq!(long.compare(&c,&scope,&source,&o).verdict(),Verdict::Unsupported);
                } else {assert_eq!(long.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap_err(),VerifyError::Budget);}
                sqlx::query("ALTER TABLE manifest_probe DROP COLUMN c").execute(&mut setup).await.unwrap();
                for i in 0..16 {sqlx::query(sqlx::AssertSqlSafe(format!("CREATE UNIQUE INDEX manifest_key_{i} ON manifest_probe(a)"))).execute(&mut setup).await.unwrap();}
                let mut keys=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(keys.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap_err(),VerifyError::Budget);
                sqlx::query("DROP TABLE manifest_probe").execute(&mut setup).await.unwrap();sqlx::query($create).execute(&mut setup).await.unwrap();
                // Real metadata visibility failure uses a separate limited fixture role.
                let hidden_options=options.clone().username("quent_manifest_hidden");
                let mut hidden=$reader::connect(hidden_options,c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(hidden.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap_err(),VerifyError::Unknown);
                for b in [Budget{queries:1,..budget()},Budget{bytes:1,..budget()},Budget{string_bytes:1,..budget()},Budget{columns:1,..budget()}] {
                    let mut r=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                    assert_eq!(r.observe(&c,&scope,&source,b,Duration::from_secs(30)).await.unwrap_err(),VerifyError::Budget);
                    assert_eq!(r.compare(&c,&scope,&source,&observed).verdict(),Verdict::Unverified);
                }
                let mut disconnected=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();disconnected.disconnect_after_header=true;
                assert_eq!(disconnected.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap_err(),VerifyError::Io);
                assert!(disconnected.trace.lock().unwrap().contains(&"fixture_disconnect"));
                assert_eq!(disconnected.trace.lock().unwrap().iter().filter(|e|**e=="catalog_complete").count(),2);
                assert_eq!(disconnected.compare(&c,&scope,&source,&observed).verdict(),Verdict::Unverified);
                let mut timed=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(timed.observe(&c,&scope,&source,Budget{timeout:Duration::from_nanos(1),..budget()},Duration::from_secs(30)).await.unwrap_err(),VerifyError::Expired);
                let mut canceled=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                let mut f=Box::pin(canceled.observe(&c,&scope,&source,budget(),Duration::from_secs(30)));
                let pending=std::future::poll_fn(|cx|std::task::Poll::Ready(f.as_mut().poll(cx).is_pending())).await;
                assert!(pending);drop(f);
                assert_eq!(canceled.compare(&c,&scope,&source,&observed).verdict(),Verdict::Unverified);
                assert!(!canceled.trace.lock().unwrap().contains(&"rolled_back"));
                sqlx::query($alter).execute(&mut setup).await.unwrap();
                let drift=reader.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap();
                assert_eq!(reader.compare(&c,&scope,&source,&drift).verdict(),Verdict::Drift);
                assert_eq!(reader.compare(&c,&scope,&source,&drift).counts()[1],1);
                sqlx::query($extra).execute(&mut setup).await.unwrap();
                let unsupported=reader.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap();
                assert_eq!(reader.compare(&c,&scope,&source,&unsupported).verdict(),Verdict::Unsupported);
                let mut limited=$reader::connect(options.clone(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(limited.observe(&c,&scope,&source,Budget{rows:1,..budget()},Duration::from_secs(30)).await.unwrap_err(),VerifyError::Budget);
                assert_eq!(limited.compare(&c,&scope,&source,&observed).verdict(),Verdict::Unverified);
                let mut tiny=$reader::connect(options,c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(tiny.observe(&c,&scope,&source,budget(),Duration::from_nanos(1)).await.unwrap_err(),VerifyError::Expired);
                assert_eq!(tiny.compare(&c,&scope,&source,&observed).verdict(),Verdict::Unverified);
                sqlx::query("DROP TABLE manifest_probe").execute(&mut setup).await.unwrap();
                assert_eq!(reader.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap_err(),VerifyError::Unknown);
                let view=if $d==Dialect::Postgres {"CREATE VIEW manifest_probe AS SELECT 1::bigint AS a,7::integer AS b"} else {"CREATE VIEW manifest_probe AS SELECT CAST(1 AS SIGNED) AS a,7 AS b"};
                sqlx::query(sqlx::AssertSqlSafe(view.to_owned())).execute(&mut setup).await.unwrap();
                let mut view_reader=$reader::connect(std::env::var($env).unwrap().parse::<$options>().unwrap(),c.clone(),Duration::from_secs(5)).await.unwrap();
                assert_eq!(view_reader.observe(&c,&scope,&source,budget(),Duration::from_secs(30)).await.unwrap_err(),VerifyError::Unsupported);
                sqlx::query("DROP VIEW manifest_probe").execute(&mut setup).await.unwrap();
                println!("catalog {:?}: actual ordinary match, added-column drift, generated unsupported, exact-context/source/reader nonpromotion, row budget, observation expiry, missing refusal",$d);
            });
        }
    }
}
#[cfg(feature = "postgres")]
suite!(
    live_catalog_postgres,
    PostgresReader,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    "public",
    "CREATE TABLE manifest_probe(a bigint NOT NULL DEFAULT 0,b integer NOT NULL DEFAULT 7,PRIMARY KEY(a,b))",
    "ALTER TABLE manifest_probe ADD COLUMN c integer NOT NULL DEFAULT 0",
    "ALTER TABLE manifest_probe ADD COLUMN g integer GENERATED ALWAYS AS (b+1) STORED",
    |o: sqlx::postgres::PgConnectOptions| o.options([("search_path", "manifest_other")])
);
#[cfg(feature = "mysql")]
suite!(
    live_catalog_mysql,
    MysqlReader,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "quent_dispatch_test",
    "CREATE TABLE manifest_probe(a bigint NOT NULL DEFAULT 0,b integer NOT NULL DEFAULT 7,PRIMARY KEY(a,b))",
    "ALTER TABLE manifest_probe ADD COLUMN c integer NOT NULL DEFAULT 0",
    "ALTER TABLE manifest_probe ADD COLUMN g integer GENERATED ALWAYS AS (b+1) STORED",
    |o: sqlx::mysql::MySqlConnectOptions| o.database("quent_manifest_other")
);
