use super::*;
use futures_util::TryStreamExt;
use sqlx::mysql::{MySqlConnectOptions, MySqlRow};
use sqlx::{
    Column as _, ConnectOptions, Connection, MySql, MySqlConnection, Row, Type, TypeInfo, ValueRef,
};

/// Owns a connection and its full immutable application snapshot. No raw executor
/// getter, callbacks or pool routing is exposed. Finite owned atomic execution owns
/// transaction completion; borrowed adapters never do. Delayed handles
/// are private-record consumers bound to this exact connection lifetime.
/// Construction authenticates neither the application context nor physical DB.
/// ```compile_fail
/// fn bypass(s: &mut quent::MysqlSession, c: &quent_core::DatabaseContext, p: quent_core::Plan) { s.execute(c, p); }
/// ```
/// ```compile_fail
/// fn bypass(s: &mut quent::MysqlSession, c: &quent_core::DatabaseContext, p: quent_core::SensitiveSource<'_>) { s.execute(c, p); }
/// ```
/// ```compile_fail
/// fn bypass(s: &mut quent::MysqlSession, c: &quent_core::DatabaseContext) { s.execute(c, "DELETE FROM items"); }
/// ```
/// ```compile_fail
/// fn recover(s: &mut quent::MysqlSession) { let _ = &mut s.connection; }
/// ```
pub struct MysqlSession {
    connection: MySqlConnection,
    context: DatabaseContext,
    available: bool,
    identity: Arc<()>,
    result_state: ResultState,
    #[cfg(test)]
    pub(super) final_now: Option<Instant>,
    #[cfg(test)]
    pub(super) final_barrier: Option<Arc<tokio::sync::Notify>>,
    #[cfg(test)]
    pub(super) recorder: Recorder,
    #[cfg(test)]
    pub(super) atomic_fault: Option<AtomicFault>,
}
impl fmt::Debug for MysqlSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("MysqlSession([REDACTED])")
    }
}
impl MysqlSession {
    /// Explicit connection I/O. Statement and slow-query logging are disabled.
    pub async fn connect(options: MySqlConnectOptions, context: DatabaseContext) -> Result<Self> {
        if context.sensitive_source().target().3 != Dialect::Mysql {
            return Err(ExecutionError::ContextMismatch);
        }
        let options = options.disable_statement_logging();
        let connection = quiet(MySqlConnection::connect_with(&options))
            .await
            .map_err(|_| ExecutionError::Connect)?;
        Ok(Self {
            connection,
            context,
            available: true,
            identity: Arc::new(()),
            result_state: ResultState::Observed,
            #[cfg(test)]
            final_now: None,
            #[cfg(test)]
            final_barrier: None,
            #[cfg(test)]
            recorder: Recorder::default(),
            #[cfg(test)]
            atomic_fault: None,
        })
    }
    immediate_dispatch!(MySql, Mysql, MysqlFoundRows);
    compiled_dispatch!(Mysql);
    atomic_dispatch!(MySql, Mysql, BorrowedMysqlTransaction);
}
fn bind<'q>(
    query: &mut sqlx::query::Query<'q, MySql, <MySql as sqlx::Database>::Arguments>,
    value: &'q Value,
) -> Result<()> {
    let result = match value {
        Value::I8(v) => query.try_bind(v),
        Value::I16(v) => query.try_bind(v),
        Value::I32(v) => query.try_bind(v),
        Value::I64(v) => query.try_bind(v),
        Value::U8(v) => query.try_bind(v),
        Value::U16(v) => query.try_bind(v),
        Value::U32(v) => query.try_bind(v),
        Value::U64(v) => query.try_bind(v),
        Value::Bool(v) => query.try_bind(v),
        Value::Text(v) => query.try_bind(v),
        Value::Null(SqlType::I8) => query.try_bind(None::<i8>),
        Value::Null(SqlType::I16) => query.try_bind(None::<i16>),
        Value::Null(SqlType::I32) => query.try_bind(None::<i32>),
        Value::Null(SqlType::I64) => query.try_bind(None::<i64>),
        Value::Null(SqlType::U8) => query.try_bind(None::<u8>),
        Value::Null(SqlType::U16) => query.try_bind(None::<u16>),
        Value::Null(SqlType::U32) => query.try_bind(None::<u32>),
        Value::Null(SqlType::U64) => query.try_bind(None::<u64>),
        Value::Null(SqlType::Bool) => query.try_bind(None::<bool>),
        Value::Null(SqlType::Text) => query.try_bind(None::<String>),
    };
    result.map_err(|_| ExecutionError::Encode)
}
fn decode_row(row: &MySqlRow, columns: &[(SqlType, bool)]) -> Result<Vec<Value>> {
    if row.len() != columns.len() {
        return Err(ExecutionError::Decode);
    }
    columns
        .iter()
        .enumerate()
        .map(|(i, &(ty, nullable))| {
            let info = row.column(i).type_info();
            let exact = match ty {
                SqlType::I8 => *info == <i8 as Type<MySql>>::type_info(),
                SqlType::I16 => *info == <i16 as Type<MySql>>::type_info(),
                SqlType::I32 => *info == <i32 as Type<MySql>>::type_info(),
                SqlType::I64 => *info == <i64 as Type<MySql>>::type_info(),
                SqlType::U8 => *info == <u8 as Type<MySql>>::type_info(),
                SqlType::U16 => *info == <u16 as Type<MySql>>::type_info(),
                SqlType::U32 => *info == <u32 as Type<MySql>>::type_info(),
                SqlType::U64 => *info == <u64 as Type<MySql>>::type_info(),
                SqlType::Bool => {
                    info.name() == "BOOLEAN" && *info == <i8 as Type<MySql>>::type_info()
                }
                SqlType::Text => {
                    matches!(info.name(), "TEXT" | "BLOB" | "VARCHAR" | "VARBINARY")
                        && <String as Type<MySql>>::compatible(info)
                }
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
                SqlType::I8 => row
                    .try_get::<i8, _>(i)
                    .map(Value::I8)
                    .map_err(|_| ExecutionError::Decode),
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
                SqlType::U8 => row
                    .try_get::<u8, _>(i)
                    .map(Value::U8)
                    .map_err(|_| ExecutionError::Decode),
                SqlType::U16 => row
                    .try_get::<u16, _>(i)
                    .map(Value::U16)
                    .map_err(|_| ExecutionError::Decode),
                SqlType::U32 => row
                    .try_get::<u32, _>(i)
                    .map(Value::U32)
                    .map_err(|_| ExecutionError::Decode),
                SqlType::U64 => row
                    .try_get::<u64, _>(i)
                    .map(Value::U64)
                    .map_err(|_| ExecutionError::Decode),
                SqlType::Bool => match row
                    .try_get::<i8, _>(i)
                    .map_err(|_| ExecutionError::Decode)?
                {
                    0 => Ok(Value::Bool(false)),
                    1 => Ok(Value::Bool(true)),
                    _ => Err(ExecutionError::Decode),
                },
                SqlType::Text => row
                    .try_get::<String, _>(i)
                    .map(Value::Text)
                    .map_err(|_| ExecutionError::Decode),
            }
        })
        .collect()
}
fn decode_count(row: &MySqlRow) -> Result<Value> {
    if *row.column(0).type_info() == <u64 as Type<MySql>>::type_info() {
        return row
            .try_get::<u64, _>(0)
            .map(Value::U64)
            .map_err(|_| ExecutionError::Decode);
    }
    if *row.column(0).type_info() != <i64 as Type<MySql>>::type_info() {
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

impl NativeConnection for MySqlConnection {
    type Connection = Self;
    fn native(&mut self) -> &mut Self {
        self
    }
}
borrowed_dispatch!(
    BorrowedMysqlTransaction,
    MySql,
    MySqlConnection,
    Mysql,
    MysqlFoundRows
);
