mod support;
use support::{fixture::*, recorder::*};
const SOURCE: &[u8] = include_bytes!("../../../fixtures/contracts-v1.json");
fn register() -> Register {
    load(SOURCE).unwrap()
}
fn json() -> serde_json::Value {
    serde_json::from_slice(SOURCE).unwrap()
}
fn check_rejected(v: serde_json::Value) {
    assert!(load(&serde_json::to_vec(&v).unwrap()).is_err());
}

#[test]
fn fixture_coverage_and_owner_register() {
    let r = register();
    assert_eq!(r.cases.len(), 24);
    for i in 1..=24 {
        let c = r
            .cases
            .iter()
            .find(|c| c.id == format!("QT-C{i:02}"))
            .unwrap();
        assert_eq!(
            c.owners.iter().map(String::as_str).collect::<Vec<_>>(),
            owners(&c.id).unwrap()
        );
        assert!(c.evidence.contains(&"runtime_pending".into()));
    }
}
#[test]
fn reject_version_kind_coverage_owners_and_evidence() {
    for (field, value) in [("version", "2"), ("kind", "public-summary")] {
        let mut v = json();
        v[field] = value.into();
        check_rejected(v);
    }
    let mut v = json();
    v["cases"][1]["id"] = "QT-C01".into();
    check_rejected(v);
    let mut v = json();
    v["cases"].as_array_mut().unwrap().pop();
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["id"] = "QT-C25".into();
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["owners"] = serde_json::json!(["QT-AI-14"]);
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["evidence"] = serde_json::json!(["runtime_passed"]);
    check_rejected(v);
}
#[test]
fn reject_unknown_duplicate_missing_mistyped_and_multiple_documents() {
    let mut v = json();
    v["extra"] = true.into();
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["preconditions"]["unknown"] = true.into();
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["scenarios"][0]["shape"]["filter"]["unknown"] = true.into();
    check_rejected(v);
    let mut v = json();
    v.as_object_mut().unwrap().remove("version");
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["dialects"] = true.into();
    check_rejected(v);
    let source = std::str::from_utf8(SOURCE).unwrap();
    assert!(
        load(
            source
                .replacen(
                    "\"version\": \"1\"",
                    "\"version\": \"1\", \"version\": \"1\"",
                    1
                )
                .as_bytes()
        )
        .is_err()
    );
    assert!(
        load(
            source
                .replacen("\"bits\": \"64\"", "\"bits\": \"64\", \"bits\": \"64\"", 1)
                .as_bytes()
        )
        .is_err()
    );
    assert!(load(format!("{source} {{}}").as_bytes()).is_err());
    assert!(load(b"{broken").is_err());
    assert!(load(b"null").is_err());
}
#[test]
fn reject_size_string_array_object_depth_and_numeric_budgets() {
    assert!(load(&vec![b' '; MAX_BYTES + 1]).is_err());
    let mut v = json();
    v["kind"] = "x".repeat(4097).into();
    check_rejected(v);
    let mut v = json();
    v["cases"] = serde_json::json!(vec![true; 257]);
    check_rejected(v);
    let mut v = json();
    for i in 0..65 {
        v[format!("x{i}")] = true.into();
    }
    check_rejected(v);
    let mut v = json();
    v["version"] = serde_json::json!(1);
    check_rejected(v);
    let mut v = json();
    v["version"] = serde_json::json!(1.5);
    check_rejected(v);
    let mut deep = serde_json::json!({"node":"absent"});
    for _ in 0..20 {
        deep = serde_json::json!({"node":"not","child":deep});
    }
    let mut v = json();
    v["cases"][0]["scenarios"][0]["shape"]["filter"] = deep;
    check_rejected(v);
    assert!(load(format!("{}true{}", "[".repeat(150), "]".repeat(150)).as_bytes()).is_err());
}
#[test]
fn lossless_types_null_missing_order_and_tree() {
    let r = register();
    let c = &r.cases[6];
    let values = &c.scenarios[2].shape.rows[0];
    assert!(
        matches!(&values[0],Input::Signed{bits,lexeme} if bits=="64" && lexeme=="-9223372036854775808")
    );
    assert!(
        matches!(&values[1],Input::Unsigned{bits,lexeme} if bits=="64" && lexeme=="18446744073709551615")
    );
    assert!(
        matches!(&values[2],Input::Decimal{precision,scale,lexeme} if precision=="24" && scale=="4" && lexeme=="12345678901234567890.0100")
    );
    assert!(matches!(&values[3], Input::Missing { .. }));
    assert!(matches!(&values[4], Input::Null { .. }));
    assert!(matches!(&values[5],Input::Text{value} if value.is_empty()));
    assert!(matches!(&values[6],Input::Signed{lexeme,..} if lexeme=="000"));
    assert_eq!(values[7], Input::Bool { value: false });
    let Filter::And { children } = &c.scenarios[0].shape.filter else {
        panic!()
    };
    assert!(
        matches!(&children[0],Filter::Not{child} if matches!(child.as_ref(),Filter::Eq{value:Input::Null{..},..}))
    );
    assert!(matches!(&children[1],Filter::In{values,..} if values.is_empty()));
    assert!(
        matches!(&r.cases[5].scenarios[0].shape.filter,Filter::Or{children} if matches!(&children[0],Filter::And{..}))
    );
    assert_eq!(r.cases[3].preconditions.schemas[0].primary_key.len(), 3);
    assert_eq!(c.preconditions.schemas[0].nullable_unique, ["items.email"]);
    assert!(r.cases[7].preconditions.allow_claim); // remains only documentary data
}
#[test]
fn reject_invalid_width_decimal_and_typed_lexemes() {
    for input in [
        serde_json::json!({"type":"signed","bits":"8","lexeme":"128"}),
        serde_json::json!({"type":"unsigned","bits":"64","lexeme":"18446744073709551616"}),
        serde_json::json!({"type":"signed","bits":"128","lexeme":"0"}),
        serde_json::json!({"type":"signed","bits":"64","lexeme":1}),
        serde_json::json!({"type":"decimal","precision":"4","scale":"2","lexeme":"1.234"}),
        serde_json::json!({"type":"decimal","precision":"4","scale":"2","lexeme":"1e2"}),
        serde_json::json!({"type":"null","sql_type":"Any"}),
    ] {
        let mut v = json();
        v["cases"][0]["preconditions"]["tenant"] = input;
        check_rejected(v);
    }
}

