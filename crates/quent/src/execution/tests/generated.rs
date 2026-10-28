use super::*;
#[path = "../../../tests/codegen_support/mod.rs"]
mod fixtures;
#[allow(dead_code)]
mod patches {
    pub mod pg {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/codegen-v2/postgres-generated.rs"
        ));
    }
    pub mod my {
        include!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/codegen-v2/mysql-generated.rs"
        ));
    }
}
macro_rules! suite {
    ($name:ident,$feature:literal,$driver:ident,$options:ty,$connection:ty,$dialect:expr,$dsn:literal,$module:ident,$create:literal,$seed:literal) => {
        #[cfg(feature=$feature)]
        #[test]
        fn $name() {
            use fixtures::$module::generated_items as model;
            use sqlx::{Connection,ConnectOptions};
            assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),Ok("quent_dispatch_test"));
            let options:$options=std::env::var($dsn).unwrap().parse().unwrap();
            let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            runtime.block_on(async {
                let d=$dialect;
                let mut raw=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                sqlx::query($create).execute(&mut raw).await.unwrap();
                sqlx::query($seed).execute(&mut raw).await.unwrap();
                // One v2 generated consumer workflow: regenerate exact bytes,
                // bind independently supplied current policy, inspect a three-state
                // patch, then execute a generated read and verify no patch effects.
                {
                    use patches::$module::generated_items as v2;
                    let columns=fixtures::columns(d,"generated_items");
                    let declarations=if d==Dialect::Postgres {vec![PolicyDeclaration::Readonly(columns[3].clone()),PolicyDeclaration::Generated(columns[4].clone())]}else{vec![]};
                    let current=fixtures::context(d,0,declarations);
                    let mapping=fixtures::mapping(&current,d,"generated_items");
                    let repository=v2::Repository::bind(&current,&mapping).unwrap();
                    let dialect_name=if d==Dialect::Postgres {"postgres"}else{"mysql"};
                    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/codegen-v2");
                    let source=std::fs::read(root.join(format!("{dialect_name}-source.json"))).unwrap();
                    let config=std::fs::read(root.join(format!("{dialect_name}-config.json"))).unwrap();
                    let existing=std::fs::read(root.join(format!("{dialect_name}-generated.rs"))).unwrap();
                    let prior=crate::codegen::generate(&source,&config).unwrap();
                    let next=crate::codegen::generate(&source,&config).unwrap();
                    assert_eq!(next.sensitive_rust_source().as_bytes(),existing);
                    assert_eq!(crate::codegen::compare(&prior,&next,Some(&existing)).state(),crate::codegen::RegenerationState::NoChange);
                    let patch=v2::Patch::new().set_null_big().unwrap().set_txt(String::new()).unwrap().set_flag(false).unwrap();
                    let compiled=repository.compile_patch(v2::Key::new(-16,1),patch).unwrap();
                    assert!(compiled.matches_current(&current,&mapping));
                    assert_eq!(compiled.inspect().sensitive_source().args(),[Value::Null(SqlType::I64),Value::Bool(false),Value::Text(String::new()),Value::I16(-16),Value::I64(1),Value::I64(11)]);
                    #[cfg(feature="review")]
                    {
                        assert_eq!(crate::review::inspection(&compiled.inspect()).state(),crate::review::State::Unknown);
                        assert_eq!(crate::review::manifest(&source).unwrap().state(),crate::review::State::Unknown);
                    }
                    let mut consumer=$driver::connect(options.clone(),current.clone()).await.unwrap();
                    assert!(consumer.recorder.calls.is_empty());
                    assert!(consumer.recorder.events.is_empty());
                    let read=repository.select(v2::Model::compact(),Some(v2::Key::new(-16,1).predicate().unwrap()),vec![],1).unwrap();
                    let inspected=read.inspect();let record=inspected.sensitive_source();let allocation=record.sql().as_ptr() as usize;let args=record.args().to_vec();let metadata=record.metadata().clone();
                    let result=execute!(read,consumer,current,mapping,$module).await.unwrap();
                    assert_eq!(result.sensitive_rows().len(),1);
                    assert_eq!(*result.sensitive_rows()[0].sensitive_id(),1);
                    assert_eq!(*result.sensitive_rows()[0].sensitive_optional(),None);
                    assert_eq!(consumer.recorder.calls.len(),1);
                    assert_eq!(consumer.recorder.sql_addresses,[allocation]);
                    assert_eq!(consumer.recorder.calls[0].1,args);
                    assert_eq!(consumer.recorder.metadata,[metadata]);
                    // Independent observer checks retained old data: inspection
                    // never executed the patch's SetNull/false/empty text.
                    use sqlx::Row;
                    let row=sqlx::query("SELECT big,txt FROM quent_dispatch_test.generated_items WHERE part=-16 AND id=1 AND tenant=11").fetch_one(&mut raw).await.unwrap();
                    assert_eq!(row.get::<i64,_>(0),i64::MIN);
                    assert_eq!(row.get::<String,_>(1),"CANARY_GENERATED_RESULT");
                    println!("QT-AI-14 {d:?} generated-v2: exact regen NoChange; patch inspection SQL0/provider0; generated-read SQL1 same allocation/args/metadata; independent observer unchanged big/txt; no patch execution authority");
                }
                let c=fixtures::context(d,0,vec![]);let m=fixtures::mapping(&c,d,"generated_items");
                let repo=model::Repository::bind(&c,&m).unwrap();
                let make=||repo.select(model::Model::detail(),Some(model::Key::new(-16,1).predicate().unwrap()),vec![model::Model::id().desc()],10).unwrap();
                let read=make();let inspected=read.inspect();let source=inspected.sensitive_source();
                let sql=if d==Dialect::Postgres {"SELECT \"generated_items\".\"part\", \"generated_items\".\"id\", \"generated_items\".\"small\", \"generated_items\".\"normal\", \"generated_items\".\"big\", \"generated_items\".\"flag\", \"generated_items\".\"txt\", \"generated_items\".\"optional\" FROM \"quent_dispatch_test\".\"generated_items\" WHERE (((\"generated_items\".\"part\" = $1) AND (\"generated_items\".\"id\" = $2)) AND (\"generated_items\".\"tenant\" = $3)) ORDER BY \"generated_items\".\"id\" DESC LIMIT $4"} else {"SELECT `generated_items`.`part`, `generated_items`.`id`, `generated_items`.`small`, `generated_items`.`normal`, `generated_items`.`big`, `generated_items`.`flag`, `generated_items`.`txt`, `generated_items`.`optional`, `generated_items`.`tiny`, `generated_items`.`u_tiny`, `generated_items`.`u_small`, `generated_items`.`u_normal`, `generated_items`.`u_big` FROM `quent_dispatch_test`.`generated_items` WHERE (((`generated_items`.`part` = ?) AND (`generated_items`.`id` = ?)) AND (`generated_items`.`tenant` = ?)) ORDER BY `generated_items`.`id` DESC LIMIT ?"};
                assert_eq!(source.sql(),sql);let address=source.sql().as_ptr() as usize;
                let args=vec![Value::I16(-16),Value::I64(1),Value::I64(11),Value::I64(10)];assert_eq!(source.args(),args);
                assert_eq!(source.metadata().family(),Family::Select);assert_eq!(inspected.scope(),Scope::Single);
                let mut session=$driver::connect(options.clone(),c.clone()).await.unwrap();
                let rows=execute!(read,session,c,m,$module).await.unwrap();
                let row=&rows.sensitive_rows()[0];assert_eq!(rows.sensitive_rows().len(),1);
                assert_eq!(*row.sensitive_part(),-16i16);assert_eq!(*row.sensitive_id(),1i64);
                assert_eq!(*row.sensitive_small(),Some(i16::MIN));assert_eq!(*row.sensitive_normal(),Some(i32::MAX));assert_eq!(*row.sensitive_big(),Some(i64::MIN));assert_eq!(*row.sensitive_flag(),Some(false));assert_eq!(row.sensitive_txt().as_deref(),Some("CANARY_GENERATED_RESULT"));assert_eq!(*row.sensitive_optional(),None);
                extra!(row,$module);
                assert!(!format!("{rows:?} {row:?}").contains("CANARY"));
                assert_eq!(session.recorder.calls,vec![(sql.into(),args)]);assert_eq!(session.recorder.sql_addresses,vec![address]);
                let compact=repo.select(model::Model::compact(),Some(model::Key::new(-16,1).predicate().unwrap()),vec![],1).unwrap();
                let rows=execute!(compact,session,c,m,$module).await.unwrap();assert_eq!(rows.sensitive_rows().len(),1);assert_eq!(*rows.sensitive_rows()[0].sensitive_optional(),None);assert_eq!(*rows.sensitive_rows()[0].sensitive_id(),1);
                let before=session.recorder.calls.len();
                for variant in 1..=8 {
                    let changed=fixtures::context(d,variant,vec![]);let read=make();
                    assert_eq!(execute!(read,session,changed,m,$module).await.unwrap_err(),ExecutionError::ContextMismatch);
                }
                let other=fixtures::mapping(&c,d,"other_items");let read=make();assert_eq!(execute!(read,session,c,other,$module).await.unwrap_err(),ExecutionError::ContextMismatch);
                let partial=repo.select(model::Model::compact(),Some(model::Model::optional().is_null()),vec![],1).unwrap();
                assert!(matches!(execute!(partial,session,c,m,$module).await,Err(ExecutionError::RiskRefused|ExecutionError::Incomplete)));
                for declaration in [PolicyDeclaration::Pii(fixtures::table("generated_items").column(fixtures::id("txt"),SqlType::Text,true)),PolicyDeclaration::RequiredFilter(fixtures::table("generated_items").column(fixtures::id("txt"),SqlType::Text,true))] {
                    let changed=fixtures::context(d,0,vec![declaration]);let mapping=fixtures::mapping(&changed,d,"generated_items");assert!(model::Repository::bind(&changed,&mapping).is_err());
                }
                assert_eq!(session.recorder.calls.len(),before);
                // Tenant is taken solely from context: another tenant's complete
                // key compiles, but the server result is empty under the guard.
                let wrong_tenant=repo.select(model::Model::compact(),Some(model::Key::new(-16,2).predicate().unwrap()),vec![],1).unwrap();let rows=execute!(wrong_tenant,session,c,m,$module).await.unwrap();assert!(rows.sensitive_rows().is_empty());
                let nulls=repo.select(model::Model::detail(),Some(model::Key::new(-16,3).predicate().unwrap()),vec![],1).unwrap();let rows=execute!(nulls,session,c,m,$module).await.unwrap();let row=&rows.sensitive_rows()[0];assert_eq!(*row.sensitive_small(),None);assert_eq!(*row.sensitive_normal(),None);assert_eq!(*row.sensitive_big(),None);assert_eq!(*row.sensitive_flag(),None);assert_eq!(*row.sensitive_txt(),None);assert_eq!(*row.sensitive_optional(),None);null_extra!(row,$module);
                // Actual native text budget failure quarantines; no retry/send.
                sqlx::query("UPDATE quent_dispatch_test.generated_items SET txt=REPEAT('x',4097) WHERE id=1").execute(&mut raw).await.unwrap();
                let before=session.recorder.calls.len();let read=make();assert_eq!(execute!(read,session,c,m,$module).await.unwrap_err(),ExecutionError::ResultLimit);assert!(!session.is_available());assert_eq!(session.recorder.calls.len(),before+1);
                let read=make();assert_eq!(execute!(read,session,c,m,$module).await.unwrap_err(),ExecutionError::SessionUnavailable);assert_eq!(session.recorder.calls.len(),before+1);
                println!("QT-AI-07 {:?}: generated composite key/native/NULL and partial projection; identical inspected/dispatched SQL allocation; context/mapping/high/PII/RequiredFilter refusals 0 sends; text budget quarantine/no retry",d);
            });
        }
    }
}
macro_rules! execute {
    ($read:ident,$session:ident,$current:ident,$mapping:ident,pg) => {
        $read.execute_postgres(&mut $session, &$current, &$mapping)
    };
    ($read:ident,$session:ident,$current:ident,$mapping:ident,my) => {
        $read.execute_mysql(&mut $session, &$current, &$mapping)
    };
}
macro_rules! extra {
    ($row:ident,pg) => {};
    ($row:ident,my) => {
        assert_eq!(*$row.sensitive_tiny(), Some(i8::MIN));
        assert_eq!(*$row.sensitive_u_tiny(), Some(u8::MAX));
        assert_eq!(*$row.sensitive_u_small(), Some(u16::MAX));
        assert_eq!(*$row.sensitive_u_normal(), Some(u32::MAX));
        assert_eq!(*$row.sensitive_u_big(), Some(u64::MAX));
    };
}
macro_rules! null_extra {
    ($row:ident,pg) => {};
    ($row:ident,my) => {
        assert_eq!(*$row.sensitive_tiny(), None);
        assert_eq!(*$row.sensitive_u_tiny(), None);
        assert_eq!(*$row.sensitive_u_small(), None);
        assert_eq!(*$row.sensitive_u_normal(), None);
        assert_eq!(*$row.sensitive_u_big(), None);
    };
}
suite!(
    live_generated_postgres,
    "postgres",
    PostgresSession,
    sqlx::postgres::PgConnectOptions,
    sqlx::PgConnection,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    pg,
    "CREATE TABLE quent_dispatch_test.generated_items(part smallint NOT NULL,id bigint NOT NULL,tenant bigint NOT NULL,small smallint,normal integer,big bigint,flag boolean,txt text,optional bigint,PRIMARY KEY(part,id))",
    "INSERT INTO quent_dispatch_test.generated_items VALUES(-16,1,11,-32768,2147483647,-9223372036854775808,false,'CANARY_GENERATED_RESULT',NULL),(-16,2,22,NULL,NULL,NULL,NULL,NULL,NULL),(-16,3,11,NULL,NULL,NULL,NULL,NULL,NULL)"
);
suite!(
    live_generated_mysql,
    "mysql",
    MysqlSession,
    sqlx::mysql::MySqlConnectOptions,
    sqlx::MySqlConnection,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    my,
    "CREATE TABLE quent_dispatch_test.generated_items(part smallint NOT NULL,id bigint NOT NULL,tenant bigint NOT NULL,small smallint,normal int,big bigint,flag boolean,txt text,optional bigint,tiny tinyint,u_tiny tinyint unsigned,u_small smallint unsigned,u_normal int unsigned,u_big bigint unsigned,PRIMARY KEY(part,id))",
    "INSERT INTO quent_dispatch_test.generated_items VALUES(-16,1,11,-32768,2147483647,-9223372036854775808,false,'CANARY_GENERATED_RESULT',NULL,-128,255,65535,4294967295,18446744073709551615),(-16,2,22,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL),(-16,3,11,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL,NULL)"
);
