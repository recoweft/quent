//! Explicit estimated evidence. No observation is an execution handle or permit.
use super::*;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::Value as Json;
use std::collections::BTreeSet;

pub(super) const MAX_BYTES: usize = 65_536;
pub(super) type PerfResult<T> = std::result::Result<T, PerformanceError>;
/// Closed, input-free failure register. No driver/parser cause is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceError {
    Execution(ExecutionError),
    Unsupported,
    Version,
    Metadata,
    Decode,
    Budget,
    Timeout,
    Read,
}
impl From<ExecutionError> for PerformanceError {
    fn from(e: ExecutionError) -> Self {
        Self::Execution(e)
    }
}
impl fmt::Display for PerformanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "QUENT_PERFORMANCE_{self:?}")
    }
}
impl std::error::Error for PerformanceError {}
/// Finite observation budgets; these bound application consumption, not a server
/// packet allocation, optimizer work, lock duration or connection initialization.
#[derive(Clone, Copy)]
pub struct PerformanceBudget {
    pub(super) bytes: usize,
    pub(super) nodes: usize,
    pub(super) timeout: Duration,
}
impl PerformanceBudget {
    pub fn new(bytes: usize, nodes: usize, timeout: Duration) -> PerfResult<Self> {
        if !(1..=MAX_BYTES).contains(&bytes)
            || !(1..=64).contains(&nodes)
            || timeout.is_zero()
            || timeout > Duration::from_secs(10)
        {
            return Err(PerformanceError::Budget);
        }
        Ok(Self {
            bytes,
            nodes,
            timeout,
        })
    }
}
impl fmt::Debug for PerformanceBudget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PerformanceBudget([REDACTED])")
    }
}
/// A purpose-specific, consumed request; ordinary validated/authorized handles
/// have no conversion. Private final record retains all native args and metadata.
/// ```compile_fail
/// fn duplicate(r: quent::PerformanceRequest) { let _ = r.clone(); }
/// ```
/// ```compile_fail
/// let _: quent::PerformanceRequest = serde_json::from_str("{}").unwrap();
/// ```
pub struct PerformanceRequest {
    pub(super) operation: ValidatedOperation,
    pub(super) budget: PerformanceBudget,
}
impl fmt::Debug for PerformanceRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PerformanceRequest([REDACTED])")
    }
}
/// Private observation, never Fresh/authorization or a reusable dispatch handle.
/// No raw EXPLAIN getter/export, serde, Display, Clone or execution conversion.
/// ```compile_fail
/// fn export(p: &quent::PerformancePlan) { serde_json::to_string(p).unwrap(); }
/// ```
/// ```compile_fail
/// let _: quent::PerformancePlan = serde_json::from_str("{}").unwrap();
/// ```
pub struct PerformancePlan {
    pub(super) _request: PerformanceRequest,
    pub(super) _raw: Vec<u8>,
    pub(super) view: PerformanceView,
}
impl fmt::Debug for PerformancePlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PerformancePlan([REDACTED])")
    }
}
/// Output-only, fixed coverage: finite structural parsing is always Partial.
/// Counts disclose no SQL/values/names, estimates, timing, role or target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerformanceView {
    nodes: u8,
}
impl PerformanceView {
    pub fn to_json(&self) -> String {
        format!(
            "{{\"kind\":\"quent_performance_view\",\"version\":1,\"state\":\"Observed\",\"coverage\":\"Partial\",\"precision\":\"Estimated\",\"nodes\":{},\"auxiliary_submissions\":1,\"explain_submissions\":1,\"business_submissions\":0,\"provider_calls\":0,\"tx_submissions\":0}}",
            self.nodes
        )
    }
}
impl fmt::Display for PerformanceView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_json())
    }
}
impl PerformancePlan {
    pub fn public_view(&self) -> PerformanceView {
        self.view
    }
}