fn call(dialect: Dialect, n: &str) -> Event {
    Event::Call(Call {
        operation: Operation::Update,
        dialect,
        descriptor: Descriptor::Probe,
        args: vec![
            Input::Signed {
                bits: "64".into(),
                lexeme: n.into(),
            },
            Input::Null {
                sql_type: "text".into(),
            },
        ],
    })
}
fn obs(event: Event, reply: Reply) -> Observation {
    Observation { event, reply }
}
fn count() -> Reply {
    Reply::Count {
        kind: CountKind::PostgresAffected,
        value: 1,
    }
}
#[test]
fn recorder_order_count_results_and_commit() {
    let script = vec![
        obs(Event::Begin, Reply::Unit),
        obs(call(Dialect::Postgres, "1"), count()),
        obs(
            call(Dialect::Postgres, "2"),
            Reply::Rows(vec![vec![Input::Bool { value: false }]]),
        ),
        obs(Event::Commit, Reply::Unit),
    ];
    let mut r = Recorder::new(script.clone());
    for s in &script {
        assert_eq!(r.record(s.event.clone()), s.reply);
    }
    assert_eq!(r.call_count(), 2);
    assert_eq!(r.verify(&script), Ok(()));
    let mut reversed = script.clone();
    reversed.swap(1, 2);
    assert!(r.verify(&reversed).is_err());
    assert!(r.verify(&script[..3]).is_err());
    assert_eq!(
        r.record(call(Dialect::Postgres, "3")),
        Reply::Error(Failure::Unexpected)
    );
    assert_eq!(r.call_count(), 3);
    assert!(r.verify(&script).is_err());
}
#[test]
fn recorder_clone_isolation_reset_and_unconsumed_script() {
    let script = vec![obs(call(Dialect::Postgres, "1"), count())];
    let mut a = Recorder::new(script.clone());
    let mut b = a.clone();
    assert!(b.verify(&[]).is_err());
    a.record(script[0].event.clone());
    assert_eq!(a.call_count(), 1);
    assert_eq!(b.call_count(), 0);
    b.reset(vec![]);
    assert_eq!(b.verify(&[]), Ok(()));
    assert_eq!(a.verify(&script), Ok(()));
    a.reset(vec![]);
    assert_eq!(a.call_count(), 0);
    assert_eq!(a.record(Event::Commit), Reply::Error(Failure::InvalidState));
}
#[test]
fn recorder_failure_rollback_unknown_commit_and_unobserved_tx() {
    for failure in [Failure::Driver, Failure::Decode] {
        let script = vec![
            obs(Event::Begin, Reply::Unit),
            obs(call(Dialect::Postgres, "1"), Reply::Error(failure)),
            obs(Event::Rollback, Reply::Unit),
        ];
        let mut r = Recorder::new(script.clone());
        for s in &script {
            assert_eq!(r.record(s.event.clone()), s.reply);
        }
        assert_eq!(r.verify(&script), Ok(()));
        assert!(!r.observations().iter().any(|o| o.event == Event::Commit));
    }
    for reply in [Reply::Unknown, Reply::Error(Failure::Commit)] {
        let script = vec![
            obs(Event::Begin, Reply::Unit),
            obs(Event::Commit, reply.clone()),
        ];
        let mut r = Recorder::new(script.clone());
        r.record(Event::Begin);
        assert_eq!(r.record(Event::Commit), reply);
        assert_eq!(r.verify(&script), Ok(()));
        assert_eq!(
            r.record(Event::Rollback),
            Reply::Error(Failure::InvalidState)
        );
    }
    let mut r = Recorder::default();
    assert_eq!(r.record(Event::Begin), Reply::Error(Failure::Unexpected));
    assert_eq!(r.record(Event::Commit), Reply::Error(Failure::InvalidState));
}
#[test]
fn recorder_rejects_order_argument_and_result_mismatch() {
    let mut r = Recorder::new(vec![obs(call(Dialect::Postgres, "1"), count())]);
    assert_eq!(
        r.record(call(Dialect::Postgres, "2")),
        Reply::Error(Failure::Unexpected)
    );
    assert!(r.verify(&[]).is_err());
    for reply in [
        Reply::Unit,
        Reply::Count {
            kind: CountKind::MysqlChanged,
            value: 1,
        },
    ] {
        let event = call(Dialect::Postgres, "1");
        let mut r = Recorder::new(vec![obs(event.clone(), reply)]);
        assert_eq!(r.record(event), Reply::Error(Failure::InvalidResult));
    }
    let Event::Call(mut c) = call(Dialect::Mysql, "1") else {
        panic!()
    };
    c.args.swap(0, 1);
    let event = Event::Call(c.clone());
    let mut r = Recorder::new(vec![obs(call(Dialect::Mysql, "1"), Reply::Unknown)]);
    assert_eq!(r.record(event), Reply::Error(Failure::Unexpected));
    c.args.push(Input::Missing {
        sql_type: "i64".into(),
    });
    assert_eq!(
        r.record(Event::Call(c)),
        Reply::Error(Failure::InvalidInput)
    );
    for kind in [
        CountKind::MysqlChanged,
        CountKind::MysqlMatched,
        CountKind::MysqlUpsert,
    ] {
        let event = call(Dialect::Mysql, "1");
        let reply = Reply::Count { kind, value: 2 };
        let mut r = Recorder::new(vec![obs(event.clone(), reply.clone())]);
        assert_eq!(r.record(event), reply);
    }
}

