use super::*;
use futures_util::TryStreamExt;
use sqlx::MySqlConnection;
use sqlx::mysql::MySqlConnectOptions;
use sqlx::{ConnectOptions, Connection};
/// Dedicated privately owned reader. No raw driver/executor access or writes.
pub struct MysqlReader {
    connection: Option<MySqlConnection>,
    context: DatabaseContext,
    target: Target,
    identity: Arc<()>,
    available: bool,
    #[cfg(test)]
    pub(super) trace: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    #[cfg(test)]
    pub(super) disconnect_after_header: bool,
}
redacted!(MysqlReader);
impl Drop for MysqlReader {
    fn drop(&mut self) {
        let dispatch = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        tracing::dispatcher::with_default(&dispatch, || drop(self.connection.take()));
    }
}
impl MysqlReader {
    /// Explicit connection only; caller supplies options, never a manifest DSN.
    pub async fn connect(
        options: MySqlConnectOptions,
        context: DatabaseContext,
        timeout: Duration,
    ) -> Result<Self> {
        let target = configured(&context, Dialect::Mysql, options.get_database())?;
        if timeout.is_zero() || timeout > Duration::from_secs(60) {
            return Err(VerifyError::Budget);
        }
        if target.address != format!("{}:{}", options.get_host(), options.get_port())
            || options.get_socket().is_some()
        {
            return Err(VerifyError::Target);
        }
        let options = options.disable_statement_logging();
        let connect_started = Instant::now();
        let connection = quiet(tokio::time::timeout(
            timeout,
            MySqlConnection::connect_with(&options),
        ))
        .await
        .map_err(|_| VerifyError::Expired)?
        .map_err(|_| VerifyError::Connect)?;
        let reader = Self {
            connection: Some(connection),
            context,
            target,
            identity: Arc::new(()),
            available: true,
            #[cfg(test)]
            trace: Arc::new(std::sync::Mutex::new(vec!["connected"])),
            #[cfg(test)]
            disconnect_after_header: false,
        };
        if connect_started.elapsed() >= timeout {
            return Err(VerifyError::Expired);
        }
        Ok(reader)
    }
    /// Finite catalog I/O. On any failure/cancellation this reader is quarantined.
    pub async fn observe(
        &mut self,
        current: &DatabaseContext,
        scope: &Scope,
        source: &ManifestSource,
        budget: Budget,
        ttl: Duration,
    ) -> Result<Observation> {
        let budget = inputs(&self.target, scope, source, budget)?;
        if !self.available || !self.context.same_snapshot(current) {
            return Err(VerifyError::Target);
        }
        let started = Instant::now();
        let deadline = expiry(started, ttl)?;
        let wall_started_ms = wall()?;
        let io_deadline = started
            .checked_add(budget.timeout)
            .ok_or(VerifyError::Clock)?;
        self.available = false;
        let connection = self.connection.as_mut().ok_or(VerifyError::Connect)?;
        let mut meter = Meter {
            b: budget,
            queries: 0,
            rows: 0,
            bytes: 0,
            #[cfg(test)]
            trace: self.trace.clone(),
            #[cfg(test)]
            disconnect_after_header: self.disconnect_after_header,
        };
        let observed = quiet(tokio::time::timeout(
            budget.timeout,
            read(connection, &self.target, scope, &mut meter, None),
        ))
        .await
        .map_err(|_| VerifyError::Expired)??;
        let completed = Instant::now();
        let wall_completed_ms = wall()?;
        if completed < started
            || completed >= deadline
            || completed >= io_deadline
            || wall_completed_ms < wall_started_ms
        {
            return Err(VerifyError::Expired);
        }
        let observation = Observation {
            context: current.clone(),
            identity: self.identity.clone(),
            scope: scope.clone(),
            source: source.clone(),
            observed,
            started,
            completed,
            deadline,
            wall_started_ms,
            wall_completed_ms,
        };
        let final_now = Instant::now();
        if final_now < completed {
            return Err(VerifyError::Clock);
        }
        if final_now >= deadline || final_now >= io_deadline {
            return Err(VerifyError::Expired);
        }
        self.available = true;
        Ok(observation)
    }
    /// Recheck actual in-process provenance, exact current inputs and expiry.
    pub fn compare(
        &self,
        current: &DatabaseContext,
        scope: &Scope,
        source: &ManifestSource,
        observation: &Observation,
    ) -> VerifySummary {
        if !self.available {
            return VerifySummary {
                verdict: Verdict::Unverified,
                coverage: SourceCoverage::Unknown,
                changes: [0; 5],
            };
        }
        observation.summary(current, &self.identity, scope, source)
    }
}
async fn rows(
    c: &mut MySqlConnection,
    sql: &'static str,
    namespace: &str,
    table: &str,
    m: &mut Meter,
) -> Result<Vec<Vec<Option<String>>>> {
    m.query()?;
    let mut stream = sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(sql.to_owned()))
        .bind(namespace)
        .bind(table)
        .fetch(&mut *c);
    let mut result = Vec::new();
    while let Some(row) = stream.try_next().await.map_err(|_| VerifyError::Io)? {
        result.push(m.row(&row)?);
    }
    drop(stream);
    m.event("catalog_complete");
    #[cfg(test)]
    if m.disconnect_after_header && m.queries == 2 {
        m.event("fixture_disconnect");
        let id = sqlx::query_scalar::<_, u64>("SELECT CONNECTION_ID()")
            .fetch_one(&mut *c)
            .await
            .map_err(|_| VerifyError::Io)?;
        let _ = sqlx::raw_sql(sqlx::AssertSqlSafe(format!("KILL CONNECTION {id}")))
            .execute(&mut *c)
            .await;
    }
    Ok(result)
}
const ID: &str = "SELECT CAST(JSON_ARRAY(DATABASE(),DATABASE(),VERSION(),CAST(@@show_gipk_in_create_table_and_information_schema AS CHAR),CAST(@@lower_case_table_names AS CHAR)) AS CHAR) WHERE ? IS NOT NULL AND ? IS NOT NULL LIMIT 2";
// Metadata visibility is conditional on explicit schema-wide SELECT and TRIGGER
// privileges. Inherited roles/column grants are conservatively not supported.
const HEADER: &str = "SELECT CAST(JSON_ARRAY(t.TABLE_TYPE,t.ENGINE,CAST((SELECT COUNT(*) FROM information_schema.SCHEMA_PRIVILEGES p WHERE p.TABLE_SCHEMA=t.TABLE_SCHEMA AND p.GRANTEE=CONCAT(CHAR(39),SUBSTRING_INDEX(CURRENT_USER(),'@',1),CHAR(39),'@',CHAR(39),SUBSTRING_INDEX(CURRENT_USER(),'@',-1),CHAR(39)) AND p.PRIVILEGE_TYPE IN ('SELECT','TRIGGER')) AS CHAR),CAST((SELECT COUNT(*) FROM information_schema.TRIGGERS g WHERE g.EVENT_OBJECT_SCHEMA=t.TABLE_SCHEMA AND g.EVENT_OBJECT_TABLE=t.TABLE_NAME) AS CHAR),CAST((SELECT COUNT(*) FROM information_schema.TABLE_CONSTRAINTS x WHERE x.TABLE_SCHEMA=t.TABLE_SCHEMA AND x.TABLE_NAME=t.TABLE_NAME AND x.CONSTRAINT_TYPE NOT IN ('PRIMARY KEY','UNIQUE')) AS CHAR),CAST((SELECT COUNT(*) FROM information_schema.PARTITIONS p WHERE p.TABLE_SCHEMA=t.TABLE_SCHEMA AND p.TABLE_NAME=t.TABLE_NAME AND p.PARTITION_NAME IS NOT NULL) AS CHAR)) AS CHAR) FROM information_schema.TABLES t WHERE t.TABLE_SCHEMA=? AND t.TABLE_NAME=? LIMIT 2";
const COLUMNS: &str = "SELECT CAST(JSON_ARRAY(LEFT(COLUMN_NAME,4097),LEFT(DATA_TYPE,4097),IS_NULLABLE,LEFT(COLUMN_DEFAULT,4097),LEFT(EXTRA,4097),LEFT(GENERATION_EXPRESSION,4097),COLLATION_NAME,CAST(ORDINAL_POSITION AS CHAR),LEFT(COLUMN_TYPE,4097)) AS CHAR) FROM information_schema.COLUMNS WHERE TABLE_SCHEMA=? AND TABLE_NAME=? ORDER BY ORDINAL_POSITION LIMIT 257";
const KEYS: &str = "SELECT CAST(JSON_ARRAY(LEFT(INDEX_NAME,4097),CAST(SEQ_IN_INDEX AS CHAR),LEFT(COLUMN_NAME,4097),LEFT(EXPRESSION,4097),CAST(SUB_PART AS CHAR),INDEX_TYPE,IS_VISIBLE,COLLATION,CAST(NON_UNIQUE AS CHAR)) AS CHAR) FROM information_schema.STATISTICS WHERE TABLE_SCHEMA=? AND TABLE_NAME=? ORDER BY INDEX_NAME,SEQ_IN_INDEX LIMIT 257";

