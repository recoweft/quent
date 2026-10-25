//! Independent untrusted data. No Deserialize implementations for Core carriers.
use super::*;
use quent_core::{
    Column, Compare, Dialect, Direction, Draft, Ident, Input, Order, PlanError, Predicate,
    Projection, SqlType, Table, Value as Native,
};
use serde_json::{Map, Value};
type Result<T> = std::result::Result<T, ReviewError>;
fn invalid<T>() -> Result<T> {
    Err(ReviewError::Invalid)
}
fn object<'a>(
    v: &'a Value,
    required: &[&str],
    optional: &[&str],
) -> Result<&'a Map<String, Value>> {
    let o = v.as_object().ok_or(ReviewError::Invalid)?;
    if required.iter().any(|k| !o.contains_key(*k))
        || o.keys()
            .any(|k| !required.contains(&k.as_str()) && !optional.contains(&k.as_str()))
    {
        return invalid();
    }
    Ok(o)
}
fn get<'a>(o: &'a Map<String, Value>, k: &str) -> Result<&'a Value> {
    o.get(k).ok_or(ReviewError::Invalid)
}
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or(ReviewError::Invalid)
}
fn array(v: &Value, min: usize, max: usize) -> Result<&[Value]> {
    let a = v.as_array().ok_or(ReviewError::Invalid)?;
    if a.len() < min {
        return invalid();
    }
    if a.len() > max {
        return Err(ReviewError::Budget);
    }
    Ok(a)
}
fn ty(v: &Value) -> Result<SqlType> {
    Ok(match text(v)? {
        "i8" => SqlType::I8,
        "i16" => SqlType::I16,
        "i32" => SqlType::I32,
        "i64" => SqlType::I64,
        "u8" => SqlType::U8,
        "u16" => SqlType::U16,
        "u32" => SqlType::U32,
        "u64" => SqlType::U64,
        "bool" => SqlType::Bool,
        "text" => SqlType::Text,
        _ => return Err(ReviewError::Unsupported),
    })
}
fn native(v: &Value) -> Result<Native> {
    let o = object(v, &["type", "value"], &[])?;
    let t = ty(get(o, "type")?)?;
    let v = get(o, "value")?;
    if v.is_null() {
        return Ok(Native::Null(t));
    }
    macro_rules! signed {
        ($n:ident, $t:ty) => {
            Native::$n(
                <$t>::try_from(v.as_i64().ok_or(ReviewError::Invalid)?)
                    .map_err(|_| ReviewError::Invalid)?,
            )
        };
    }
    macro_rules! unsigned {
        ($n:ident, $t:ty) => {
            Native::$n(
                <$t>::try_from(v.as_u64().ok_or(ReviewError::Invalid)?)
                    .map_err(|_| ReviewError::Invalid)?,
            )
        };
    }
    Ok(match t {
        SqlType::I8 => signed!(I8, i8),
        SqlType::I16 => signed!(I16, i16),
        SqlType::I32 => signed!(I32, i32),
        SqlType::I64 => signed!(I64, i64),
        SqlType::U8 => unsigned!(U8, u8),
        SqlType::U16 => unsigned!(U16, u16),
        SqlType::U32 => unsigned!(U32, u32),
        SqlType::U64 => unsigned!(U64, u64),
        SqlType::Bool => Native::Bool(v.as_bool().ok_or(ReviewError::Invalid)?),
        SqlType::Text => Native::Text(text(v)?.to_owned()),
    })
}
fn column(v: &Value, columns: &[Column]) -> Result<Column> {
    let index = v.as_u64().ok_or(ReviewError::Invalid)?;
    let index = usize::try_from(index).map_err(|_| ReviewError::Invalid)?;
    columns.get(index).cloned().ok_or(ReviewError::Invalid)
}
fn predicate(v: &Value, columns: &[Column]) -> Result<Predicate> {
    let o = v.as_object().ok_or(ReviewError::Invalid)?;
    let op = text(get(o, "op")?)?;
    Ok(match op {
        "and" | "or" => {
            object(v, &["op", "args"], &[])?;
            let ps = array(get(o, "args")?, 1, 32)?
                .iter()
                .map(|v| predicate(v, columns))
                .collect::<Result<Vec<_>>>()?;
            if op == "and" {
                Predicate::And(ps)
            } else {
                Predicate::Or(ps)
            }
        }
        "not" => {
            object(v, &["op", "arg"], &[])?;
            Predicate::Not(Box::new(predicate(get(o, "arg")?, columns)?))
        }
        "is_null" => {
            object(v, &["op", "column"], &[])?;
            Predicate::IsNull(column(get(o, "column")?, columns)?)
        }
        "in" => {
            object(v, &["op", "column", "values"], &[])?;
            Predicate::In(
                column(get(o, "column")?, columns)?,
                array(get(o, "values")?, 0, 64)?
                    .iter()
                    .map(native)
                    .collect::<Result<_>>()?,
            )
        }
        "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {
            object(v, &["op", "column", "value"], &[])?;
            let c = match op {
                "eq" => Compare::Eq,
                "ne" => Compare::Ne,
                "lt" => Compare::Lt,
                "le" => Compare::Le,
                "gt" => Compare::Gt,
                _ => Compare::Ge,
            };
            Predicate::Compare(
                column(get(o, "column")?, columns)?,
                c,
                native(get(o, "value")?)?,
            )
        }
        _ => return Err(ReviewError::Unsupported),
    })
}
fn core(e: PlanError) -> ReviewError {
    match e {
        PlanError::Budget => ReviewError::Budget,
        PlanError::Unsupported => ReviewError::Unsupported,
        _ => ReviewError::Invalid,
    }
}
fn choice(v: &Value, choices: &[&str]) -> Result<()> {
    if choices.contains(&text(v)?) {
        Ok(())
    } else {
        Err(ReviewError::Unsupported)
    }
}
fn claims(v: &Value) -> Result<u8> {
    let o = object(
        v,
        &[],
        &[
            "sql",
            "args",
            "metadata",
            "verdict",
            "risk",
            "coverage",
            "precision",
            "freshness",
            "fingerprint",
        ],
    )?;
    for (k, v) in o {
        match k.as_str() {
            "sql" => {
                text(v)?;
            }
            "fingerprint" => {
                if text(v)?.len() > 128 {
                    return Err(ReviewError::Budget);
                }
            }
            "args" => {
                for v in array(v, 0, 64)? {
                    native(v)?;
                }
            }
            "metadata" => {
                let m = object(v, &["family", "argument_count"], &[])?;
                choice(
                    get(m, "family")?,
                    &["select", "count", "insert", "update", "delete"],
                )?;
                if get(m, "argument_count")?.as_u64().is_none_or(|n| n > 1024) {
                    return invalid();
                }
            }
            "verdict" => choice(v, &["accepted", "refused", "unknown"])?,
            "risk" => choice(
                v,
                &["low", "medium", "high", "destructive", "blocked", "unknown"],
            )?,
            "coverage" => choice(v, &["complete", "partial", "unknown", "unsupported"])?,
            "precision" => choice(
                v,
                &["complete", "partial", "unknown", "conditional_conservative"],
            )?,
            "freshness" => choice(
                v,
                &[
                    "fresh",
                    "stale",
                    "unknown",
                    "live_verified",
                    "application_asserted",
                    "offline_consistent",
                ],
            )?,
            _ => return invalid(),
        }
    }
    Ok(o.len() as u8)
}
pub(super) fn review(bytes: &[u8]) -> Result<Review> {
    crate::operation_spec::bounded::check(bytes)?;
    let v: Value = serde_json::from_slice(bytes).map_err(|_| ReviewError::Invalid)?;
    let root = object(
        &v,
        &["kind", "version", "dialect", "operation"],
        &["claims"],
    )?;
    if text(get(root, "kind")?)? != "quent_review_plan_source"
        || get(root, "version")?.as_u64() != Some(1)
    {
        return Err(ReviewError::Unsupported);
    }
    let dialect = match text(get(root, "dialect")?)? {
        "postgres" => Dialect::Postgres,
        "mysql" => Dialect::Mysql,
        _ => return Err(ReviewError::Unsupported),
    };
    // Claims validated separately, never merged with operation data or ScopeFacts.
    let claim_count = root.get("claims").map(claims).transpose()?;
    let o = object(
        get(root, "operation")?,
        &[
            "family",
            "table",
            "columns",
            "projection",
            "writes",
            "order",
        ],
        &["filter", "limit"],
    )?;
    let path = array(get(o, "table")?, 1, 2)?
        .iter()
        .map(|v| Ident::new(text(v)?).map_err(core))
        .collect::<Result<Vec<_>>>()?;
    let table = Table::new(path, None).map_err(core)?;
    let mut names = std::collections::BTreeSet::new();
    let columns = array(get(o, "columns")?, 1, 64)?
        .iter()
        .map(|v| {
            let c = object(v, &["name", "type", "nullable"], &[])?;
            let name = text(get(c, "name")?)?;
            if !names.insert(name) {
                return invalid();
            }
            let native = ty(get(c, "type")?)?;
            if dialect == Dialect::Postgres
                && matches!(
                    native,
                    SqlType::I8 | SqlType::U8 | SqlType::U16 | SqlType::U32 | SqlType::U64
                )
            {
                return Err(ReviewError::Unsupported);
            }
            Ok(table.column(
                Ident::new(name).map_err(core)?,
                native,
                get(c, "nullable")?.as_bool().ok_or(ReviewError::Invalid)?,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let projections = array(get(o, "projection")?, 0, 64)?
        .iter()
        .map(|v| Ok(Projection::new(column(v, &columns)?, None)))
        .collect::<Result<Vec<_>>>()?;
    let writes = array(get(o, "writes")?, 0, 64)?
        .iter()
        .map(|v| {
            let w = object(v, &["column", "value"], &[])?;
            Ok((
                column(get(w, "column")?, &columns)?,
                Input::Value(native(get(w, "value")?)?),
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    let family = text(get(o, "family")?)?;
    if (!matches!(family, "select") && !projections.is_empty())
        || (!matches!(family, "insert" | "update") && !writes.is_empty())
    {
        return invalid();
    }
    let mut draft = match family {
        "select" => Draft::select(table, projections),
        "count" => Draft::select(table, vec![]).count().map_err(core)?,
        "insert" => Draft::insert(table, writes),
        "update" => Draft::update(table, writes),
        "delete" => Draft::delete(table),
        _ => return Err(ReviewError::Unsupported),
    };
    if let Some(p) = o.get("filter") {
        if family == "insert" {
            return invalid();
        }
        draft = draft.filter(predicate(p, &columns)?);
    }
    let order = array(get(o, "order")?, 0, 16)?
        .iter()
        .map(|v| {
            let e = object(v, &["column", "direction"], &[])?;
            Ok(Order {
                column: column(get(e, "column")?, &columns)?,
                direction: match text(get(e, "direction")?)? {
                    "asc" => Direction::Asc,
                    "desc" => Direction::Desc,
                    _ => return Err(ReviewError::Unsupported),
                },
            })
        })
        .collect::<Result<Vec<_>>>()?;
    if family != "select" && (!order.is_empty() || o.contains_key("limit")) {
        return invalid();
    }
    draft = draft.order(order);
    if let Some(n) = o.get("limit") {
        draft = draft.limit(n.as_u64().ok_or(ReviewError::Invalid)?);
    }
    let plan = draft.freeze().finalize(dialect).map_err(core)?;
    let i = plan.inspect(None);
    let mut r = super::inspection(&i);
    r.source = "plan_source";
    r.precision.wire = "checked";
    r.precision.external_verdict = "unverified";
    r.precision.sql = "finite_data_finalized";
    r.precision.native = "finite_data_checked";
    r.action = "supply_application_context";
    // High/Unknown from missing facts is retained, not an external verdict proof.
    r.state = if i.risk() == Risk::Blocked {
        State::Refused
    } else {
        State::Unknown
    };
    r.code = if i.risk() == Risk::Blocked {
        Code::StructuralBlock
    } else {
        Code::SourceParsed
    };
    r.claim_count = claim_count.unwrap_or(0);
    r.claims = if claim_count.is_some() {
        "unverified"
    } else {
        "absent"
    };
    Ok(r)
}