// These adapters select the SAME source scenarios for each dialect. They do not
// lower production IR or run SQL, and intentionally have no runtime verdict API.
fn dialect_template(r: &Register, dialect: Dialect) -> Vec<(&str, &[Scenario])> {
    r.cases
        .iter()
        .filter(|c| c.dialects.contains(&dialect))
        .map(|c| (c.id.as_str(), c.scenarios.as_slice()))
        .collect()
}
#[test]
fn postgres_mysql_templates_share_semantics_without_sql_equality() {
    let r = register();
    let pg = dialect_template(&r, Dialect::Postgres);
    let my = dialect_template(&r, Dialect::Mysql);
    assert_eq!(pg, my);
    assert_eq!(pg.len(), 24);
    // A recorded descriptor can point at either source without pretending dispatch.
    for dialect in [Dialect::Postgres, Dialect::Mysql] {
        let event = Event::Call(Call {
            operation: Operation::Select,
            dialect,
            descriptor: Descriptor::Fixture {
                case: pg[0].0.into(),
                scenario: pg[0].1[0].name.clone(),
            },
            args: vec![],
        });
        let script = vec![obs(event.clone(), Reply::Unknown)];
        let mut recorder = Recorder::new(script.clone());
        assert_eq!(recorder.record(event), Reply::Unknown);
        assert_eq!(recorder.verify(&script), Ok(()));
    }
}

#[test]
fn reject_schema_row_width_and_nested_missing_fields() {
    let mut v = json();
    v["cases"][0]["preconditions"]["schemas"][0]["primary_key"] =
        serde_json::json!(["unknown.column"]);
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["preconditions"]["schemas"][0]["columns"][0]["sql_type"] = "Any".into();
    check_rejected(v);
    let mut v = json();
    v["cases"][8]["scenarios"][0]["shape"]["rows"][0]
        .as_array_mut()
        .unwrap()
        .pop();
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["scenarios"][0]["expected"]
        .as_object_mut()
        .unwrap()
        .remove("risk");
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["scenarios"][0]["expected"]["risk"] = "safe".into();
    check_rejected(v);
    let mut v = json();
    v["cases"][0]["dialects"] = serde_json::json!(["postgres"]);
    check_rejected(v);
}
#[test]
fn recorder_reset_clears_active_state_and_clone_has_independent_tx() {
    let script = vec![
        obs(Event::Begin, Reply::Unit),
        obs(Event::Rollback, Reply::Unit),
    ];
    let mut a = Recorder::new(script.clone());
    assert_eq!(a.record(Event::Begin), Reply::Unit);
    let mut b = a.clone();
    a.reset(vec![]);
    assert_eq!(
        a.record(Event::Rollback),
        Reply::Error(Failure::InvalidState)
    );
    assert_eq!(b.record(Event::Rollback), Reply::Unit);
    assert_eq!(b.verify(&script), Ok(()));
}

