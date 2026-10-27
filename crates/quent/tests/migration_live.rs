//! Test-only raw DDL applies the generated finite sequence; no product apply API.
#![cfg(all(feature = "postgres", feature = "mysql", feature = "runtime-tokio"))]
mod migration_support;
use migration_support::*;
use quent_core::{migration::*, *};
use sqlx::{Connection, Row};
fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}
fn disposable_env(key: &str) -> String {
    assert_eq!(
        std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),
        Ok("quent_dispatch_test"),
        "requires dedicated fixture; no skip"
    );
    std::env::var(key).expect("dedicated DB required")
}
#[test]
fn live_migration_postgres_finite_sql_and_independent_catalog() {
    let dsn = disposable_env("QUENT_TEST_POSTGRES_DSN");
    runtime().block_on(async {
        let mut c = sqlx::PgConnection::connect(&dsn).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_, String>("SHOW server_version_num").fetch_one(&mut c).await.unwrap(), "160014");
        sqlx::query("CREATE TABLE quent_dispatch_test.migration_probe(a bigint NULL)").execute(&mut c).await.unwrap();
        sqlx::query("INSERT INTO quent_dispatch_test.migration_probe VALUES (7)").execute(&mut c).await.unwrap();
        let p = plan_migration(&schema(Dialect::Postgres, false), &schema(Dialect::Postgres, true));
        assert_eq!(p.public_view().status(), MigrationStatus::SqlPlanned);
        assert_eq!(p.sensitive_source().sql().len(), 3);
        for sql in p.sensitive_source().sql() { sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut c).await.unwrap(); }
        let rows = sqlx::query("SELECT column_name, data_type, is_nullable, column_default FROM information_schema.columns WHERE table_schema='quent_dispatch_test' AND table_name='migration_probe' ORDER BY ordinal_position").fetch_all(&mut c).await.unwrap();
        let expected = [("a", "bigint"), ("small", "smallint"), ("normal", "integer"), ("big", "bigint")];
        assert_eq!(rows.len(), 4);
        for (r, (name, ty)) in rows.iter().zip(expected) { assert_eq!(r.get::<String,_>(0), name); assert_eq!(r.get::<String,_>(1), ty); assert_eq!(r.get::<String,_>(2), "YES"); assert_eq!(r.get::<Option<String>,_>(3), None); }
        let r = sqlx::query("SELECT a, small, normal, big FROM quent_dispatch_test.migration_probe").fetch_one(&mut c).await.unwrap();
        assert_eq!(r.get::<i64,_>(0), 7); assert_eq!(r.get::<Option<i16>,_>(1), None); assert_eq!(r.get::<Option<i32>,_>(2), None); assert_eq!(r.get::<Option<i64>,_>(3), None);
        sqlx::query("INSERT INTO quent_dispatch_test.migration_probe VALUES (8,-32768,-2147483648,-9223372036854775808)").execute(&mut c).await.unwrap();
        let r = sqlx::query("SELECT small, normal, big FROM quent_dispatch_test.migration_probe WHERE a=8").fetch_one(&mut c).await.unwrap();
        assert_eq!(r.get::<i16,_>(0), i16::MIN); assert_eq!(r.get::<i32,_>(1), i32::MIN); assert_eq!(r.get::<i64,_>(2), i64::MIN);
        sqlx::query("DROP TABLE quent_dispatch_test.migration_probe").execute(&mut c).await.unwrap();
        c.close().await.unwrap();
    });
}
#[test]
fn live_migration_mysql_finite_sql_and_independent_catalog() {
    let dsn = disposable_env("QUENT_TEST_MYSQL_DSN");
    runtime().block_on(async {
        let mut c = sqlx::MySqlConnection::connect(&dsn).await.unwrap();
        assert_eq!(sqlx::query_scalar::<_, String>("SELECT VERSION()").fetch_one(&mut c).await.unwrap(), "8.4.8");
        sqlx::query("CREATE TABLE quent_dispatch_test.migration_probe(a bigint NULL) ENGINE=InnoDB").execute(&mut c).await.unwrap();
        sqlx::query("INSERT INTO quent_dispatch_test.migration_probe VALUES (7)").execute(&mut c).await.unwrap();
        let p = plan_migration(&schema(Dialect::Mysql, false), &schema(Dialect::Mysql, true));
        assert_eq!(p.public_view().status(), MigrationStatus::SqlPlanned); assert_eq!(p.sensitive_source().sql().len(), 4);
        for sql in p.sensitive_source().sql() { sqlx::query(sqlx::AssertSqlSafe(sql.as_str())).execute(&mut c).await.unwrap(); }
        let rows = sqlx::query("SELECT COLUMN_NAME,COLUMN_TYPE,IS_NULLABLE,COLUMN_DEFAULT FROM information_schema.COLUMNS WHERE TABLE_SCHEMA='quent_dispatch_test' AND TABLE_NAME='migration_probe' ORDER BY ORDINAL_POSITION").fetch_all(&mut c).await.unwrap();
        let expected = [("a", "bigint"), ("small", "smallint"), ("normal", "int"), ("big", "bigint"), ("tiny", "tinyint")];
        assert_eq!(rows.len(), 5);
        for (r, (name, ty)) in rows.iter().zip(expected) { assert_eq!(r.get::<String,_>(0), name); assert_eq!(r.get::<String,_>(1), ty); assert_eq!(r.get::<String,_>(2), "YES"); assert_eq!(r.get::<Option<String>,_>(3), None); }
        let r = sqlx::query("SELECT a,small,normal,big,tiny FROM quent_dispatch_test.migration_probe").fetch_one(&mut c).await.unwrap();
        assert_eq!(r.get::<i64,_>(0), 7); assert_eq!(r.get::<Option<i16>,_>(1), None); assert_eq!(r.get::<Option<i32>,_>(2), None); assert_eq!(r.get::<Option<i64>,_>(3), None); assert_eq!(r.get::<Option<i8>,_>(4), None);
        sqlx::query("INSERT INTO quent_dispatch_test.migration_probe VALUES (8,-32768,-2147483648,-9223372036854775808,-128)").execute(&mut c).await.unwrap();
        let r = sqlx::query("SELECT small,normal,big,tiny FROM quent_dispatch_test.migration_probe WHERE a=8").fetch_one(&mut c).await.unwrap();
        assert_eq!(r.get::<i16,_>(0), i16::MIN); assert_eq!(r.get::<i32,_>(1), i32::MIN); assert_eq!(r.get::<i64,_>(2), i64::MIN); assert_eq!(r.get::<i8,_>(3), i8::MIN);
        sqlx::query("DROP TABLE quent_dispatch_test.migration_probe").execute(&mut c).await.unwrap(); c.close().await.unwrap();
    });
}
