use quent_core::*;
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn table() -> Table {
    Table::new(vec![id("app"), id("items")], None).unwrap()
}
fn key() -> Column {
    table().column(id("id"), SqlType::I64, false)
}
fn version() -> Column {
    table().column(id("version"), SqlType::I32, false)
}
fn write() -> Draft {
    Draft::update(
        table(),
        vec![(
            table().column(id("text"), SqlType::Text, false),
            Input::Value(Value::Text("canary_value".into())),
        )],
    )
}
fn predicate() -> Predicate {
    Predicate::Compare(key(), Compare::Eq, Value::I64(7))
}
#[test]
fn version_and_returning_share_one_final_record() {
    let op = write()
        .filter(predicate())
        .optimistic_version(version(), Value::I32(4))
        .unwrap()
        .returning(vec![
            Projection::new(key(), None),
            Projection::new(version(), Some(id("next"))),
        ])
        .freeze();
    let plan = op.finalize(Dialect::Postgres).unwrap();
    let source = plan.sensitive_source();
    assert_eq!(
        source.sql(),
        "UPDATE \"app\".\"items\" SET \"text\" = $1, \"version\" = $2 WHERE (\"items\".\"id\" = $3) AND (\"items\".\"version\" = $4) RETURNING \"items\".\"id\", \"items\".\"version\" AS \"next\""
    );
    assert_eq!(
        source.args(),
        [
            Value::Text("canary_value".into()),
            Value::I32(5),
            Value::I64(7),
            Value::I32(4)
        ]
    );
    assert_eq!(
        source.result_columns(),
        [(SqlType::I64, false), (SqlType::I32, false)]
    );
    assert_eq!(
        source.metadata().ordered_write_data().last(),
        Some(&(version(), Input::Value(Value::I32(5))))
    );
    assert_eq!(
        source.metadata().sensitive_version(),
        Some((&version(), &Value::I32(4)))
    );
    assert!(
        source
            .metadata()
            .operation()
            .same_bounded_input(&op)
            .unwrap()
    );
    assert!(!format!("{op:?} {plan:?} {:?}", source.metadata()).contains("canary"));
    assert_eq!(
        op.finalize(Dialect::Mysql).unwrap_err(),
        PlanError::Unsupported
    );
}
#[test]
fn versions_cannot_supply_missing_key_proof_or_missing_where() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let facts = ScopeFacts::application_asserted(
            table(),
            d,
            vec![
                key(),
                version(),
                table().column(id("text"), SqlType::Text, false),
            ],
            vec![Key::Primary(vec![key()]), Key::Unique(vec![version()])],
            TableContext::OrdinaryBaseTableWithoutEffects,
        )
        .unwrap();
        let op = write()
            .optimistic_version(version(), Value::I32(1))
            .unwrap()
            .freeze();
        assert_eq!(
            op.finalize(d).unwrap().inspect(Some(&facts)).risk(),
            Risk::Blocked
        );
        let op = write()
            .filter(Predicate::Compare(key(), Compare::Gt, Value::I64(0)))
            .optimistic_version(version(), Value::I32(1))
            .unwrap()
            .freeze();
        let plan = op.finalize(d).unwrap();
        assert_eq!(plan.inspect(Some(&facts)).scope(), Scope::Broad);
        assert_eq!(plan.inspect(Some(&facts)).risk(), Risk::High);
        let op = write()
            .filter(predicate())
            .optimistic_version(version(), Value::I32(1))
            .unwrap()
            .freeze();
        let plan = op.finalize(d).unwrap();
        assert_eq!(plan.inspect(Some(&facts)).scope(), Scope::Single);
    }
}
#[test]
fn finite_version_domain_and_duplicate_assignments_refuse() {
    for value in [
        Value::I8(i8::MAX),
        Value::I16(i16::MAX),
        Value::I32(i32::MAX),
        Value::I64(i64::MAX),
        Value::U64(0),
        Value::Null(SqlType::I32),
        Value::Bool(false),
        Value::Text("x".into()),
    ] {
        let c = table().column(id("v"), value.sql_type(), false);
        assert!(write().optimistic_version(c, value).is_err());
    }
    assert!(
        write()
            .optimistic_version(table().column(id("v"), SqlType::I32, true), Value::I32(1))
            .is_err()
    );
    for (ty, value, next) in [
        (SqlType::I8, Value::I8(-2), Value::I8(-1)),
        (SqlType::I16, Value::I16(-2), Value::I16(-1)),
        (SqlType::I32, Value::I32(-2), Value::I32(-1)),
        (SqlType::I64, Value::I64(-2), Value::I64(-1)),
    ] {
        let op = write()
            .filter(predicate())
            .optimistic_version(table().column(id("v"), ty, false), value.clone())
            .unwrap()
            .freeze();
        let plan = op.finalize(Dialect::Mysql).unwrap();
        assert_eq!(
            plan.sensitive_source().args(),
            [
                Value::Text("canary_value".into()),
                next,
                Value::I64(7),
                value
            ]
        );
    }
    let op = Draft::update(table(), vec![(version(), Input::Value(Value::I32(2)))])
        .filter(predicate())
        .optimistic_version(version(), Value::I32(1))
        .unwrap()
        .freeze();
    assert_eq!(
        op.finalize(Dialect::Postgres).unwrap_err(),
        PlanError::Shape
    );
}
#[test]
fn returning_shape_capability_and_budgets_refuse() {
    let p = Projection::new(key(), None);
    for op in [
        write()
            .filter(predicate())
            .returning(vec![p.clone(), p.clone()])
            .freeze(),
        write()
            .filter(predicate())
            .returning(vec![p.clone(); 257])
            .freeze(),
        Draft::select(table(), vec![p.clone()])
            .returning(vec![p.clone()])
            .freeze(),
    ] {
        assert!(op.finalize(Dialect::Postgres).is_err());
    }
    let op = Draft::delete(table())
        .filter(predicate())
        .returning(vec![p])
        .freeze();
    assert_eq!(
        op.finalize(Dialect::Postgres)
            .unwrap()
            .sensitive_source()
            .result_columns(),
        [(SqlType::I64, false)]
    );
    assert_eq!(
        op.finalize(Dialect::Mysql).unwrap_err(),
        PlanError::Unsupported
    );
}