// Deliberately finite test-source bridge, not a production JSON/OperationSpec reader.
// Fixture expectations never become risk/authorization or dispatch decisions.
fn plan_value(input: &Input) -> Result<quent_core::Value, quent_core::PlanError> {
    use quent_core::{PlanError as E, SqlType as T, Value as V};
    Ok(match input {
        Input::Signed { bits, lexeme } => match bits.as_str() {
            "8" => V::I8(lexeme.parse().map_err(|_| E::Type)?),
            "16" => V::I16(lexeme.parse().map_err(|_| E::Type)?),
            "32" => V::I32(lexeme.parse().map_err(|_| E::Type)?),
            "64" => V::I64(lexeme.parse().map_err(|_| E::Type)?),
            _ => return Err(E::Unsupported),
        },
        Input::Unsigned { bits, lexeme } => match bits.as_str() {
            "8" => V::U8(lexeme.parse().map_err(|_| E::Type)?),
            "16" => V::U16(lexeme.parse().map_err(|_| E::Type)?),
            "32" => V::U32(lexeme.parse().map_err(|_| E::Type)?),
            "64" => V::U64(lexeme.parse().map_err(|_| E::Type)?),
            _ => return Err(E::Unsupported),
        },
        Input::Null { sql_type } => V::Null(match sql_type.as_str() {
            "i64" => T::I64,
            "text" => T::Text,
            "bool" => T::Bool,
            _ => return Err(E::Unsupported),
        }),
        Input::Text { value } => V::Text(value.clone()),
        Input::Bool { value } => V::Bool(*value),
        Input::Missing { .. } => return Err(E::Missing),
        Input::Decimal { .. } => return Err(E::Unsupported),
    })
}
fn plan_column(table: &quent_core::Table, schema: &Schema, name: &str) -> quent_core::Column {
    use quent_core::{Ident, SqlType};
    let source = schema.columns.iter().find(|c| c.name == name).unwrap();
    let (prefix, name) = name.split_once('.').unwrap();
    assert_eq!(prefix, "items");
    let ty = match source.sql_type.as_str() {
        "i64" => SqlType::I64,
        "text" => SqlType::Text,
        "bool" => SqlType::Bool,
        _ => panic!("unsupported test column"),
    };
    table.column(Ident::new(name).unwrap(), ty, source.nullable)
}
fn plan_filter(
    table: &quent_core::Table,
    schema: &Schema,
    filter: &Filter,
) -> Option<quent_core::Predicate> {
    use quent_core::{Compare, Predicate as P};
    let c = |name: &str| plan_column(table, schema, name);
    Some(match filter {
        Filter::Absent => return None,
        Filter::And { children } => P::And(
            children
                .iter()
                .map(|f| plan_filter(table, schema, f).unwrap())
                .collect(),
        ),
        Filter::Or { children } => P::Or(
            children
                .iter()
                .map(|f| plan_filter(table, schema, f).unwrap())
                .collect(),
        ),
        Filter::Not { child } => P::Not(Box::new(plan_filter(table, schema, child).unwrap())),
        Filter::Eq { column, value } => {
            P::Compare(c(column), Compare::Eq, plan_value(value).unwrap())
        }
        Filter::Gt { column, value } => {
            P::Compare(c(column), Compare::Gt, plan_value(value).unwrap())
        }
        Filter::In { column, values } => P::In(
            c(column),
            values.iter().map(|v| plan_value(v).unwrap()).collect(),
        ),
        Filter::IsNull { column } => P::IsNull(c(column)),
    })
}
#[test]
fn production_plan_consumes_c01_c07_source_without_db_or_provider() {
    use quent_core::{Dialect as D, Draft, Ident, Projection, Table, Value};
    let register = register();
    for (case, pg, args) in [
        (
            "QT-C01",
            "SELECT \"items\".\"id\" FROM \"items\" WHERE ((\"items\".\"tenant_id\" = $1) AND (\"items\".\"id\" = $2))",
            vec![Value::I64(1), Value::I64(1)],
        ),
        (
            "QT-C07",
            "SELECT \"items\".\"id\" FROM \"items\" WHERE ((NOT (\"items\".\"email\" = $1)) AND (1 = 0) AND (\"items\".\"email\" IS NULL))",
            vec![Value::Null(quent_core::SqlType::Text)],
        ),
    ] {
        let case = register.cases.iter().find(|c| c.id == case).unwrap();
        let source = &case.scenarios[0].shape;
        let table = Table::new(vec![Ident::new("items").unwrap()], None).unwrap();
        let projection = source
            .projection
            .iter()
            .map(|n| Projection::new(plan_column(&table, &case.preconditions.schemas[0], n), None))
            .collect();
        let predicate =
            plan_filter(&table, &case.preconditions.schemas[0], &source.filter).unwrap();
        let operation = Draft::select(table, projection)
            .filter(predicate.clone())
            .freeze();
        for d in [D::Postgres, D::Mysql] {
            // There is no endpoint to attach this recorder or a provider to finalize.
            // Empty observations support DB-free design; they do not test a dispatcher.
            let recorder = Recorder::new(vec![]);
            let plan = operation.finalize(d).unwrap();
            let source = plan.sensitive_source();
            let sql = if d == D::Postgres {
                pg.into()
            } else {
                pg.replace('"', "`").replace("$1", "?").replace("$2", "?")
            };
            assert_eq!(source.sql(), sql);
            assert_eq!(source.args(), args);
            assert_eq!(source.metadata().predicate(), Some(&predicate));
            assert_eq!(plan, operation.finalize(d).unwrap());
            assert_eq!(recorder.call_count(), 0);
            recorder.verify(&[]).unwrap();
        }
        assert!(case.evidence.contains(&"runtime_pending".into()));
    }
}
#[test]
fn production_plan_fixture_values_refuse_decimal_missing_and_overflow() {
    use quent_core::{PlanError as E, Value};
    let r = register();
    let case = r.cases.iter().find(|c| c.id == "QT-C07").unwrap();
    let row = &case
        .scenarios
        .iter()
        .find(|s| s.name == "lossless-values")
        .unwrap()
        .shape
        .rows[0];
    assert_eq!(plan_value(&row[0]), Ok(Value::I64(i64::MIN)));
    assert_eq!(plan_value(&row[1]), Ok(Value::U64(u64::MAX)));
    assert_eq!(plan_value(&row[2]), Err(E::Unsupported));
    assert_eq!(plan_value(&row[3]), Err(E::Missing));
    assert_eq!(
        plan_value(&row[4]),
        Ok(Value::Null(quent_core::SqlType::Text))
    );
    assert_eq!(plan_value(&row[5]), Ok(Value::Text(String::new())));
    assert_eq!(plan_value(&row[6]), Ok(Value::I64(0))); // native semantic value; source lexeme is documentary
    assert_eq!(plan_value(&row[7]), Ok(Value::Bool(false)));
    assert_eq!(
        plan_value(&Input::Signed {
            bits: "8".into(),
            lexeme: "128".into()
        }),
        Err(E::Type)
    );
    assert_eq!(
        plan_value(&Input::Unsigned {
            bits: "64".into(),
            lexeme: "18446744073709551616".into()
        }),
        Err(E::Type)
    );
    assert_eq!(
        plan_value(&Input::Null {
            sql_type: "text); DROP".into()
        }),
        Err(E::Unsupported)
    );
}

