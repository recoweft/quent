use quent_core::*;
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn table() -> Table {
    Table::new(vec![id("items")], None).unwrap()
}
fn col(t: &Table, s: &str, ty: SqlType, nullable: bool) -> Column {
    t.column(id(s), ty, nullable)
}
fn select(t: &Table) -> Draft {
    Draft::select(
        t.clone(),
        vec![Projection::new(col(t, "id", SqlType::I64, false), None)],
    )
}
fn value(v: Value) -> Input {
    Input::Value(v)
}
fn eq(c: Column, v: Value) -> Predicate {
    Predicate::Compare(c, Compare::Eq, v)
}
fn dialects() -> [Dialect; 2] {
    [Dialect::Postgres, Dialect::Mysql]
}
#[test]
fn five_families_exact_sql_args_and_metadata() {
    let t = table();
    let c = col(&t, "id", SqlType::I64, false);
    let p = eq(c.clone(), Value::I64(7));
    let drafts = [
        select(&t)
            .filter(p.clone())
            .order(vec![Order {
                column: c.clone(),
                direction: Direction::Desc,
            }])
            .limit(2),
        select(&t).filter(p.clone()).limit(3).count().unwrap(),
        Draft::insert(t.clone(), vec![(c.clone(), value(Value::I64(9)))]),
        Draft::update(t.clone(), vec![(c.clone(), value(Value::I64(9)))]).filter(p.clone()),
        Draft::delete(t.clone()).filter(p),
    ];
    let pg = [
        "SELECT \"items\".\"id\" FROM \"items\" WHERE (\"items\".\"id\" = $1) ORDER BY \"items\".\"id\" DESC LIMIT $2",
        "SELECT COUNT(*) FROM \"items\" WHERE (\"items\".\"id\" = $1)",
        "INSERT INTO \"items\" (\"id\") VALUES ($1)",
        "UPDATE \"items\" SET \"id\" = $1 WHERE (\"items\".\"id\" = $2)",
        "DELETE FROM \"items\" WHERE (\"items\".\"id\" = $1)",
    ];
    let expected_args = [
        vec![Value::I64(7), Value::I64(2)],
        vec![Value::I64(7)],
        vec![Value::I64(9)],
        vec![Value::I64(9), Value::I64(7)],
        vec![Value::I64(7)],
    ];
    for d in dialects() {
        for (i, draft) in drafts.iter().enumerate() {
            let op = draft.clone().freeze();
            let plan = op.finalize(d).unwrap();
            let s = plan.sensitive_source();
            let expected = if d == Dialect::Postgres {
                pg[i].into()
            } else {
                pg[i]
                    .replace('"', "`")
                    .replace("$1", "?")
                    .replace("$2", "?")
            };
            assert_eq!(s.sql(), expected);
            assert_eq!(s.args(), expected_args[i]);
            assert_eq!(
                s.metadata().family(),
                [
                    Family::Select,
                    Family::Count,
                    Family::Insert,
                    Family::Update,
                    Family::Delete
                ][i]
            );
            assert_eq!(s.metadata().operation(), &op);
            if i == 1 {
                assert_eq!(s.metadata().projection(), &FinalProjection::Count);
                assert!(s.metadata().order().is_empty());
                assert_eq!(s.metadata().limit(), None);
            }
            if i == 2 || i == 3 {
                assert_eq!(s.metadata().ordered_write_data().len(), 1);
            }

            assert_eq!(s.dialect(), d);
            assert_eq!(plan, op.finalize(d).unwrap());
        }
    }
}
#[test]
fn ordered_tree_null_three_valued_and_empty_in() {
    let t = table();
    let c = col(&t, "id", SqlType::I64, false);
    let email = col(&t, "email", SqlType::Text, true);
    let p = Predicate::And(vec![
        Predicate::Not(Box::new(eq(email.clone(), Value::Null(SqlType::Text)))),
        Predicate::Or(vec![
            Predicate::In(c.clone(), vec![]),
            Predicate::In(c, vec![Value::I64(0), Value::Null(SqlType::I64)]),
        ]),
        Predicate::IsNull(email),
    ]);
    let op = select(&t).filter(p.clone()).freeze();
    let pg = "SELECT \"items\".\"id\" FROM \"items\" WHERE ((NOT (\"items\".\"email\" = $1)) AND ((1 = 0) OR (\"items\".\"id\" IN ($2, $3))) AND (\"items\".\"email\" IS NULL))";
    for d in dialects() {
        let plan = op.finalize(d).unwrap();
        let s = plan.sensitive_source();
        let expected = if d == Dialect::Postgres {
            pg.into()
        } else {
            pg.replace('"', "`")
                .replace("$1", "?")
                .replace("$2", "?")
                .replace("$3", "?")
        };
        assert_eq!(s.sql(), expected);
        assert_eq!(
            s.args(),
            [
                Value::Null(SqlType::Text),
                Value::I64(0),
                Value::Null(SqlType::I64)
            ]
        );
        assert_eq!(s.metadata().predicate(), Some(&p));
        // SQL NULL comparison remains UNKNOWN under NOT; only explicit IS NULL is a null test.
        assert!(!s.sql().contains("IS NOT NULL"));
    }
}
#[test]
fn snapshot_independent_of_input_and_builder_changes() {
    let t = table();
    let mut text = String::from("before");
    let mut draft = Draft::insert(
        t.clone(),
        vec![(
            col(&t, "s", SqlType::Text, false),
            value(Value::Text(text.clone())),
        )],
    );
    let op = draft.clone().freeze();
    let plan = op.finalize(Dialect::Postgres).unwrap();
    text.clear();
    draft = Draft::delete(t);
    assert_eq!(plan, op.finalize(Dialect::Postgres).unwrap());
    assert_eq!(
        plan.sensitive_source().args(),
        [Value::Text("before".into())]
    );
    assert_ne!(plan, draft.freeze().finalize(Dialect::Postgres).unwrap());
}
#[test]
fn projection_alias_path_and_identifier_escaping() {
    let t = Table::new(vec![id("schema"), id("ta\"`ble")], Some(id("a\"`"))).unwrap();
    let c = col(&t, "co\"`l", SqlType::I64, false);
    let draft = Draft::select(
        t.clone(),
        vec![
            Projection::new(c.clone(), Some(id("out\"`"))),
            Projection::new(col(&t, "other", SqlType::Bool, false), None),
        ],
    );
    for (d, expected) in [
        (
            Dialect::Postgres,
            "SELECT \"a\"\"`\".\"co\"\"`l\" AS \"out\"\"`\", \"a\"\"`\".\"other\" FROM \"schema\".\"ta\"\"`ble\" AS \"a\"\"`\"",
        ),
        (
            Dialect::Mysql,
            "SELECT `a\"```.`co\"``l` AS `out\"```, `a\"```.`other` FROM `schema`.`ta\"``ble` AS `a\"```",
        ),
    ] {
        let plan = draft.clone().freeze().finalize(d).unwrap();
        assert_eq!(plan.sensitive_source().sql(), expected);
    }
    let other = table();
    assert_eq!(
        select(&other)
            .filter(eq(c, Value::I64(1)))
            .freeze()
            .finalize(Dialect::Postgres),
        Err(PlanError::Shape)
    );
    for s in ["", "a.b", "a b", "x;DROP", "x--", "x\0", "x()", "é"] {
        assert_eq!(Ident::new(s), Err(PlanError::Identifier));
    }
    assert_eq!(Ident::new("a".repeat(64)), Err(PlanError::Identifier));
    assert_eq!(Table::new(vec![], None), Err(PlanError::Shape));
}
#[test]
fn width_sign_boundaries_are_lossless_and_capability_checked() {
    let values = [
        Value::I8(i8::MIN),
        Value::I8(i8::MAX),
        Value::I16(i16::MIN),
        Value::I16(i16::MAX),
        Value::I32(i32::MIN),
        Value::I32(i32::MAX),
        Value::I64(i64::MIN),
        Value::I64(i64::MAX),
        Value::U8(u8::MAX),
        Value::U16(u16::MAX),
        Value::U32(u32::MAX),
        Value::U64(u64::MAX),
        Value::U64(0),
        Value::Bool(false),
        Value::Bool(true),
        Value::Text(String::new()),
        Value::Text("日本語".into()),
    ];
    let t = table();
    for v in values {
        for d in dialects() {
            let op = Draft::insert(
                t.clone(),
                vec![(col(&t, "v", v.sql_type(), false), value(v.clone()))],
            )
            .freeze();
            let plan = op.finalize(d);
            if d == Dialect::Postgres
                && matches!(
                    v.sql_type(),
                    SqlType::I8 | SqlType::U8 | SqlType::U16 | SqlType::U32 | SqlType::U64
                )
            {
                assert_eq!(plan, Err(PlanError::Unsupported));
            } else {
                assert_eq!(
                    plan.unwrap().sensitive_source().args(),
                    std::slice::from_ref(&v)
                );
            }
        }
    }
}
#[test]
fn missing_null_empty_zero_false_and_ordered_writes() {
    let t = table();
    let data = vec![
        (
            col(&t, "s", SqlType::Text, false),
            value(Value::Text(String::new())),
        ),
        (
            col(&t, "b", SqlType::Bool, false),
            value(Value::Bool(false)),
        ),
        (
            col(&t, "n", SqlType::I64, true),
            value(Value::Null(SqlType::I64)),
        ),
        (col(&t, "z", SqlType::I64, false), value(Value::I64(0))),
    ];
    for d in dialects() {
        let plan = Draft::insert(t.clone(), data.clone())
            .freeze()
            .finalize(d)
            .unwrap();
        assert_eq!(
            plan.sensitive_source().args(),
            [
                Value::Text(String::new()),
                Value::Bool(false),
                Value::Null(SqlType::I64),
                Value::I64(0)
            ]
        );
        let mut missing = data.clone();
        missing[0].1 = Input::Missing;
        assert_eq!(
            Draft::insert(t.clone(), missing).freeze().finalize(d),
            Err(PlanError::Missing)
        );
        let mut bad = data.clone();
        bad[0].1 = value(Value::Null(SqlType::Text));
        assert_eq!(
            Draft::insert(t.clone(), bad).freeze().finalize(d),
            Err(PlanError::Type)
        );
    }
}
#[test]
fn fixed_rejections_for_shapes_types_and_dialects() {
    let t = table();
    let c = col(&t, "id", SqlType::I64, false);
    let data = vec![(c.clone(), value(Value::I64(0)))];
    for d in dialects() {
        let cases = [
            (Draft::select(t.clone(), vec![]), PlanError::Shape),
            (Draft::insert(t.clone(), vec![]), PlanError::Shape),
            (Draft::update(t.clone(), vec![]), PlanError::Shape),
            (
                Draft::insert(t.clone(), [data.clone(), data.clone()].concat()),
                PlanError::Shape,
            ),
            (
                Draft::insert(t.clone(), vec![(c.clone(), value(Value::Bool(false)))]),
                PlanError::Type,
            ),
            (select(&t).filter(Predicate::And(vec![])), PlanError::Shape),
            (select(&t).filter(Predicate::Or(vec![])), PlanError::Shape),
            (Draft::delete(t.clone()).limit(1), PlanError::Unsupported),
            (
                Draft::update(t.clone(), data.clone()).order(vec![Order {
                    column: c.clone(),
                    direction: Direction::Asc,
                }]),
                PlanError::Unsupported,
            ),
            (
                Draft::insert(t.clone(), data.clone()).filter(Predicate::IsNull(c.clone())),
                PlanError::Shape,
            ),
            (select(&t).limit(u64::MAX), PlanError::Type),
        ];
        for (draft, error) in cases {
            assert_eq!(draft.freeze().finalize(d), Err(error));
        }
        let text = col(&t, "text", SqlType::Text, true);
        assert_eq!(
            select(&t)
                .filter(Predicate::Compare(
                    text,
                    Compare::Gt,
                    Value::Text("a".into())
                ))
                .freeze()
                .finalize(d),
            Err(PlanError::Unsupported)
        );
        let alias = Table::new(vec![id("items")], Some(id("a"))).unwrap();
        assert_eq!(
            Draft::delete(alias).freeze().finalize(d),
            Err(PlanError::Unsupported)
        );
        assert_eq!(
            select(&t)
                .filter(eq(col(&t, "id", SqlType::Bool, false), Value::Bool(false)))
                .freeze()
                .finalize(d),
            Err(PlanError::Type)
        );
    }
    assert_eq!(Draft::delete(t).count(), Err(PlanError::Shape));
}
#[test]
fn budgets_are_explicit_and_bounded() {
    let t = table();
    let c = col(&t, "id", SqlType::I64, false);
    for d in dialects() {
        let mut deep = Predicate::IsNull(c.clone());
        for _ in 0..17 {
            deep = Predicate::Not(Box::new(deep));
        }
        let many = Predicate::And(vec![
            Predicate::And(vec![eq(c.clone(), Value::I64(1)); 256]);
            5
        ]);
        for p in [
            deep,
            many,
            Predicate::In(c.clone(), vec![Value::I64(1); 257]),
        ] {
            assert_eq!(
                select(&t).filter(p).freeze().finalize(d),
                Err(PlanError::Budget)
            );
        }
        for (text, error) in [
            ("x".repeat(4097), PlanError::Budget),
            ("a\0b".into(), PlanError::Type),
        ] {
            assert_eq!(
                Draft::insert(
                    t.clone(),
                    vec![(col(&t, "s", SqlType::Text, false), value(Value::Text(text)))]
                )
                .freeze()
                .finalize(d),
                Err(error)
            );
        }
        assert_eq!(
            select(&t)
                .order(vec![
                    Order {
                        column: c.clone(),
                        direction: Direction::Asc
                    };
                    257
                ])
                .freeze()
                .finalize(d),
            Err(PlanError::Budget)
        );
    }
}
#[test]
fn all_comparisons_and_boolean_group_order() {
    let t = table();
    let c = col(&t, "id", SqlType::I64, false);
    for d in dialects() {
        for (op, token) in [
            (Compare::Eq, "="),
            (Compare::Ne, "<>"),
            (Compare::Lt, "<"),
            (Compare::Le, "<="),
            (Compare::Gt, ">"),
            (Compare::Ge, ">="),
        ] {
            let p = Predicate::Or(vec![
                Predicate::Compare(c.clone(), op, Value::I64(2)),
                eq(c.clone(), Value::I64(1)),
            ]);
            let plan = select(&t).filter(p.clone()).freeze().finalize(d).unwrap();
            let s = plan.sensitive_source();
            assert!(s.sql().contains(&format!(" {token} ")));
            assert_eq!(s.args(), [Value::I64(2), Value::I64(1)]);
            assert_eq!(s.metadata().predicate(), Some(&p));
        }
    }
}
#[test]
fn default_output_redacts_every_sensitive_carrier() {
    let t = Table::new(vec![id("canary_table")], Some(id("canary_alias"))).unwrap();
    let c = col(&t, "canary_column", SqlType::Text, true);
    let v = Value::Text("canary_value reason source_error".into());
    let p = eq(c.clone(), v.clone());
    let draft = Draft::select(t.clone(), vec![Projection::new(c.clone(), None)]).filter(p.clone());
    let op = draft.clone().freeze();
    let plan = op.finalize(Dialect::Postgres).unwrap();
    let output = format!(
        "{t:?} {c:?} {v:?} {p:?} {draft:?} {op:?} {plan:?} {:?} {:?} {:?}",
        plan.sensitive_source(),
        plan.sensitive_source().metadata(),
        plan.sensitive_source().args()
    );
    for s in ["canary", "SELECT", "source_error", "reason"] {
        assert!(!output.contains(s));
    }
    for error in [
        PlanError::Identifier,
        PlanError::Shape,
        PlanError::Type,
        PlanError::Missing,
        PlanError::Unsupported,
        PlanError::Budget,
    ] {
        assert!(std::error::Error::source(&error).is_none());
        assert!(error.to_string().starts_with("QUENT_PLAN_"));
    }
}

