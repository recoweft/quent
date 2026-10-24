//! Explicit, read-only catalog observations. No execution authority is produced.
use super::*;
use crate::execution::quiet_driver::quiet;
use quent_core::DatabaseContext;
use std::{
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Closed failures; driver errors, identifiers and configuration are discarded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyError {
    Invalid,
    Target,
    Budget,
    Clock,
    Expired,
    Connect,
    Io,
    Missing,
    Unknown,
    Unsupported,
}
impl fmt::Display for VerifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_VERIFY_{self:?}")
    }
}
impl std::error::Error for VerifyError {}
type Result<T> = std::result::Result<T, VerifyError>;
/// All limits are explicit; values exceeding the fixed ceilings are refused.
#[derive(Debug, Clone, Copy)]
pub struct Budget {
    pub queries: usize,
    pub rows: usize,
    pub bytes: usize,
    pub string_bytes: usize,
    pub tables: usize,
    pub columns: usize,
    pub keys: usize,
    pub branches: usize,
    pub timeout: Duration,
}
impl Budget {
    fn check(self) -> Result<Self> {
        if self.queries == 0
            || self.queries > 65
            || self.rows == 0
            || self.rows > 8192
            || self.bytes == 0
            || self.bytes > 1_048_576
            || self.string_bytes == 0
            || self.string_bytes > 4096
            || self.tables == 0
            || self.tables > 16
            || self.columns == 0
            || self.columns > 256
            || self.keys == 0
            || self.keys > 16
            || self.branches == 0
            || self.branches > 256
            || self.timeout.is_zero()
            || self.timeout > Duration::from_secs(60)
        {
            return Err(VerifyError::Budget);
        }
        Ok(self)
    }
}
/// Exact explicit scope; names are sensitive. No scope is inferred from a source.
#[derive(Clone, PartialEq, Eq)]
pub struct Scope {
    namespace: String,
    tables: Vec<String>,
}
impl Scope {
    pub fn new(namespace: &str, tables: &[&str]) -> Result<Self> {
        fn name(s: &str) -> bool {
            !s.is_empty()
                && s.len() <= 63
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        }
        if !name(namespace)
            || tables.is_empty()
            || tables.len() > 16
            || tables.iter().any(|s| !name(s))
        {
            return Err(VerifyError::Invalid);
        }
        let mut names: Vec<_> = tables.iter().map(|s| (*s).to_owned()).collect();
        names.sort();
        if names.windows(2).any(|w| w[0] == w[1]) {
            return Err(VerifyError::Invalid);
        }
        Ok(Self {
            namespace: namespace.into(),
            tables: names,
        })
    }
}
redacted!(Scope);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Matched,
    Drift,
    Missing,
    Unverified,
    Stale,
    Unknown,
    Unsupported,
}
/// Output-only schema result. Other domains remain unverified even after a match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifySummary {
    verdict: Verdict,
    changes: [usize; 5],
    coverage: SourceCoverage,
}
impl VerifySummary {
    pub fn verdict(&self) -> Verdict {
        self.verdict
    }
    pub fn coverage(&self) -> SourceCoverage {
        self.coverage
    }
    /// Table, column, native, null/default and ordered-key difference counts.
    pub fn counts(&self) -> [usize; 5] {
        self.changes
    }
    pub fn to_json(&self) -> String {
        format!(
            "{{\"kind\":\"quent_verify_summary\",\"version\":1,\"schema\":\"{:?}\",\"coverage\":\"{:?}\",\"changes\":{:?},\"policy\":\"Unverified\",\"generated_code\":\"Unverified\",\"external_live\":\"Unverified\"}}",
            self.verdict, self.coverage, self.changes
        )
    }
}
impl fmt::Display for VerifySummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
/// Private provenance, minted only after real complete catalog I/O and rollback.
/// Never serializable or convertible into application facts or execution handles.
pub struct Observation {
    context: DatabaseContext,
    identity: Arc<()>,
    scope: Scope,
    source: ManifestSource,
    observed: Schema,
    started: Instant,
    completed: Instant,
    deadline: Instant,
    wall_started_ms: u64,
    wall_completed_ms: u64,
}
redacted!(Observation);
fn wall() -> Result<u64> {
    u64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| VerifyError::Clock)?
            .as_millis(),
    )
    .map_err(|_| VerifyError::Clock)
}
fn expiry(start: Instant, ttl: Duration) -> Result<Instant> {
    if ttl.is_zero() || ttl > Duration::from_secs(300) {
        return Err(VerifyError::Expired);
    }
    start.checked_add(ttl).ok_or(VerifyError::Clock)
}
impl Observation {
    fn summary(
        &self,
        context: &DatabaseContext,
        identity: &Arc<()>,
        scope: &Scope,
        source: &ManifestSource,
    ) -> VerifySummary {
        let mut result = VerifySummary {
            verdict: Verdict::Unverified,
            changes: [0; 5],
            coverage: schema_coverage(&self.observed),
        };
        if !Arc::ptr_eq(identity, &self.identity)
            || !context.same_snapshot(&self.context)
            || scope != &self.scope
            || source.source != self.source.source
        {
            return result;
        }
        let now = Instant::now();
        if now < self.completed
            || self.completed < self.started
            || now >= self.deadline
            || match wall() {
                Ok(w) => {
                    w < self.wall_completed_ms || self.wall_completed_ms < self.wall_started_ms
                }
                Err(_) => true,
            }
        {
            result.verdict = Verdict::Stale;
            return result;
        }
        let Some(expected) = &source.source.schema else {
            result.verdict = Verdict::Missing;
            return result;
        };
        let mut expected = expected.clone();
        normalize(&mut expected);
        let ec = schema_coverage(&expected);
        result.coverage = if ec == SourceCoverage::Unsupported
            || result.coverage == SourceCoverage::Unsupported
        {
            SourceCoverage::Unsupported
        } else if ec == SourceCoverage::Unknown || result.coverage == SourceCoverage::Unknown {
            SourceCoverage::Unknown
        } else if ec == SourceCoverage::Partial || result.coverage == SourceCoverage::Partial {
            SourceCoverage::Partial
        } else {
            SourceCoverage::Complete
        };
        result.verdict = if ec == SourceCoverage::Unsupported
            || result.coverage == SourceCoverage::Unsupported
        {
            Verdict::Unsupported
        } else if ec != SourceCoverage::Complete || result.coverage != SourceCoverage::Complete {
            Verdict::Unknown
        } else if canonical(&expected) == canonical(&self.observed) {
            Verdict::Matched
        } else {
            Verdict::Drift
        };
        // Bounded by validated schema/scope ceilings. No names or old/new values.
        for t in &expected.tables {
            let Some(o) = self.observed.tables.iter().find(|o| o.path == t.path) else {
                result.changes[0] += 1;
                continue;
            };
            result.changes[1] += usize::from(
                t.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
                    != o.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
                    && t.columns.len() == o.columns.len(),
            );
            for c in &t.columns {
                if let Some(d) = o.columns.iter().find(|d| d.name == c.name) {
                    result.changes[2] += usize::from(c.native != d.native);
                    result.changes[3] +=
                        usize::from(c.nullability != d.nullability || c.default != d.default);
                } else {
                    result.changes[1] += 1;
                }
            }
            result.changes[1] += o
                .columns
                .iter()
                .filter(|d| !t.columns.iter().any(|c| c.name == d.name))
                .count();
            result.changes[4] += usize::from(t.keys != o.keys);
        }
        result.changes[0] += self
            .observed
            .tables
            .iter()
            .filter(|o| !expected.tables.iter().any(|t| t.path == o.path))
            .count();
        // Canonical comparison and bounded drift work consume the same TTL.
        let wall_invalid = match wall() {
            Ok(w) => w < self.wall_completed_ms,
            Err(_) => true,
        };
        let final_now = Instant::now();
        if wall_invalid || final_now < self.completed || final_now >= self.deadline {
            result.verdict = Verdict::Stale;
        }
        result
    }
}
fn normalize(s: &mut Schema) {
    s.tables.sort_by(|a, b| a.path.cmp(&b.path));
    for t in &mut s.tables {
        t.keys.sort_by_key(|k| {
            (
                match k.kind {
                    KeyKind::Primary => 0,
                    KeyKind::Unique => 1,
                },
                k.columns.clone(),
            )
        });
    }
    // Column order and the order inside each composite key are never sorted.
}
fn configured(
    context: &DatabaseContext,
    dialect: Dialect,
    database: Option<&str>,
) -> Result<Target> {
    let s = context.sensitive_source();
    let (db, address, catalog, d) = s.target();
    let actual = match d {
        quent_core::Dialect::Postgres => Dialect::Postgres,
        quent_core::Dialect::Mysql => Dialect::Mysql,
    };
    if actual != dialect || database != Some(catalog.sensitive_text()) {
        return Err(VerifyError::Target);
    }
    Ok(Target {
        database: db.into(),
        address: address.into(),
        catalog: catalog.sensitive_text().into(),
        dialect,
    })
}
fn inputs(
    target: &Target,
    scope: &Scope,
    source: &ManifestSource,
    budget: Budget,
) -> Result<Budget> {
    let b = budget.check()?;
    if scope.tables.len() > b.tables || scope.tables.len() > b.branches {
        return Err(VerifyError::Budget);
    }
    if target.dialect == Dialect::Mysql && scope.namespace != target.catalog {
        return Err(VerifyError::Target);
    }
    let s = source.source.schema.as_ref().ok_or(VerifyError::Missing)?;
    if s.target != *target
        || s.tables.len() != scope.tables.len()
        || s.tables.iter().any(|t| {
            t.path.len() != 2 || t.path[0] != scope.namespace || !scope.tables.contains(&t.path[1])
        })
    {
        return Err(VerifyError::Target);
    }
    Ok(b)
}
struct Meter {
    b: Budget,
    queries: usize,
    rows: usize,
    bytes: usize,
    #[cfg(test)]
    trace: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    #[cfg(test)]
    disconnect_after_header: bool,
}
impl Meter {
    fn event(&self, event: &'static str) {
        #[cfg(test)]
        self.trace.lock().unwrap().push(event);
        #[cfg(not(test))]
        let _ = event;
    }
    fn query(&mut self) -> Result<()> {
        self.queries += 1;
        if self.queries > self.b.queries {
            Err(VerifyError::Budget)
        } else {
            self.event("catalog_select");
            Ok(())
        }
    }
    fn row(&mut self, text: &str) -> Result<Vec<Option<String>>> {
        self.rows = self.rows.checked_add(1).ok_or(VerifyError::Budget)?;
        self.bytes = self
            .bytes
            .checked_add(text.len())
            .ok_or(VerifyError::Budget)?;
        if self.rows > self.b.rows || self.bytes > self.b.bytes || text.len() > 65_536 {
            return Err(VerifyError::Budget);
        }
        let values: Vec<Option<String>> =
            serde_json::from_str(text).map_err(|_| VerifyError::Io)?;
        if values.len() > 16
            || values
                .iter()
                .flatten()
                .any(|s| s.len() > self.b.string_bytes)
        {
            return Err(VerifyError::Budget);
        }
        Ok(values)
    }
}
fn field(v: &[Option<String>], n: usize) -> Result<&str> {
    v.get(n)
        .and_then(Option::as_deref)
        .ok_or(VerifyError::Unknown)
}
fn integer_default(
    native: Native,
    raw: Option<&str>,
    pg: bool,
    nullable: bool,
) -> Option<DefaultValue> {
    let Some(raw) = raw else {
        return Some(if pg || nullable {
            DefaultValue::Unknown {}
        } else {
            DefaultValue::None {}
        });
    };
    let raw = if pg {
        raw.strip_prefix('(')
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or(raw)
    } else {
        raw
    };
    let raw = if pg && raw.starts_with('\'') {
        if !matches!(native, Native::I16 | Native::I32 | Native::I64) {
            return None;
        }
        let body = raw.strip_prefix('\'')?;
        // PG deparses a smallint literal's expression as an integer Const;
        // accept only these built-in integer casts and checked enclosing width.
        ["'::smallint", "'::integer", "'::bigint"]
            .iter()
            .find_map(|suffix| body.strip_suffix(suffix))?
    } else {
        raw
    };
    if raw.is_empty()
        || !raw
            .bytes()
            .enumerate()
            .all(|(i, b)| b.is_ascii_digit() || (i == 0 && b == b'-'))
    {
        return None;
    }
    let value = match native {
        Native::I8 => Literal::I8(raw.parse().ok()?),
        Native::I16 => Literal::I16(raw.parse().ok()?),
        Native::I32 => Literal::I32(raw.parse().ok()?),
        Native::I64 => Literal::I64(raw.parse().ok()?),
        Native::U8 => Literal::U8(raw.parse().ok()?),
        Native::U16 => Literal::U16(raw.parse().ok()?),
        Native::U32 => Literal::U32(raw.parse().ok()?),
        Native::U64 => Literal::U64(raw.parse().ok()?),
        _ => return None,
    };
    Some(DefaultValue::Literal { value })
}
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "postgres")]
pub use postgres::PostgresReader;
#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "mysql")]
pub use mysql::MysqlReader;
#[cfg(test)]
mod tests;