#[test]
fn production_inspection_c02_c05_c07_independent_of_fixture_verdicts() {
    use quent_core::{
        Dialect as D, Draft, Evidence, Ident, Input as I, Key, Projection, Risk, Scope, ScopeFacts,
        Table, TableContext, Value,
    };
    let r = register();
    let mut count = 0;
    for case in r
        .cases
        .iter()
        .filter(|c| ["QT-C02", "QT-C03", "QT-C04", "QT-C05", "QT-C07"].contains(&c.id.as_str()))
    {
        let schema = &case.preconditions.schemas[0];
        let t = Table::new(vec![Ident::new("items").unwrap()], None).unwrap();
        for scenario in &case.scenarios {
            if scenario.name == "lossless-values" {
                continue;
            } // unsupported decimal/missing tested separately
            let filter = plan_filter(&t, schema, &scenario.shape.filter);
            let draft = match scenario.shape.operation {
                Operation::Update => Draft::update(
                    t.clone(),
                    vec![(
                        plan_column(&t, schema, "items.status"),
                        I::Value(Value::Text("updated_canary".into())),
                    )],
                ),
                Operation::Delete => Draft::delete(t.clone()),
                Operation::Select => Draft::select(
                    t.clone(),
                    scenario
                        .shape
                        .projection
                        .iter()
                        .map(|n| Projection::new(plan_column(&t, schema, n), None))
                        .collect(),
                ),
                _ => panic!("unsupported bridge shape"),
            };
            let operation = if let Some(p) = filter.clone() {
                draft.filter(p)
            } else {
                draft
            }
            .freeze();
            for d in [D::Postgres, D::Mysql] {
                // Registry is documentary. Neither its schema/freshness/allow claims
                // nor expected risk can establish ordinary-table context.
                let columns = schema
                    .columns
                    .iter()
                    .map(|c| plan_column(&t, schema, &c.name))
                    .collect::<Vec<_>>();
                let keys = vec![Key::Primary(
                    schema
                        .primary_key
                        .iter()
                        .map(|n| plan_column(&t, schema, n))
                        .collect(),
                )];
                let unsupported = ScopeFacts::application_asserted(
                    t.clone(),
                    d,
                    columns.clone(),
                    keys.clone(),
                    TableContext::Unsupported,
                )
                .unwrap();
                let plan = operation.finalize(d).unwrap();
                assert_eq!(
                    plan.inspect(Some(&unsupported)).evidence(),
                    Evidence::UnsupportedContext
                );
                // Explicit test-harness assumption, independent of fixture claims.
                // Exclude nullable text unique from this supported signed-key domain.
                let asserted = ScopeFacts::application_asserted(
                    t.clone(),
                    d,
                    columns,
                    keys,
                    TableContext::OrdinaryBaseTableWithoutEffects,
                )
                .unwrap();
                let sql = plan.sensitive_source().sql().to_owned();
                let args = plan.sensitive_source().args().to_vec();
                let meta = plan.sensitive_source().metadata().clone();
                let i = plan.inspect(Some(&asserted));
                // Independent expected mapping for the limited production contract.
                // Full fixture policy/dispatch/Tx verdicts remain runtime_pending.
                let (scope, risk) = match scenario.name.as_str() {
                    "no-where-update" | "no-where-delete" => (Scope::Broad, Risk::Blocked),
                    "tenant-wide" | "id-positive" | "partial-composite" => {
                        (Scope::Broad, Risk::High)
                    }
                    "multi-in" => (Scope::Broad, Risk::Destructive),
                    "unproven-or" => (Scope::Unknown, Risk::High),
                    "not-null-empty-in" => (Scope::Zero, Risk::Medium),
                    "nullable-unique" => (Scope::Zero, Risk::Low), // = NULL has no TRUE rows; not single or authority
                    "complete-composite" | "same-composite-or" => (Scope::Single, Risk::Low),
                    "different-composite-or" => (Scope::FiniteBound(2), Risk::High),
                    _ => panic!("unregistered scope scenario"),
                };
                assert_eq!(i.scope(), scope);
                assert_eq!(i.risk(), risk);
                assert_eq!(i.scope(), plan.inspect(Some(&asserted)).scope());
                assert_eq!(i.findings(), plan.inspect(Some(&asserted)).findings());
                assert_eq!(plan.sensitive_source().sql(), sql);
                assert_eq!(plan.sensitive_source().args(), args);
                assert_eq!(plan.sensitive_source().metadata(), &meta);
                assert_eq!(meta.predicate(), filter.as_ref());
                // Independent argument oracle for each source scenario.
                let expected = match scenario.name.as_str() {
                    "no-where-update" => vec![Value::Text("updated_canary".into())],
                    "no-where-delete" => vec![],
                    "tenant-wide" => vec![Value::Text("updated_canary".into()), Value::I64(1)],
                    "id-positive" => vec![Value::Text("updated_canary".into()), Value::I64(0)],
                    "multi-in" => vec![Value::I64(1), Value::I64(2)],
                    "partial-composite" => vec![
                        Value::Text("updated_canary".into()),
                        Value::I64(1),
                        Value::I64(1),
                    ],
                    "unproven-or" => vec![
                        Value::Text("updated_canary".into()),
                        Value::I64(1),
                        Value::Text("active".into()),
                    ],
                    "not-null-empty-in" => vec![Value::Null(quent_core::SqlType::Text)],
                    "nullable-unique" => vec![
                        Value::Text("updated_canary".into()),
                        Value::Null(quent_core::SqlType::Text),
                    ],
                    "complete-composite" => vec![
                        Value::Text("updated_canary".into()),
                        Value::I64(1),
                        Value::I64(1),
                        Value::I64(1),
                    ],
                    "same-composite-or" => vec![
                        Value::Text("updated_canary".into()),
                        Value::I64(1),
                        Value::I64(1),
                        Value::I64(1),
                        Value::I64(1),
                        Value::I64(1),
                        Value::I64(1),
                    ],
                    "different-composite-or" => vec![
                        Value::Text("updated_canary".into()),
                        Value::I64(1),
                        Value::I64(1),
                        Value::I64(1),
                        Value::I64(1),
                        Value::I64(2),
                        Value::I64(1),
                    ],
                    _ => unreachable!(),
                };
                assert_eq!(args, expected);
                let recorder = Recorder::default();
                assert_eq!(recorder.call_count(), 0);
                recorder.verify(&[]).unwrap();
                count += 1;
            }
        }
    }
    assert_eq!(count, 24); // 12 source scenarios × 2 dialects; one unsupported value scenario skipped
}