// First pass bounds every token and rejects duplicates before Value can erase
// them. This is a DB-result parser, never a public authority/input wire parser.
struct Scan<'a> {
    depth: usize,
    remaining: &'a mut usize,
}
impl<'de> DeserializeSeed<'de> for Scan<'_> {
    type Value = ();
    fn deserialize<D: de::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        if self.depth > 16 || *self.remaining == 0 {
            return Err(de::Error::custom("budget"));
        }
        *self.remaining -= 1;
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Scan<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded result")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, v: i64) -> std::result::Result<(), E> {
        if v < 0 {
            Err(E::custom("negative"))
        } else {
            self.visit_u64(v as u64)
        }
    }
    fn visit_u64<E: de::Error>(self, v: u64) -> std::result::Result<(), E> {
        if v > 1_000_000_000_000 {
            Err(E::custom("number budget"))
        } else {
            Ok(())
        }
    }
    fn visit_f64<E: de::Error>(self, v: f64) -> std::result::Result<(), E> {
        if !v.is_finite() || !(0.0..=1e12).contains(&v) {
            Err(E::custom("number budget"))
        } else {
            Ok(())
        }
    }
    fn visit_str<E: de::Error>(self, v: &str) -> std::result::Result<(), E> {
        if v.len() > 4096 {
            Err(E::custom("string budget"))
        } else {
            Ok(())
        }
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> std::result::Result<(), A::Error> {
        let mut n = 0;
        while a
            .next_element_seed(Scan {
                depth: self.depth + 1,
                remaining: self.remaining,
            })?
            .is_some()
        {
            n += 1;
            if n > 64 {
                return Err(de::Error::custom("children budget"));
            }
        }
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> std::result::Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(k) = a.next_key::<String>()? {
            if k.len() > 128 || keys.len() >= 64 || !keys.insert(k) {
                return Err(de::Error::custom("key budget"));
            }
            a.next_value_seed(Scan {
                depth: self.depth + 1,
                remaining: self.remaining,
            })?;
        }
        Ok(())
    }
}
pub(super) fn version(bytes: &[u8], dialect: Dialect) -> PerfResult<()> {
    if bytes.len() > 64 {
        return Err(PerformanceError::Budget);
    }
    let expected: &[u8] = if dialect == Dialect::Postgres {
        b"160014"
    } else {
        b"8.4.8"
    };
    if bytes != expected {
        return Err(PerformanceError::Version);
    }
    Ok(())
}
fn number(v: Option<&Json>) -> PerfResult<f64> {
    let n = match v {
        Some(Json::Number(n)) => n.as_f64(),
        // MySQL estimates/costs use JSON strings; accept only bounded plain decimals.
        Some(Json::String(s))
            if !s.is_empty()
                && s.len() <= 32
                && s.bytes().all(|c| c.is_ascii_digit() || c == b'.') =>
        {
            s.parse().ok()
        }
        _ => None,
    }
    .ok_or(PerformanceError::Decode)?;
    if !n.is_finite() || !(0.0..=1e12).contains(&n) {
        return Err(PerformanceError::Decode);
    }
    Ok(n)
}
fn pg_node(v: &Json, depth: usize, nodes: &mut usize, max: usize) -> PerfResult<()> {
    if depth > 8 || *nodes == max {
        return Err(PerformanceError::Budget);
    }
    let obj = v.as_object().ok_or(PerformanceError::Decode)?;
    let kind = obj
        .get("Node Type")
        .and_then(Json::as_str)
        .ok_or(PerformanceError::Decode)?;
    if !matches!(
        kind,
        "Seq Scan"
            | "Index Scan"
            | "Index Only Scan"
            | "Bitmap Heap Scan"
            | "Bitmap Index Scan"
            | "Limit"
            | "Aggregate"
            | "Result"
            | "Sort"
            | "BitmapAnd"
            | "BitmapOr"
    ) {
        return Err(PerformanceError::Unsupported);
    }
    let startup = number(obj.get("Startup Cost"))?;
    if number(obj.get("Total Cost"))? < startup {
        return Err(PerformanceError::Decode);
    }
    number(obj.get("Plan Rows"))?;
    number(obj.get("Plan Width"))?;
    // An actual-execution field is never accepted as estimated evidence.
    if obj
        .keys()
        .any(|k| k.starts_with("Actual ") || matches!(k.as_str(), "Triggers" | "Execution Time"))
    {
        return Err(PerformanceError::Unsupported);
    }
    *nodes += 1;
    if let Some(children) = obj.get("Plans") {
        let children = children.as_array().ok_or(PerformanceError::Decode)?;
        if children.len() > 16 {
            return Err(PerformanceError::Budget);
        }
        for child in children {
            pg_node(child, depth + 1, nodes, max)?;
        }
    }
    Ok(())
}
pub(super) fn parse(
    raw: &[u8],
    dialect: Dialect,
    b: PerformanceBudget,
) -> PerfResult<PerformanceView> {
    if raw.len() > b.bytes {
        return Err(PerformanceError::Budget);
    }
    let mut d = serde_json::Deserializer::from_slice(raw);
    let mut remaining = 2048;
    Scan {
        depth: 0,
        remaining: &mut remaining,
    }
    .deserialize(&mut d)
    .map_err(|_| PerformanceError::Decode)?;
    d.end().map_err(|_| PerformanceError::Decode)?;
    let v: Json = serde_json::from_slice(raw).map_err(|_| PerformanceError::Decode)?;
    let nodes = if dialect == Dialect::Postgres {
        let root = v
            .as_array()
            .filter(|a| a.len() == 1)
            .ok_or(PerformanceError::Decode)?;
        let root = root[0].as_object().ok_or(PerformanceError::Decode)?;
        if root.len() != 1 {
            return Err(PerformanceError::Unsupported);
        }
        let mut n = 0;
        pg_node(
            root.get("Plan").ok_or(PerformanceError::Decode)?,
            0,
            &mut n,
            b.nodes,
        )?;
        n
    } else {
        let root = v
            .as_object()
            .filter(|o| o.len() == 1)
            .ok_or(PerformanceError::Decode)?;
        let block = root
            .get("query_block")
            .and_then(Json::as_object)
            .ok_or(PerformanceError::Unsupported)?;
        if block.get("select_id").and_then(Json::as_u64) != Some(1) {
            return Err(PerformanceError::Unsupported);
        }
        if block.len() == 2
            && matches!(
                block.get("message").and_then(Json::as_str),
                Some("Impossible WHERE" | "Impossible WHERE noticed after reading const tables")
            )
        {
            return Ok(PerformanceView { nodes: 0 });
        }
        // Only a single-table block: no nested-loop, subquery, materialization,
        // ordering/grouping/window or arbitrary optimizer operator is supported.
        if block
            .keys()
            .any(|k| !matches!(k.as_str(), "select_id" | "cost_info" | "table"))
        {
            return Err(PerformanceError::Unsupported);
        }
        let table = block
            .get("table")
            .and_then(Json::as_object)
            .ok_or(PerformanceError::Unsupported)?;
        if table.keys().any(|k| {
            !matches!(
                k.as_str(),
                "table_name"
                    | "access_type"
                    | "possible_keys"
                    | "key"
                    | "used_key_parts"
                    | "key_length"
                    | "ref"
                    | "rows_examined_per_scan"
                    | "rows_produced_per_join"
                    | "filtered"
                    | "cost_info"
                    | "used_columns"
                    | "attached_condition"
                    | "using_index"
            )
        }) {
            return Err(PerformanceError::Unsupported);
        }
        if !matches!(
            table.get("access_type").and_then(Json::as_str),
            Some("ALL" | "index" | "range" | "ref" | "const" | "eq_ref")
        ) {
            return Err(PerformanceError::Unsupported);
        }
        if table.get("table_name").and_then(Json::as_str).is_none() {
            return Err(PerformanceError::Decode);
        }
        number(table.get("rows_examined_per_scan"))?;
        number(table.get("rows_produced_per_join"))?;
        if number(table.get("filtered"))? > 100.0 {
            return Err(PerformanceError::Decode);
        }
        let cost = block.get("cost_info").and_then(|v| v.get("query_cost"));
        number(cost)?;
        1
    };
    Ok(PerformanceView { nodes: nodes as u8 })
}

