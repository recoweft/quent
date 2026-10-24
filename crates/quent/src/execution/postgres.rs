use super::*;
use futures_util::TryStreamExt;
use sqlx::postgres::{PgConnectOptions, PgRow};
use sqlx::{
    Column as _, ConnectOptions, Connection, PgConnection, Postgres, Row, Type, TypeInfo, ValueRef,
};

/// Owns a connection and its full immutable application snapshot. No raw executor
/// getter, callbacks, pool routing or transaction API is exposed. Delayed handles
/// are private-record consumers bound to this exact connection lifetime.
/// Construction authenticates neither the application context nor physical DB.
/// ```compile_fail
/// fn bypass(s: &mut quent::PostgresSession, c: &quent_core::DatabaseContext, p: quent_core::Plan) { s.execute(c, p); }
/// ```
/// ```compile_fail
/// fn bypass(s: &mut quent::PostgresSession, c: &quent_core::DatabaseContext, p: quent_core::SensitiveSource<'_>) { s.execute(c, p); }
/// ```
/// ```compile_fail
/// fn bypass(s: &mut quent::PostgresSession, c: &quent_core::DatabaseContext) { s.execute(c, "DELETE FROM items"); }
/// ```
/// ```compile_fail
/// fn recover(s: &mut quent::PostgresSession) { let _ = &mut s.connection; }
/// ```
pub struct PostgresSession {
    connection: PgConnection,
    context: DatabaseContext,
    available: bool,
    identity: Arc<()>,
    #[cfg(test)]
    pub(super) final_now: Option<Instant>,
    #[cfg(test)]
    pub(super) final_barrier: Option<Arc<tokio::sync::Notify>>,
    #[cfg(test)]
    pub(super) recorder: Recorder,
}
impl fmt::Debug for PostgresSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PostgresSession([REDACTED])")
    }
}
impl PostgresSession {
    /// Explicit connection I/O. Statement and slow-query logging are disabled.
    pub async fn connect(options: PgConnectOptions, context: DatabaseContext) -> Result<Self> {
        if context.sensitive_source().target().3 != Dialect::Postgres {
            return Err(ExecutionError::ContextMismatch);
        }
        let options = options.disable_statement_logging();
        let connection = PgConnection::connect_with(&options)
            .await
            .map_err(|_| ExecutionError::Connect)?;
        Ok(Self {
            connection,
            context,
            available: true,
            identity: Arc::new(()),
            #[cfg(test)]
            final_now: None,
            #[cfg(test)]
            final_barrier: None,
            #[cfg(test)]
            recorder: Recorder::default(),
        })
    }
    immediate_dispatch!(Postgres, Postgres, PostgresMatched);
}
fn bind<'q>(
    query: &mut sqlx::query::Query<'q, Postgres, <Postgres as sqlx::Database>::Arguments>,
    value: &'q Value,
) -> Result<()> {
    let result = match value {
        Value::I16(v) => query.try_bind(v),
        Value::I32(v) => query.try_bind(v),
        Value::I64(v) => query.try_bind(v),
        Value::Bool(v) => query.try_bind(v),
        Value::Text(v) => query.try_bind(v),
        Value::Null(SqlType::I16) => query.try_bind(None::<i16>),
        Value::Null(SqlType::I32) => query.try_bind(None::<i32>),
        Value::Null(SqlType::I64) => query.try_bind(None::<i64>),
        Value::Null(SqlType::Bool) => query.try_bind(None::<bool>),
        Value::Null(SqlType::Text) => query.try_bind(None::<String>),
        _ => return Err(ExecutionError::Unsupported),
    };
    result.map_err(|_| ExecutionError::Encode)
}
fn decode_row(row: &PgRow, columns: &[(SqlType, bool)]) -> Result<Vec<Value>> {
    if row.len() != columns.len() {
        return Err(ExecutionError::Decode);
    }
    columns
        .iter()
        .enumerate()
        .map(|(i, &(ty, nullable))| {
            let info = row.column(i).type_info();
            let exact = match ty {
                SqlType::I16 => *info == <i16 as Type<Postgres>>::type_info(),
                SqlType::I32 => *info == <i32 as Type<Postgres>>::type_info(),
                SqlType::I64 => *info == <i64 as Type<Postgres>>::type_info(),
                SqlType::Bool => *info == <bool as Type<Postgres>>::type_info(),
                SqlType::Text => matches!(info.name(), "TEXT" | "VARCHAR"),
                _ => false,
            };
            if !exact {
                return Err(ExecutionError::Decode);
            }
            if row
                .try_get_raw(i)
                .map_err(|_| ExecutionError::Decode)?
                .is_null()
            {
                return if nullable {
                    Ok(Value::Null(ty))
                } else {
                    Err(ExecutionError::Decode)
                };
            }
            match ty {
                SqlType::I16 => row
                    .try_get::<i16, _>(i)
                    .map(Value::I16)
                    .map_err(|_| ExecutionError::Decode),
                SqlType::I32 => row
                    .try_get::<i32, _>(i)
                    .map(Value::I32)
                    .map_err(|_| ExecutionError::Decode),
                SqlType::I64 => row
                    .try_get::<i64, _>(i)
                    .map(Value::I64)
                    .map_err(|_| ExecutionError::Decode),
                SqlType::Bool => row
                    .try_get::<bool, _>(i)
                    .map(Value::Bool)
                    .map_err(|_| ExecutionError::Decode),
                SqlType::Text => row
                    .try_get::<String, _>(i)
                    .map(Value::Text)
                    .map_err(|_| ExecutionError::Decode),
                _ => Err(ExecutionError::Unsupported),
            }
        })
        .collect()
}
fn decode_count(row: &PgRow) -> Result<Value> {
    if *row.column(0).type_info() != <i64 as Type<Postgres>>::type_info() {
        return Err(ExecutionError::Decode);
    }
    let count = row
        .try_get::<i64, _>(0)
        .map_err(|_| ExecutionError::Decode)?;
    if count < 0 {
        return Err(ExecutionError::Decode);
    }
    Ok(Value::I64(count))
}
