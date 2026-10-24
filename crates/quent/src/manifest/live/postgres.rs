use super::*;
use futures_util::TryStreamExt;
use sqlx::PgConnection;
use sqlx::postgres::PgConnectOptions;
use sqlx::{ConnectOptions, Connection};
/// Dedicated privately owned reader. No raw driver/executor access or writes.
pub struct PostgresReader {
    connection: Option<PgConnection>,
    context: DatabaseContext,
    target: Target,
    identity: Arc<()>,
    available: bool,
    #[cfg(test)]
    pub(super) trace: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    #[cfg(test)]
    pub(super) disconnect_after_header: bool,
}
redacted!(PostgresReader);
impl Drop for PostgresReader {
    fn drop(&mut self) {
        let dispatch = tracing::Dispatch::new(tracing::subscriber::NoSubscriber::default());
        tracing::dispatcher::with_default(&dispatch, || drop(self.connection.take()));
    }
}
impl PostgresReader {
    /// Explicit connection only; caller supplies options, never a manifest DSN.
    pub async fn connect(
        options: PgConnectOptions,
        context: DatabaseContext,
        timeout: Duration,
    ) -> Result<Self> {
        let target = configured(&context, Dialect::Postgres, options.get_database())?;
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
            PgConnection::connect_with(&options),
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
            read(connection, &self.target, scope, &mut meter),
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
    c: &mut PgConnection,
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
        let _ = sqlx::query("SELECT pg_catalog.pg_terminate_backend(pg_catalog.pg_backend_pid())")
            .execute(&mut *c)
            .await;
    }
    Ok(result)
}
// Every query has a fixed LIMIT, and every variable catalog string is clipped to
// a sentinel byte/character bound and then refused by Meter. No user-row reads.
const ID: &str = "SELECT json_build_array(current_database(),current_schema(),current_setting('server_version_num'))::text WHERE $1::text IS NOT NULL AND $2::text IS NOT NULL LIMIT 2";
const HEADER: &str = r#"
SELECT pg_catalog.json_build_array(
 c.relkind::text,pg_catalog.has_schema_privilege(n.oid,'USAGE')::text,
 pg_catalog.has_table_privilege(c.oid,'SELECT')::text,
 c.relrowsecurity::text,c.relforcerowsecurity::text,
 EXISTS(SELECT 1 FROM pg_catalog.pg_trigger t WHERE t.tgrelid=c.oid)::text,
 EXISTS(SELECT 1 FROM pg_catalog.pg_constraint x WHERE x.conrelid=c.oid AND x.contype NOT IN ('p','u'))::text,
 EXISTS(SELECT 1 FROM pg_catalog.pg_inherits i WHERE i.inhrelid=c.oid OR i.inhparent=c.oid)::text,
 EXISTS(SELECT 1 FROM pg_catalog.pg_rewrite r WHERE r.ev_class=c.oid)::text,
 (c.relam<>(SELECT oid FROM pg_catalog.pg_am WHERE amname='heap') OR
  EXISTS(SELECT 1 FROM pg_catalog.pg_index i
   JOIN pg_catalog.pg_class ic ON ic.oid=i.indexrelid
   JOIN pg_catalog.pg_am am ON am.oid=ic.relam
   WHERE i.indrelid=c.oid AND
    (i.indexprs IS NOT NULL OR i.indpred IS NOT NULL OR NOT i.indisvalid OR NOT i.indimmediate
     OR am.amname<>'btree' OR i.indnkeyatts<>i.indnatts
     OR EXISTS(SELECT 1 FROM pg_catalog.unnest(i.indclass) op(oid)
      JOIN pg_catalog.pg_opclass oc ON oc.oid=op.oid
      WHERE NOT oc.opcdefault OR oc.opcnamespace<>'pg_catalog'::regnamespace
       OR oc.opcname NOT IN ('int2_ops','int4_ops','int8_ops','bool_ops')))))::text
)::text FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace
WHERE n.nspname=$1 AND c.relname=$2 LIMIT 2
"#;
const COLUMNS: &str = "SELECT json_build_array(left(a.attname,4097),CASE WHEN t.typnamespace='pg_catalog'::regnamespace THEN t.typname ELSE 'unsupported' END,a.attnotnull::text,CASE WHEN d.adbin IS NULL THEN NULL WHEN pg_catalog.octet_length(d.adbin::text)<=4096 AND NOT EXISTS(SELECT 1 FROM pg_catalog.regexp_matches(d.adbin::text, ':consttype ([0-9]+)', 'g') AS m(type_oid) WHERE m.type_oid[1] NOT IN ('16','20','21','23')) THEN left(pg_catalog.pg_get_expr(d.adbin,d.adrelid),4097) ELSE 'unsupported_default' END,a.attgenerated::text,a.attidentity::text,a.attcollation::text,a.attnum::text)::text FROM pg_catalog.pg_attribute a JOIN pg_catalog.pg_class c ON c.oid=a.attrelid JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace JOIN pg_catalog.pg_type t ON t.oid=a.atttypid LEFT JOIN pg_catalog.pg_attrdef d ON d.adrelid=a.attrelid AND d.adnum=a.attnum WHERE n.nspname=$1 AND c.relname=$2 AND a.attnum>0 AND NOT a.attisdropped ORDER BY a.attnum LIMIT 257";
const KEYS: &str = "SELECT json_build_array(i.indexrelid::text,i.indisprimary::text,k.ordinality::text,a.attname,i.indisunique::text)::text FROM pg_catalog.pg_index i JOIN pg_catalog.pg_class c ON c.oid=i.indrelid JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace CROSS JOIN LATERAL unnest(i.indkey) WITH ORDINALITY k(attnum,ordinality) LEFT JOIN pg_catalog.pg_attribute a ON a.attrelid=c.oid AND a.attnum=k.attnum WHERE n.nspname=$1 AND c.relname=$2 AND i.indisunique ORDER BY i.indisprimary DESC,i.indexrelid,k.ordinality LIMIT 257";
async fn read(
    c: &mut PgConnection,
    target: &Target,
    scope: &Scope,
    m: &mut Meter,
) -> Result<Schema> {
    // PostgreSQL catalogs use one REPEATABLE READ snapshot, no DDL locks claimed.
    m.event("begin_attempt");
    let mut tx = c
        .begin_with("BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
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
        if field(&id[0], 2)? != "160014" {
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
            let h = &header[0];
            if field(h, 0)? != "r" {
                return Err(VerifyError::Unsupported);
            }
            if field(h, 1)? != "true" || field(h, 2)? != "true" {
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
            if field(h, 0)? != "r"
                || (3..10).any(|i| h.get(i).and_then(Option::as_deref) != Some("false"))
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
            for v in columns {
                let native = match field(&v, 1)? {
                    "int2" => Native::I16,
                    "int4" => Native::I32,
                    "int8" => Native::I64,
                    "bool" => Native::Bool,
                    "text" => Native::Text,
                    _ => Native::Unsupported,
                };
                let nullable = field(&v, 2)? == "false";
                if !field(&v, 4)?.is_empty() || !field(&v, 5)?.is_empty() || field(&v, 6)? != "0" {
                    t.semantics = Semantics::Unsupported;
                }
                let raw = v.get(3).and_then(Option::as_deref);
                let default = integer_default(native, raw, true, nullable)
                    .or(match (native, raw) {
                        (Native::Bool, Some("true")) => Some(DefaultValue::Literal {
                            value: Literal::Bool(true),
                        }),
                        (Native::Bool, Some("false")) => Some(DefaultValue::Literal {
                            value: Literal::Bool(false),
                        }),
                        _ => None,
                    })
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
            append_keys(&mut t, keys, m.b.keys)?;
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
    // No successful observation until cleanup is acknowledged; no commit or writes.
    m.event("rollback_attempt");
    tx.rollback().await.map_err(|_| VerifyError::Io)?;
    m.event("rolled_back");
    result
}
fn append_keys(t: &mut Table, rows: Vec<Vec<Option<String>>>, max: usize) -> Result<()> {
    let mut last = String::new();
    for v in rows {
        let id = field(&v, 0)?;
        if id != last {
            if t.keys.len() >= max {
                return Err(VerifyError::Budget);
            }
            t.keys.push(Key {
                kind: if field(&v, 1)? == "true" {
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
        if field(&v, 2)?.parse::<usize>().ok() != Some(k.columns.len() + 1) || k.columns.len() >= 16
        {
            return Err(VerifyError::Budget);
        }
        let Some(name) = v.get(3).and_then(Option::as_deref) else {
            t.semantics = Semantics::Unsupported;
            continue;
        };
        if !t.columns.iter().any(|c| c.name == name) {
            t.semantics = Semantics::Unsupported;
            continue;
        }
        k.columns.push(name.into());
    }
    Ok(())
}