macro_rules! performance_dispatch {
    ($driver:ident, $dialect:ident) => {
        #[cfg(test)]
        pub(super) async fn test_performance_metadata(&mut self) {
            // Fixture-only diagnostic control, never a production submission.
            let row = sqlx::query::<$driver>("SELECT 1")
                .fetch_one(self.connection.native())
                .await
                .unwrap();
            assert_eq!(performance_version(&row), Err(PerformanceError::Metadata));
            assert_eq!(performance_payload(&row), Err(PerformanceError::Metadata));
        }
        /// DB/provider-free opt-in. Only the ordinary common gate's finite SELECT
        /// subset is supported; COUNT/DML/Raw/functions/permits are not promoted.
        pub fn performance_request(
            &self,
            current: &DatabaseContext,
            operation: &Operation,
            budget: PerformanceBudget,
            ttl: Duration,
        ) -> performance::PerfResult<PerformanceRequest> {
            let handle = self.validate(current, operation, ttl)?;
            if handle
                .prepared
                .sensitive_source()
                .final_source()
                .metadata()
                .family()
                != Family::Select
            {
                return Err(PerformanceError::Unsupported);
            }
            Ok(PerformanceRequest {
                operation: handle,
                budget,
            })
        }
        /// Explicit fixed non-ANALYZE EXPLAIN only. One independent version read
        /// plus one EXPLAIN; no business execution, provider or transaction.
        /// Failure/cancellation after I/O starts quarantines this owned session.
        pub async fn acquire_performance(
            &mut self,
            current: &DatabaseContext,
            operation: &Operation,
            request: PerformanceRequest,
        ) -> performance::PerfResult<PerformancePlan> {
            observation::unsupported(&self.observation)?;
            quiet(async {
                if !self.available {
                    return Err(ExecutionError::SessionUnavailable.into());
                }
                let handle = &request.operation;
                recheck(
                    handle,
                    &self.identity,
                    &self.context,
                    current,
                    Some(operation),
                )?;
                check_expiry(handle, Instant::now())?;
                let budget = request.budget;
                let io_expires = Instant::now()
                    .checked_add(budget.timeout)
                    .ok_or(PerformanceError::Budget)?;
                self.available = false;
                let result = tokio::time::timeout(budget.timeout, async {
                    let version_sql = if Dialect::$dialect == Dialect::Postgres {
                        "SELECT pg_catalog.current_setting('server_version_num')"
                    } else {
                        "SELECT VERSION()"
                    };
                    #[cfg(test)]
                    self.recorder.event("performance_version");
                    let mut stream =
                        sqlx::query::<$driver>(version_sql).fetch(self.connection.native());
                    let row = stream
                        .try_next()
                        .await
                        .map_err(|_| PerformanceError::Read)?
                        .ok_or(PerformanceError::Metadata)?;
                    if row.len() != 1 {
                        return Err(PerformanceError::Metadata);
                    }
                    performance_version(&row)?;
                    if stream
                        .try_next()
                        .await
                        .map_err(|_| PerformanceError::Read)?
                        .is_some()
                    {
                        return Err(PerformanceError::Metadata);
                    }
                    drop(stream);
                    recheck(
                        handle,
                        &self.identity,
                        &self.context,
                        current,
                        Some(operation),
                    )?;
                    check_expiry(handle, Instant::now())?;
                    let source = handle.prepared.sensitive_source();
                    let record = source.final_source();
                    // Only these literal prefixes are permitted. The final SQL
                    // remains byte-for-byte intact; values are never interpolated.
                    let prefix = if Dialect::$dialect == Dialect::Postgres {
                        "EXPLAIN (FORMAT JSON) "
                    } else {
                        "EXPLAIN FORMAT=JSON "
                    };
                    let sql = format!("{prefix}{}", record.sql());
                    let mut query = sqlx::query::<$driver>(sqlx::AssertSqlSafe(sql.as_str()));
                    for value in record.args() {
                        bind(&mut query, value)?;
                    }
                    #[cfg(test)]
                    if let Some(barrier) = self.final_barrier.take() {
                        barrier.notified().await;
                    }
                    recheck(
                        handle,
                        &self.identity,
                        &self.context,
                        current,
                        Some(operation),
                    )?;
                    #[cfg(test)]
                    let now = self.final_now.take().unwrap_or_else(Instant::now);
                    #[cfg(not(test))]
                    let now = Instant::now();
                    check_expiry(handle, now)?;
                    #[cfg(test)]
                    {
                        self.recorder.event("performance_explain");
                        self.recorder.sent(&sql, record.args(), record.metadata());
                    }
                    let mut stream = query.fetch(self.connection.native());
                    let row = stream
                        .try_next()
                        .await
                        .map_err(|_| PerformanceError::Read)?
                        .ok_or(PerformanceError::Metadata)?;
                    let raw = performance_payload(&row)?;
                    if raw.len() > budget.bytes {
                        return Err(PerformanceError::Budget);
                    }
                    // Consume through end, including a sentinel second row.
                    if stream
                        .try_next()
                        .await
                        .map_err(|_| PerformanceError::Read)?
                        .is_some()
                    {
                        return Err(PerformanceError::Metadata);
                    }
                    drop(stream);
                    let view = performance::parse(&raw, Dialect::$dialect, budget)?;
                    #[cfg(test)]
                    if let Some(barrier) = self.performance_return_barrier.take() {
                        barrier.notified().await;
                    }
                    recheck(
                        handle,
                        &self.identity,
                        &self.context,
                        current,
                        Some(operation),
                    )?;
                    check_expiry(handle, Instant::now())?;
                    Ok((raw, view))
                })
                .await
                .map_err(|_| PerformanceError::Timeout)??;
                // Re-sample the real clock after all full comparisons and parse.
                recheck(
                    handle,
                    &self.identity,
                    &self.context,
                    current,
                    Some(operation),
                )?;
                check_expiry(handle, Instant::now())?;
                if Instant::now() >= io_expires {
                    return Err(PerformanceError::Timeout);
                }
                self.available = true;
                Ok(PerformancePlan {
                    _request: request,
                    _raw: result.0,
                    view: result.1,
                })
            })
            .await
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    fn budget() -> PerformanceBudget {
        PerformanceBudget::new(MAX_BYTES, 64, Duration::from_secs(5)).unwrap()
    }
    const PG:&[u8]=br#"[{"Plan":{"Node Type":"Seq Scan","Startup Cost":0.0,"Total Cost":12.5,"Plan Rows":3,"Plan Width":8,"Relation Name":"CANARY_NAME","Filter":"CANARY_VALUE"}}]"#;
    const MY:&[u8]=br#"{"query_block":{"select_id":1,"cost_info":{"query_cost":"1.25"},"table":{"table_name":"CANARY_NAME","access_type":"ALL","rows_examined_per_scan":3,"rows_produced_per_join":3,"filtered":"100.00","attached_condition":"CANARY_VALUE"}}}"#;
    #[test]
    fn finite_version_refusal() {
        for (d, good, bad) in [
            (
                Dialect::Postgres,
                b"160014".as_slice(),
                b"160013".as_slice(),
            ),
            (Dialect::Mysql, b"8.4.8".as_slice(), b"8.4.9".as_slice()),
        ] {
            assert_eq!(version(good, d), Ok(()));
            assert_eq!(version(bad, d), Err(PerformanceError::Version));
            assert_eq!(version(b"", d), Err(PerformanceError::Version));
            assert_eq!(version(&[b'x'; 65], d), Err(PerformanceError::Budget));
        }
    }
    #[test]
    fn finite_estimates_and_redaction() {
        for (raw, d) in [(PG, Dialect::Postgres), (MY, Dialect::Mysql)] {
            let view = parse(raw, d, budget()).unwrap();
            let text = view.to_json();
            let j: Json = serde_json::from_str(&text).unwrap();
            assert_eq!(
                j,
                serde_json::json!({"kind":"quent_performance_view","version":1,"state":"Observed","coverage":"Partial","precision":"Estimated","nodes":1,"auxiliary_submissions":1,"explain_submissions":1,"business_submissions":0,"provider_calls":0,"tx_submissions":0})
            );
            assert!(!format!("{view:?} {view} {text}").contains("CANARY"));
            assert!(
                std::str::from_utf8(raw).unwrap().contains("CANARY"),
                "positive control"
            );
            let tiny = PerformanceBudget::new(raw.len() - 1, 64, Duration::from_secs(1)).unwrap();
            assert_eq!(parse(raw, d, tiny), Err(PerformanceError::Budget));
        }
        for (bytes, nodes, timeout) in [
            (0, 1, 1),
            (MAX_BYTES + 1, 1, 1),
            (1, 0, 1),
            (1, 65, 1),
            (1, 1, 0),
            (1, 1, 11),
        ] {
            assert!(PerformanceBudget::new(bytes, nodes, Duration::from_secs(timeout)).is_err());
        }
    }
    #[test]
    fn rejects_unknown_malformed_sentinels_and_budgets() {
        for bad in [
            String::from_utf8(PG.to_vec())
                .unwrap()
                .replace("Seq Scan", "Unknown CANARY_NODE"),
            String::from_utf8(PG.to_vec())
                .unwrap()
                .replace("\"Plan Rows\":3,", ""),
            String::from_utf8(PG.to_vec())
                .unwrap()
                .replace("\"Plan Rows\":3", "\"Plan Rows\":-1"),
            String::from_utf8(PG.to_vec())
                .unwrap()
                .replace("\"Plan Rows\":3", "\"Plan Rows\":1e999"),
            String::from_utf8(PG.to_vec())
                .unwrap()
                .replace("\"Plan Rows\":3", "\"Plan Rows\":NaN"),
            String::from_utf8(PG.to_vec())
                .unwrap()
                .replace("\"Plan Rows\":3", "\"Plan Rows\":18446744073709551616"),
            String::from_utf8(PG.to_vec())
                .unwrap()
                .replace("\"Plan Rows\":3", "\"Plan Rows\":3,\"Plan Rows\":3"),
            format!("{}{{}}", std::str::from_utf8(PG).unwrap()),
            format!("{}0{}", "[".repeat(18), "]".repeat(18)),
            format!("[{}]", vec!["0"; 65].join(",")),
            format!("\"{}\"", "X".repeat(4097)),
        ] {
            assert!(parse(bad.as_bytes(), Dialect::Postgres, budget()).is_err());
        }
        for bad in [
            String::from_utf8(MY.to_vec())
                .unwrap()
                .replace("ALL", "UNKNOWN"),
            String::from_utf8(MY.to_vec())
                .unwrap()
                .replace("\"filtered\":\"100.00\"", "\"filtered\":\"101\""),
            String::from_utf8(MY.to_vec())
                .unwrap()
                .replace("\"query_cost\":\"1.25\"", "\"query_cost\":\"NaN\""),
            String::from_utf8(MY.to_vec())
                .unwrap()
                .replace("\"select_id\":1", "\"select_id\":1,\"nested_loop\":[]"),
        ] {
            assert!(parse(bad.as_bytes(), Dialect::Mysql, budget()).is_err());
        }
        let mut root: Json = serde_json::from_slice(PG).unwrap();
        root[0]["Plan"]["Plans"] = serde_json::json!([root[0]["Plan"].clone()]);
        let one = PerformanceBudget::new(MAX_BYTES, 1, Duration::from_secs(1)).unwrap();
        assert_eq!(
            parse(&serde_json::to_vec(&root).unwrap(), Dialect::Postgres, one),
            Err(PerformanceError::Budget)
        );
    }
}
