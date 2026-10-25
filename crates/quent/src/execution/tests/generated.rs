use super::*;
#[path = "../../../tests/codegen_support/mod.rs"]
mod fixtures;
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