#[test]
fn c08_documentary_claims_do_not_populate_application_snapshot() {
    use quent_core as core;
    let r = register();
    let case = r.cases.iter().find(|c| c.id == "QT-C08").unwrap();
    assert!(case.preconditions.allow_claim);
    for dialect in [core::Dialect::Postgres, core::Dialect::Mysql] {
        // Explicit server-side construction, deliberately no bridge from fixture
        // claims to tenant/subject/schema. Expectations are independent literals.
        let binding = core::DatabaseBinding::application_supplied(
            core::DatabaseIdentity::new("server-target").unwrap(),
            core::TargetAddress::new("server-route").unwrap(),
            core::Ident::new("catalog").unwrap(),
            dialect,
        );
        let policy = core::PolicySet::new(
            binding.clone(),
            core::PolicyIdentity::new("policy").unwrap(),
            core::Generation::new(1).unwrap(),
            vec![],
        )
        .unwrap();
        let settings = core::Settings::new(
            binding.clone(),
            core::SettingsIdentity::new("settings").unwrap(),
            core::Generation::new(1).unwrap(),
            policy,
        )
        .unwrap();
        let app =
            core::ApplicationContext::application_supplied(binding, None, None, None).unwrap();
        let context = core::DatabaseContext::new(settings, app).unwrap();
        assert_eq!(
            context.presence(),
            core::ContextPresence {
                schema: false,
                subject: false,
                tenant: false
            }
        );
        assert_eq!(context.sensitive_source().tenant(), None);
        assert_eq!(context.sensitive_source().subject(), None);
        let table = core::Table::new(
            vec![
                core::Ident::new("catalog").unwrap(),
                core::Ident::new("items").unwrap(),
            ],
            None,
        )
        .unwrap();
        assert_eq!(
            context
                .prepare()
                .prepare_operation(&core::Draft::delete(table).freeze())
                .unwrap_err(),
            core::PolicyError::MissingSchema
        );
        // No dispatcher/Tx exists: do not claim fixture expected refusal/rollback.
    }
}

