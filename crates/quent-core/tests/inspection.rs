use quent_core::*;
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn table() -> Table {
    Table::new(vec![id("scope_canary")], None).unwrap()
}
fn col(t: &Table, n: &str) -> Column {
    t.column(id(n), SqlType::I64, false)
}
fn eq(c: &Column, n: i64) -> Predicate {
    Predicate::Compare(c.clone(), Compare::Eq, Value::I64(n))
}
fn facts(t: &Table, d: Dialect, cs: Vec<Column>, keys: Vec<Key>) -> ScopeFacts {
    ScopeFacts::application_asserted(
        t.clone(),
        d,
        cs,
        keys,
        TableContext::OrdinaryBaseTableWithoutEffects,
    )
    .unwrap()
}
fn plan(t: &Table, p: Predicate, d: Dialect) -> Plan {
    Draft::delete(t.clone())
        .filter(p)
        .freeze()
        .finalize(d)
        .unwrap()
}
fn summary(
    i: &Inspection<'_>,
) -> (
    Scope,
    Completeness,
    Assumptions,
    Evidence,
    Risk,
    Vec<Finding>,
) {
    (
        i.scope(),
        i.completeness(),
        i.assumptions(),
        i.evidence(),
        i.risk(),
        i.findings().to_vec(),
    )
}
#[test]
fn full_key_or_and_upper_bounds_and_plan_identity_both_dialects() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let t = table();
        let a = col(&t, "key_a");
        let b = col(&t, "key_b");
        let tenant = col(&t, "tenant");
        let f = facts(
            &t,
            d,
            vec![a.clone(), b.clone(), tenant.clone()],
            vec![Key::Primary(vec![a.clone(), b.clone()])],
        );
        let key = |x, y| Predicate::And(vec![eq(&a, x), eq(&b, y)]);
        for (p, want) in [
            (key(1, 2), Scope::Single),
            (Predicate::Or(vec![key(1, 2), key(1, 2)]), Scope::Single),
            (
                Predicate::Or(vec![key(1, 2), key(1, 3)]),
                Scope::FiniteBound(2),
            ),
            (Predicate::Or(vec![key(1, 2), eq(&tenant, 1)]), Scope::Broad),
            (eq(&a, 1), Scope::Broad),
            (eq(&tenant, 1), Scope::Broad),
            (
                Predicate::Compare(a.clone(), Compare::Gt, Value::I64(0)),
                Scope::Broad,
            ),
            (
                Predicate::In(a.clone(), vec![Value::I64(1), Value::I64(2)]),
                Scope::Broad,
            ),
            (
                Predicate::And(vec![
                    Predicate::Or(vec![eq(&tenant, 1), eq(&tenant, 2)]),
                    key(1, 2),
                ]),
                Scope::Single,
            ),
            (Predicate::And(vec![eq(&a, 1), eq(&a, 2)]), Scope::Zero),
            (
                Predicate::And(vec![key(1, 2), Predicate::Or(vec![key(1, 2), key(3, 4)])]),
                Scope::Single,
            ),
        ] {
            let p0 = p.clone();
            let plan = plan(&t, p, d);
            let sql = plan.sensitive_source().sql().to_owned();
            let args = plan.sensitive_source().args().to_vec();
            let metadata = plan.sensitive_source().metadata().clone();
            let i = plan.inspect(Some(&f));
            assert_eq!(i.scope(), want);
            assert_eq!(i.assumptions(), Assumptions::ApplicationAsserted);
            assert_eq!(summary(&i), summary(&plan.inspect(Some(&f))));
            assert_eq!(i.sensitive_source().sql(), sql);
            assert_eq!(i.sensitive_source().args(), args);
            assert_eq!(i.sensitive_source().metadata(), &metadata);
            assert_eq!(metadata.predicate(), Some(&p0));
        }
    }
}
#[test]
fn nullable_unique_null_not_empty_in_and_independent_bound() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let t = table();
        let a = col(&t, "id");
        let n = t.column(id("nullable_canary"), SqlType::I64, true);
        let text = t.column(id("text_canary"), SqlType::Text, false);
        let cs = vec![a.clone(), n.clone(), text.clone()];
        let f = facts(
            &t,
            d,
            cs.clone(),
            vec![Key::Primary(vec![a.clone()]), Key::Unique(vec![n.clone()])],
        );
        for (p, want) in [
            (eq(&n, 1), Scope::Unknown),
            (Predicate::IsNull(n.clone()), Scope::Unknown),
            (Predicate::Not(Box::new(eq(&a, 1))), Scope::Unknown),
            (
                Predicate::Not(Box::new(Predicate::In(a.clone(), vec![]))),
                Scope::Unknown,
            ),
            (
                Predicate::Compare(n.clone(), Compare::Eq, Value::Null(SqlType::I64)),
                Scope::Zero,
            ),
            (
                Predicate::Not(Box::new(Predicate::Compare(
                    n.clone(),
                    Compare::Eq,
                    Value::Null(SqlType::I64),
                ))),
                Scope::Zero,
            ),
            (Predicate::In(n.clone(), vec![]), Scope::Zero),
            (
                Predicate::And(vec![eq(&a, 1), Predicate::Not(Box::new(eq(&n, 2)))]),
                Scope::Single,
            ),
            (
                Predicate::Or(vec![
                    eq(&a, 1),
                    Predicate::Compare(
                        text.clone(),
                        Compare::Eq,
                        Value::Text("value_canary".into()),
                    ),
                ]),
                Scope::Unknown,
            ),
        ] {
            let p = plan(&t, p, d);
            let i = p.inspect(Some(&f));
            assert_eq!(i.scope(), want);
            if want == Scope::Single {
                assert_eq!(i.completeness(), Completeness::Partial);
                assert_eq!(i.risk(), Risk::High);
            }
        }
        for key in [
            Key::Unique(vec![text.clone()]),
            Key::Unique(vec![n.clone()]),
        ] {
            let f = facts(&t, d, cs.clone(), vec![key]);
            assert_eq!(
                plan(&t, eq(&a, 1), d).inspect(Some(&f)).scope(),
                Scope::Unknown
            );
        }
        let unique = facts(&t, d, cs, vec![Key::Unique(vec![a.clone()])]);
        assert_eq!(
            plan(&t, eq(&a, 1), d).inspect(Some(&unique)).scope(),
            Scope::Single
        );
    }
}
#[test]
fn facts_missing_mismatch_context_and_no_name_authority() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let t = table();
        let a = col(&t, "id");
        let p = plan(&t, eq(&a, 1), d);
        assert_eq!(p.inspect(None).evidence(), Evidence::MissingFacts);
        let empty = facts(&t, d, vec![a.clone()], vec![]);
        assert_eq!(p.inspect(Some(&empty)).scope(), Scope::Unknown);
        let other = Table::new(vec![id("other_canary"), id("scope_canary")], None).unwrap();
        let alias = Table::new(vec![id("scope_canary")], Some(id("alias_canary"))).unwrap();
        let other_d = if d == Dialect::Postgres {
            Dialect::Mysql
        } else {
            Dialect::Postgres
        };
        for f in [
            facts(&other, d, vec![col(&other, "id")], vec![]),
            facts(&alias, d, vec![col(&alias, "id")], vec![]),
            facts(&t, other_d, vec![a.clone()], vec![]),
            facts(&t, d, vec![], vec![]),
            facts(&t, d, vec![t.column(id("id"), SqlType::I32, false)], vec![]),
            facts(&t, d, vec![t.column(id("id"), SqlType::I64, true)], vec![]),
        ] {
            let i = p.inspect(Some(&f));
            assert_eq!(i.scope(), Scope::Unknown);
            assert_eq!(i.evidence(), Evidence::InconsistentFacts);
        }
        let f = ScopeFacts::application_asserted(
            t.clone(),
            d,
            vec![a.clone()],
            vec![Key::Primary(vec![a.clone()])],
            TableContext::Unsupported,
        )
        .unwrap();
        assert_eq!(p.inspect(Some(&f)).evidence(), Evidence::UnsupportedContext);
        assert_eq!(
            plan(&t, Predicate::In(a.clone(), vec![]), d)
                .inspect(Some(&f))
                .scope(),
            Scope::Unknown
        );
        assert!(
            ScopeFacts::application_asserted(
                t.clone(),
                d,
                vec![a.clone(), a.clone()],
                vec![],
                TableContext::Unsupported
            )
            .is_err()
        );
        assert!(
            ScopeFacts::application_asserted(
                t.clone(),
                d,
                vec![a.clone()],
                vec![Key::Primary(vec![])],
                TableContext::Unsupported
            )
            .is_err()
        );
        assert!(
            ScopeFacts::application_asserted(
                t.clone(),
                d,
                vec![a.clone()],
                vec![Key::Primary(vec![col(&other, "id")])],
                TableContext::Unsupported
            )
            .is_err()
        );
        // Alias identity is supported when all facts match (reads only in PR1).
        let c = col(&alias, "id");
        let f = facts(
            &alias,
            d,
            vec![c.clone()],
            vec![Key::Primary(vec![c.clone()])],
        );
        let read = Draft::select(alias, vec![Projection::new(c.clone(), None)])
            .filter(eq(&c, 1))
            .freeze()
            .finalize(d)
            .unwrap();
        assert_eq!(read.inspect(Some(&f)).scope(), Scope::Single);
    }
}
#[test]
fn fixed_risk_missing_where_cannot_be_suppressed_or_publicly_forged() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let t = table();
        let a = col(&t, "id");
        let f = facts(&t, d, vec![a.clone()], vec![Key::Primary(vec![a.clone()])]);
        for draft in [
            Draft::delete(t.clone()),
            Draft::update(t.clone(), vec![(a.clone(), Input::Value(Value::I64(7)))]),
        ] {
            let p = draft.freeze().finalize(d).unwrap();
            for facts in [None, Some(&f)] {
                let i = p.inspect(facts);
                assert_eq!(i.risk(), Risk::Blocked);
                assert_eq!(i.findings()[0].code(), FindingCode::MissingWhere);
                assert!(!i.findings()[0].suppressible());
                let mut displayed = i.findings().to_vec();
                displayed.clear();
                let mut displayed_risk = i.risk();
                assert_eq!(displayed_risk, Risk::Blocked);
                displayed_risk = Risk::Low;
                assert_ne!(displayed_risk, p.inspect(facts).risk());
            }
        }
        let p = plan(&t, eq(&a, 1), d);
        assert_eq!(p.inspect(Some(&f)).risk(), Risk::Low);
        let finite = plan(&t, Predicate::Or(vec![eq(&a, 1), eq(&a, 2)]), d);
        assert_eq!(
            finite.inspect(Some(&f)).findings()[0].code(),
            FindingCode::BulkWrite
        );
        let broad = plan(
            &t,
            Predicate::Compare(a.clone(), Compare::Gt, Value::I64(0)),
            d,
        );
        assert_eq!(broad.inspect(Some(&f)).risk(), Risk::Destructive);
        let read = Draft::select(t.clone(), vec![Projection::new(a.clone(), None)])
            .freeze()
            .finalize(d)
            .unwrap();
        assert_eq!(read.inspect(Some(&f)).risk(), Risk::Medium);
        assert_eq!(
            read.inspect(Some(&f))
                .findings()
                .iter()
                .map(Finding::code)
                .collect::<Vec<_>>(),
            [FindingCode::ReadProjection, FindingCode::ReadWithoutLimit]
        );
        let read = Draft::select(t.clone(), vec![Projection::new(a.clone(), None)])
            .limit(1)
            .freeze()
            .finalize(d)
            .unwrap();
        assert_eq!(read.inspect(Some(&f)).scope(), Scope::Broad);
        assert_eq!(read.inspect(Some(&f)).risk(), Risk::Low);
        assert_eq!(read.inspect(None).risk(), Risk::High);
        let count = Draft::select(t.clone(), vec![Projection::new(a.clone(), None)])
            .limit(1)
            .count()
            .unwrap()
            .freeze()
            .finalize(d)
            .unwrap();
        assert_eq!(
            count.inspect(Some(&f)).findings()[0].code(),
            FindingCode::ReadWithoutLimit
        );
        let insert = Draft::insert(t, vec![(a, Input::Value(Value::I64(1)))])
            .freeze()
            .finalize(d)
            .unwrap();
        assert_eq!(insert.inspect(Some(&f)).risk(), Risk::High);
        assert_eq!(insert.inspect(Some(&f)).scope(), Scope::NotApplicable);
    }
}
#[test]
fn canaries_redacted_and_assertion_snapshot_owned() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let t = table();
        let a = col(&t, "identifier_canary");
        let mut cs = vec![a.clone()];
        let mut keys = vec![Key::Primary(cs.clone())];
        let f = facts(&t, d, cs.clone(), keys.clone());
        cs.clear();
        keys.clear();
        let p = plan(&t, eq(&a, 938457234), d);
        let i = p.inspect(Some(&f));
        assert_eq!(i.scope(), Scope::Single);
        let output = format!(
            "{f:?} {i:?} {:?} {:?} {:?} {:?}",
            Key::Primary(vec![a]),
            i.sensitive_source(),
            i.findings(),
            PlanError::Shape
        );
        for c in ["scope_canary", "identifier_canary", "938457234", "DELETE"] {
            assert!(!output.contains(c));
        }
        assert_eq!(format!("{}", PlanError::Shape), "QUENT_PLAN_Shape");
    }
}
#[test]
fn signed_widths_unsigned_refusal_and_bounded_expansion() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let t = table();
        for (ty, v) in [
            (SqlType::I16, Value::I16(i16::MIN)),
            (SqlType::I32, Value::I32(i32::MAX)),
            (SqlType::I64, Value::I64(i64::MIN)),
        ] {
            let a = t.column(id("key"), ty, false);
            let f = facts(&t, d, vec![a.clone()], vec![Key::Primary(vec![a.clone()])]);
            assert_eq!(
                plan(&t, Predicate::Compare(a, Compare::Eq, v), d)
                    .inspect(Some(&f))
                    .scope(),
                Scope::Single
            );
        }
        if d == Dialect::Mysql {
            for (ty, v, want) in [
                (SqlType::I8, Value::I8(i8::MIN), Scope::Single),
                (SqlType::U64, Value::U64(u64::MAX), Scope::Unknown),
            ] {
                let a = t.column(id("key"), ty, false);
                let f = facts(&t, d, vec![a.clone()], vec![Key::Primary(vec![a.clone()])]);
                assert_eq!(
                    plan(&t, Predicate::Compare(a, Compare::Eq, v), d)
                        .inspect(Some(&f))
                        .scope(),
                    want
                );
            }
        }
        let a = col(&t, "key");
        let b = col(&t, "other");
        let f = facts(
            &t,
            d,
            vec![a.clone(), b.clone()],
            vec![Key::Primary(vec![a.clone()])],
        );
        let p = Predicate::And(vec![
            Predicate::Or((0..17).map(|n| eq(&a, n)).collect()),
            Predicate::Or((0..17).map(|n| eq(&b, n)).collect()),
        ]);
        let p = plan(&t, p, d);
        assert_eq!(p.inspect(Some(&f)).evidence(), Evidence::AnalysisBudget);
        assert_eq!(p.inspect(Some(&f)).risk(), Risk::High);
    }
}