#[test]
fn projection_write_and_argument_budgets_and_alias_filters() {
    let t = table();
    let c = col(&t, "id", SqlType::I64, false);
    for d in dialects() {
        let long_table = Table::new(vec![id(&"\"".repeat(63))], None).unwrap();
        let long_col = col(&long_table, &"\"".repeat(63), SqlType::I64, false);
        let wide = Predicate::And(vec![
            Predicate::And(vec![
                Predicate::IsNull(long_col.clone());
                250
            ]);
            4
        ]);
        let huge_sql = Draft::select(long_table, vec![Projection::new(long_col, None)])
            .filter(wide)
            .freeze();
        assert_eq!(huge_sql.finalize(d), Err(PlanError::Budget));
        assert_eq!(
            Draft::select(t.clone(), vec![Projection::new(c.clone(), None); 257])
                .freeze()
                .finalize(d),
            Err(PlanError::Budget)
        );
        assert_eq!(
            Draft::insert(t.clone(), vec![(c.clone(), value(Value::I64(0))); 257])
                .freeze()
                .finalize(d),
            Err(PlanError::Budget)
        );
        assert_eq!(
            Draft::select(t.clone(), vec![Projection::new(c.clone(), None); 2])
                .freeze()
                .finalize(d),
            Err(PlanError::Shape)
        );
        let p = Predicate::And(
            (0..5)
                .map(|_| Predicate::In(c.clone(), vec![Value::I64(1); 256]))
                .collect(),
        );
        assert_eq!(
            select(&t).filter(p).freeze().finalize(d),
            Err(PlanError::Budget)
        );
        let alias = Table::new(vec![id("items")], Some(id("a"))).unwrap();
        let ac = col(&alias, "id", SqlType::I64, false);
        let plan = Draft::select(alias, vec![Projection::new(ac.clone(), Some(id("result")))])
            .filter(eq(ac.clone(), Value::I64(4)))
            .order(vec![Order {
                column: ac,
                direction: Direction::Asc,
            }])
            .limit(0)
            .freeze()
            .finalize(d)
            .unwrap();
        let expected = if d == Dialect::Postgres {
            "SELECT \"a\".\"id\" AS \"result\" FROM \"items\" AS \"a\" WHERE (\"a\".\"id\" = $1) ORDER BY \"a\".\"id\" ASC LIMIT $2".to_string()
        } else {
            "SELECT `a`.`id` AS `result` FROM `items` AS `a` WHERE (`a`.`id` = ?) ORDER BY `a`.`id` ASC LIMIT ?".to_string()
        };
        assert_eq!(plan.sensitive_source().sql(), expected);
        assert_eq!(
            plan.sensitive_source().args(),
            [Value::I64(4), Value::I64(0)]
        );
    }
}