#[test]
fn production_policy_common_source_c02_to_c10_finite_bridge() {
    use quent_core as q;
    let r = register();
    let mut measured = 0;
    for case in r.cases.iter().filter(|c| {
        [
            "QT-C02", "QT-C03", "QT-C04", "QT-C05", "QT-C06", "QT-C07", "QT-C09", "QT-C10",
        ]
        .contains(&c.id.as_str())
    }) {
        let schema = &case.preconditions.schemas[0];
        // Explicit harness namespace/ordinary-table assumptions, not fixture claims.
        let t = q::Table::new(
            vec![
                q::Ident::new("catalog").unwrap(),
                q::Ident::new("items").unwrap(),
            ],
            None,
        )
        .unwrap();
        for scenario in &case.scenarios {
            if ["lossless-values", "one-bad-tenant"].contains(&scenario.name.as_str()) {
                continue;
            }
            // Batch is absent from the production IR. QT-C09 is deliberately not
            // lowered to a first-row single INSERT and counted as batch evidence.
            let filter = plan_filter(&t, schema, &scenario.shape.filter);
            let c = |name: &str| plan_column(&t, schema, name);
            let draft = match scenario.shape.operation {
                Operation::Select => q::Draft::select(
                    t.clone(),
                    scenario
                        .shape
                        .projection
                        .iter()
                        .map(|n| q::Projection::new(c(n), None))
                        .collect(),
                ),
                Operation::Delete => q::Draft::delete(t.clone()),
                Operation::Update => {
                    let data = if scenario.name == "forbidden-patch" {
                        scenario
                            .shape
                            .input_columns
                            .iter()
                            .zip(&scenario.shape.rows[0])
                            .map(|(name, input)| {
                                (
                                    c(name),
                                    if matches!(input, Input::Missing { .. }) {
                                        q::Input::Missing
                                    } else {
                                        q::Input::Value(plan_value(input).unwrap())
                                    },
                                )
                            })
                            .collect()
                    } else {
                        vec![(
                            c("items.status"),
                            q::Input::Value(q::Value::Text("updated_canary".into())),
                        )]
                    };
                    q::Draft::update(t.clone(), data)
                }
                _ => panic!("unregistered policy bridge operation"),
            };
            let op = if let Some(p) = filter.clone() {
                draft.filter(p)
            } else {
                draft
            }
            .freeze();
            for d in [q::Dialect::Postgres, q::Dialect::Mysql] {
                let b = q::DatabaseBinding::application_supplied(
                    q::DatabaseIdentity::new("db_a").unwrap(),
                    q::TargetAddress::new("route_a").unwrap(),
                    q::Ident::new("catalog").unwrap(),
                    d,
                );
                let g = q::Generation::new(1).unwrap();
                let mut declarations = vec![q::PolicyDeclaration::Tenant(c("items.tenant_id"))];
                // Minimal declaration extension is finite, not a callback or Any.
                declarations.push(q::PolicyDeclaration::Generated(c("items.generated")));
                declarations.push(q::PolicyDeclaration::Readonly(c("items.readonly")));
                let facts = q::ScopeFacts::application_asserted(
                    t.clone(),
                    d,
                    schema.columns.iter().map(|x| c(&x.name)).collect(),
                    vec![q::Key::Primary(
                        schema.primary_key.iter().map(|n| c(n)).collect(),
                    )],
                    q::TableContext::OrdinaryBaseTableWithoutEffects,
                )
                .unwrap();
                let context = q::DatabaseContext::new(
                    q::Settings::new(
                        b.clone(),
                        q::SettingsIdentity::new("settings").unwrap(),
                        g,
                        q::PolicySet::new(
                            b.clone(),
                            q::PolicyIdentity::new("policy").unwrap(),
                            g,
                            declarations,
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                    q::ApplicationContext::application_supplied(
                        b.clone(),
                        Some(
                            q::SchemaSnapshot::application_supplied(
                                b.clone(),
                                q::SchemaIdentity::new("schema").unwrap(),
                                g,
                                vec![facts],
                            )
                            .unwrap(),
                        ),
                        None,
                        Some(
                            q::TenantSnapshot::application_supplied(
                                b.clone(),
                                c("items.tenant_id"),
                                q::Value::I64(1),
                            )
                            .unwrap(),
                        ),
                    )
                    .unwrap(),
                )
                .unwrap();
                let result = context.prepare().prepare_operation(&op);
                measured += 1;
                if scenario.name == "forbidden-patch" {
                    assert_eq!(result.unwrap_err(), q::PolicyError::TenantWrite);
                    continue;
                }
                let p = result.unwrap();
                let i = p.inspect();
                // Literal expectations for this limited API, independent of the
                // fixture's future dispatcher/provider/transaction verdicts.
                let (scope, risk) = match scenario.name.as_str() {
                    "no-where-update" | "no-where-delete" => (q::Scope::Broad, q::Risk::Blocked),
                    "tenant-wide" | "id-positive" | "partial-composite" | "tenant-leak" => {
                        (q::Scope::Broad, q::Risk::High)
                    }
                    "multi-in" => (q::Scope::Broad, q::Risk::Destructive),
                    "complete-composite" | "same-composite-or" => (q::Scope::Single, q::Risk::Low),
                    "different-composite-or" => (q::Scope::FiniteBound(2), q::Risk::High),
                    "unproven-or" => (q::Scope::Unknown, q::Risk::High),
                    "auto-wrap" => (q::Scope::Broad, q::Risk::Medium),
                    "not-null-empty-in" => (q::Scope::Zero, q::Risk::Medium),
                    "nullable-unique" => (q::Scope::Zero, q::Risk::Low),
                    _ => panic!("unregistered policy bridge expectation"),
                };
                assert_eq!((i.scope(), i.risk()), (scope, risk), "{}", scenario.name);
                let expected_filter = q::Predicate::And(
                    filter
                        .clone()
                        .into_iter()
                        .chain([q::Predicate::Compare(
                            c("items.tenant_id"),
                            q::Compare::Eq,
                            q::Value::I64(1),
                        )])
                        .collect(),
                );
                assert_eq!(
                    i.sensitive_source().metadata().predicate(),
                    Some(&expected_filter)
                );
                let original = op.finalize(d).unwrap();
                let mut args = original.sensitive_source().args().to_vec();
                args.push(q::Value::I64(1));
                assert_eq!(i.sensitive_source().args(), args);
                assert_eq!(p.sensitive_source().original_operation(), &op);
                assert!(p.sensitive_source().context().same_snapshot(&context));
                assert_eq!(op.finalize(d).unwrap(), original);
            }
        }
    }
    assert_eq!(measured, 30); // 15 supported/refused source shapes × two dialects.
}