// Independent finite SQL three-valued evaluator: tests TRUE-row bounds against
// actual predicate semantics rather than the analyzer's own branch abstraction.
#[test]
fn three_valued_truth_table_checks_nested_bounds() {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Truth {
        T,
        F,
        U,
    }
    fn not(t: Truth) -> Truth {
        match t {
            Truth::T => Truth::F,
            Truth::F => Truth::T,
            Truth::U => Truth::U,
        }
    }
    fn eval(p: &Predicate, row: i64) -> Truth {
        use Truth::*;
        match p {
            Predicate::And(ps) => {
                let ts = ps.iter().map(|p| eval(p, row)).collect::<Vec<_>>();
                if ts.contains(&F) {
                    F
                } else if ts.contains(&U) {
                    U
                } else {
                    T
                }
            }
            Predicate::Or(ps) => {
                let ts = ps.iter().map(|p| eval(p, row)).collect::<Vec<_>>();
                if ts.contains(&T) {
                    T
                } else if ts.contains(&U) {
                    U
                } else {
                    F
                }
            }
            Predicate::Not(p) => not(eval(p, row)),
            Predicate::Compare(_, _, Value::Null(_)) => U,
            Predicate::Compare(_, op, Value::I64(n)) => {
                if match op {
                    Compare::Eq => row == *n,
                    Compare::Ne => row != *n,
                    Compare::Lt => row < *n,
                    Compare::Le => row <= *n,
                    Compare::Gt => row > *n,
                    Compare::Ge => row >= *n,
                } {
                    T
                } else {
                    F
                }
            }
            Predicate::In(_, vs) => {
                if vs.contains(&Value::I64(row)) {
                    T
                } else if vs.iter().any(|v| matches!(v, Value::Null(_))) {
                    U
                } else {
                    F
                }
            }
            Predicate::IsNull(_) => F,
            _ => panic!("unsupported oracle input"),
        }
    }
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let t = table();
        let a = col(&t, "key");
        let f = facts(&t, d, vec![a.clone()], vec![Key::Primary(vec![a.clone()])]);
        let atoms = vec![
            eq(&a, 0),
            eq(&a, 1),
            Predicate::Compare(a.clone(), Compare::Gt, Value::I64(0)),
            Predicate::Compare(a.clone(), Compare::Eq, Value::Null(SqlType::I64)),
            Predicate::In(a.clone(), vec![]),
            Predicate::In(a.clone(), vec![Value::I64(1), Value::Null(SqlType::I64)]),
            Predicate::IsNull(a),
        ];
        let mut checked = 0;
        for x in &atoms {
            for y in &atoms {
                for p in [
                    Predicate::And(vec![x.clone(), y.clone()]),
                    Predicate::Or(vec![x.clone(), y.clone()]),
                    Predicate::Not(Box::new(Predicate::Or(vec![x.clone(), y.clone()]))),
                    Predicate::And(vec![
                        eq(&col(&t, "key"), 0),
                        Predicate::Not(Box::new(Predicate::Or(vec![x.clone(), y.clone()]))),
                    ]),
                ] {
                    let actual = (-2..=2).filter(|n| eval(&p, *n) == Truth::T).count();
                    let p = plan(&t, p, d);
                    match p.inspect(Some(&f)).scope() {
                        Scope::Zero => assert_eq!(actual, 0),
                        Scope::Single => assert!(actual <= 1),
                        Scope::FiniteBound(n) => assert!(actual <= usize::from(n)),
                        _ => (),
                    }
                    checked += 1;
                }
            }
        }
        assert_eq!(checked, 196);
    }
}