// PR2-only finite extension. This proves no represented local indexes/constraints,
// not absence of every possible effect or an authenticated physical target.
const PREFLIGHT: &str = "SELECT CAST(JSON_ARRAY(LEFT(t.TABLE_SCHEMA,4097),LEFT(t.TABLE_NAME,4097),LEFT(CURRENT_USER(),4097),CAST((SELECT COUNT(*) FROM information_schema.STATISTICS i WHERE i.TABLE_SCHEMA=t.TABLE_SCHEMA AND i.TABLE_NAME=t.TABLE_NAME) AS CHAR),CAST((SELECT COUNT(*) FROM information_schema.TABLE_CONSTRAINTS x WHERE x.TABLE_SCHEMA=t.TABLE_SCHEMA AND x.TABLE_NAME=t.TABLE_NAME) AS CHAR),'false') AS CHAR) FROM information_schema.TABLES t WHERE t.TABLE_SCHEMA=? AND t.TABLE_NAME=? LIMIT 2";
async fn read(
    c: &mut MySqlConnection,
    target: &Target,
    scope: &Scope,
    m: &mut Meter,
    mut extension: Option<&mut Vec<Vec<Option<String>>>>,
) -> Result<Schema> {
    // MySQL information_schema metadata does NOT promise one MVCC DDL snapshot.
    // This read-only transaction is a bounded observation under a caller-enforced
    // no-concurrent-DDL condition, not a lock or lasting physical-schema guarantee.
    m.event("begin_attempt");
    let mut tx = c
        .begin_with("START TRANSACTION READ ONLY")
        .await
        .map_err(|_| VerifyError::Io)?;
    m.event("begun");
    let result = async {
        let id = rows(&mut tx, ID, &scope.namespace, "", m).await?;
        if id.len() != 1
            || field(&id[0], 0)? != target.catalog
            || field(&id[0], 1)? != scope.namespace
        {
            return Err(VerifyError::Target);
        }
        if field(&id[0], 2)? != "8.4.8" || field(&id[0], 3)? != "1" || field(&id[0], 4)? != "0" {
            return Err(VerifyError::Unsupported);
        }
        let mut schema = Schema {
            kind: SchemaKind::QuentSchema,
            version: 1,
            target: target.clone(),
            coverage: Coverage::Complete,
            tables: Vec::new(),
        };
        for name in &scope.tables {
            let header = rows(&mut tx, HEADER, &scope.namespace, name, m).await?;
            if header.len() != 1 {
                return Err(VerifyError::Unknown);
            }
            if let Some(extra) = extension.as_deref_mut() {
                let result = rows(&mut tx, PREFLIGHT, &scope.namespace, name, m).await?;
                if result.len() != 1 || result[0].len() != 6 {
                    return Err(VerifyError::Unknown);
                }
                extra.push(result.into_iter().next().ok_or(VerifyError::Unknown)?);
            }
            let h = &header[0];
            if field(h, 0)? != "BASE TABLE" {
                return Err(VerifyError::Unsupported);
            }
            if field(h, 2)? != "2" {
                return Err(VerifyError::Unknown);
            }
            let mut t = Table {
                path: vec![scope.namespace.clone(), name.clone()],
                semantics: Semantics::Ordinary,
                columns_coverage: Coverage::Complete,
                keys_coverage: Coverage::Complete,
                columns: Vec::new(),
                keys: Vec::new(),
            };
            if field(h, 0)? != "BASE TABLE"
                || field(h, 1)? != "InnoDB"
                || (3..6).any(|i| h.get(i).and_then(Option::as_deref) != Some("0"))
            {
                t.semantics = Semantics::Unsupported;
            }
            let columns = rows(&mut tx, COLUMNS, &scope.namespace, name, m).await?;
            if columns.is_empty() {
                return Err(VerifyError::Unknown);
            }
            if columns.len() > m.b.columns {
                return Err(VerifyError::Budget);
            }
            for (ordinal, v) in columns.into_iter().enumerate() {
                if extension.is_some()
                    && (v.len() != 9
                        || !matches!(field(&v, 2)?, "YES" | "NO")
                        || field(&v, 7)?.parse::<usize>().ok() != Some(ordinal + 1))
                {
                    return Err(VerifyError::Unknown);
                }
                let unsigned = field(&v, 8)?.contains(" unsigned");
                let native = match (field(&v, 1)?, unsigned) {
                    ("tinyint", false) => Native::I8,
                    ("smallint", false) => Native::I16,
                    ("int", false) => Native::I32,
                    ("bigint", false) => Native::I64,
                    ("tinyint", true) => Native::U8,
                    ("smallint", true) => Native::U16,
                    ("int", true) => Native::U32,
                    ("bigint", true) => Native::U64,
                    ("text", _) => Native::Text,
                    _ => Native::Unsupported,
                };
                let nullable = field(&v, 2)? == "YES";
                if field(&v, 8)?.contains("zerofill")
                    || !field(&v, 4)?.is_empty()
                    || !field(&v, 5)?.is_empty()
                    || v.get(6).and_then(Option::as_deref).is_some()
                {
                    t.semantics = Semantics::Unsupported;
                }
                let default =
                    integer_default(native, v.get(3).and_then(Option::as_deref), false, nullable)
                        .unwrap_or(DefaultValue::UnsupportedExpression {
                            expression: String::new(),
                        });
                t.columns.push(Column {
                    name: field(&v, 0)?.into(),
                    native,
                    nullability: if nullable {
                        Nullability::Nullable
                    } else {
                        Nullability::Nonnull
                    },
                    default: Some(default),
                });
            }
            let keys = rows(&mut tx, KEYS, &scope.namespace, name, m).await?;
            if keys.len() > 256 {
                return Err(VerifyError::Budget);
            }
            let mut last = String::new();
            for v in keys {
                if v.get(3).and_then(Option::as_deref).is_some()
                    || v.get(4).and_then(Option::as_deref).is_some()
                    || field(&v, 5)? != "BTREE"
                    || field(&v, 6)? != "YES"
                    || field(&v, 7)? != "A"
                {
                    t.semantics = Semantics::Unsupported;
                }
                if field(&v, 8)? != "0" {
                    continue;
                }
                let id = field(&v, 0)?;
                if id != last {
                    if t.keys.len() >= m.b.keys {
                        return Err(VerifyError::Budget);
                    }
                    t.keys.push(Key {
                        kind: if id == "PRIMARY" {
                            KeyKind::Primary
                        } else {
                            KeyKind::Unique
                        },
                        columns: Vec::new(),
                        coverage: Coverage::Complete,
                    });
                    last = id.into();
                }
                let k = t.keys.last_mut().ok_or(VerifyError::Unknown)?;
                if field(&v, 1)?.parse::<usize>().ok() != Some(k.columns.len() + 1)
                    || k.columns.len() >= 16
                {
                    return Err(VerifyError::Budget);
                }
                let Some(name) = v.get(2).and_then(Option::as_deref) else {
                    t.semantics = Semantics::Unsupported;
                    continue;
                };
                k.columns.push(name.into());
            }
            schema.tables.push(t);
        }
        if schema_coverage(&schema) == SourceCoverage::Complete {
            // A Complete observation must inhabit the same closed v1 model:
            // identifier grammar, nonempty/resolved keys and exact literal types.
            super::super::schema(&schema).map_err(|_| VerifyError::Unsupported)?;
        }
        normalize(&mut schema);
        Ok(schema)
    }
    .await;
    m.event("rollback_attempt");
    tx.rollback().await.map_err(|_| VerifyError::Io)?;
    m.event("rolled_back");
    result
}

preflight_reader!(MysqlReader);
