use quent_core::*;
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn table() -> Table {
    Table::new(vec![id("catalog"), id("items")], None).unwrap()
}
fn col(s: &str) -> Column {
    table().column(
        id(s),
        match s {
            "deleted" => SqlType::Bool,
            "secret" => SqlType::Text,
            _ => SqlType::I64,
        },
        false,
    )
}
fn eq(s: &str, v: i64) -> Predicate {
    Predicate::Compare(col(s), Compare::Eq, Value::I64(v))
}
fn data(s: &str, v: i64) -> (Column, Input) {
    (col(s), Input::Value(Value::I64(v)))
}
fn binding(d: Dialect, db: &str) -> DatabaseBinding {
    DatabaseBinding::application_supplied(
        DatabaseIdentity::new(db).unwrap(),
        TargetAddress::new("route").unwrap(),
        id("catalog"),
        d,
    )
}
fn facts(d: Dialect) -> ScopeFacts {
    ScopeFacts::application_asserted(
        table(),
        d,
        [
            "id",
            "tenant",
            "value",
            "deleted",
            "secret",
            "ro",
            "generated",
        ]
        .map(col)
        .to_vec(),
        vec![Key::Primary(vec![col("tenant"), col("id")])],
        TableContext::OrdinaryBaseTableWithoutEffects,
    )
    .unwrap()
}
fn context(d: Dialect, tenant: Option<i64>, extra: Vec<PolicyDeclaration>) -> DatabaseContext {
    configured(d, "A", tenant, Some(vec![facts(d)]), extra)
}
fn configured(
    d: Dialect,
    db: &str,
    tenant: Option<i64>,
    fs: Option<Vec<ScopeFacts>>,
    extra: Vec<PolicyDeclaration>,
) -> DatabaseContext {
    let b = binding(d, db);
    let g = Generation::new(1).unwrap();
    let mut ds = vec![PolicyDeclaration::Tenant(col("tenant"))];
    ds.extend(extra);
    DatabaseContext::new(
        Settings::new(
            b.clone(),
            SettingsIdentity::new("settings").unwrap(),
            g,
            PolicySet::new(b.clone(), PolicyIdentity::new("policy").unwrap(), g, ds).unwrap(),
        )
        .unwrap(),
        ApplicationContext::application_supplied(
            b.clone(),
            fs.map(|fs| {
                SchemaSnapshot::application_supplied(
                    b.clone(),
                    SchemaIdentity::new("schema").unwrap(),
                    g,
                    fs,
                )
                .unwrap()
            }),
            Some(SubjectIdentity::new("CANARY_SUBJECT").unwrap()),
            tenant.map(|n| {
                TenantSnapshot::application_supplied(b.clone(), col("tenant"), Value::I64(n))
                    .unwrap()
            }),
        )
        .unwrap(),
    )
    .unwrap()
}
fn update(p: Option<Predicate>) -> Operation {
    let d = Draft::update(table(), vec![data("value", 9)]);
    p.map_or_else(|| d.clone().freeze(), |p| d.clone().filter(p).freeze())
}
fn prepare(c: &DatabaseContext, op: &Operation) -> Result<PolicyPreparation, PolicyError> {
    c.prepare().prepare_operation(op)
}
fn assert_error(c: &DatabaseContext, op: &Operation, e: PolicyError) {
    assert_eq!(prepare(c, op).unwrap_err(), e);
}

#[test]
fn five_families_one_record_exact_sql_args_metadata_and_original() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, Some(11), vec![]);
        let operations = [
            Draft::select(table(), vec![Projection::new(col("id"), None)])
                .filter(eq("id", 1))
                .freeze(),
            Draft::select(table(), vec![Projection::new(col("id"), None)])
                .filter(eq("id", 1))
                .count()
                .unwrap()
                .freeze(),
            Draft::insert(table(), vec![data("value", 9)]).freeze(),
            update(Some(eq("id", 1))),
            Draft::delete(table()).filter(eq("id", 1)).freeze(),
        ];
        let sqls = [
            "SELECT {q}items{q}.{q}id{q} FROM {q}catalog{q}.{q}items{q} WHERE (({q}items{q}.{q}id{q} = @1) AND ({q}items{q}.{q}tenant{q} = @2))",
            "SELECT COUNT(*) FROM {q}catalog{q}.{q}items{q} WHERE (({q}items{q}.{q}id{q} = @1) AND ({q}items{q}.{q}tenant{q} = @2))",
            "INSERT INTO {q}catalog{q}.{q}items{q} ({q}value{q}, {q}tenant{q}) VALUES (@1, @2)",
            "UPDATE {q}catalog{q}.{q}items{q} SET {q}value{q} = @1 WHERE (({q}items{q}.{q}id{q} = @2) AND ({q}items{q}.{q}tenant{q} = @3))",
            "DELETE FROM {q}catalog{q}.{q}items{q} WHERE (({q}items{q}.{q}id{q} = @1) AND ({q}items{q}.{q}tenant{q} = @2))",
        ];
        for (n, op) in operations.iter().enumerate() {
            let raw = op.finalize(d).unwrap();
            let prepared = prepare(&c, op).unwrap();
            let source = prepared.sensitive_source();
            let final_ = source.final_source();
            let mut expected =
                sqls[n].replace("{q}", if d == Dialect::Postgres { "\"" } else { "`" });
            for i in 1..=3 {
                expected = expected.replace(
                    &format!("@{i}"),
                    &if d == Dialect::Postgres {
                        format!("${i}")
                    } else {
                        "?".into()
                    },
                );
            }
            assert_eq!(final_.sql(), expected);
            let args = match n {
                2 => vec![Value::I64(9), Value::I64(11)],
                3 => vec![Value::I64(9), Value::I64(1), Value::I64(11)],
                _ => vec![Value::I64(1), Value::I64(11)],
            };
            assert_eq!(final_.args(), args);
            assert_eq!(
                final_.metadata().family(),
                [
                    Family::Select,
                    Family::Count,
                    Family::Insert,
                    Family::Update,
                    Family::Delete
                ][n]
            );
            assert_eq!(prepared.inspect().sensitive_source().sql(), expected);
            assert_eq!(prepared.inspect().sensitive_source().args(), args);
            assert_eq!(
                prepared.inspect().sensitive_source().metadata(),
                final_.metadata()
            );
            assert_eq!(source.original_operation(), op);
            assert!(source.context().same_snapshot(&c));
            assert_eq!(op.finalize(d).unwrap(), raw);
            assert_eq!(prepare(&c, op).unwrap(), prepared);
        }
    }
}
#[test]
fn c06_outer_and_all_occurrences_not_first_or_branch() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, Some(11), vec![]);
        for (second, expected_arg) in [(eq("id", 2), 2), (eq("tenant", 11), 11)] {
            let caller = Predicate::Or(vec![
                Predicate::And(vec![eq("tenant", 11), eq("id", 1)]),
                second,
            ]);
            let op = update(Some(caller.clone()));
            let p = prepare(&c, &op).unwrap();
            assert_eq!(
                p.sensitive_source().final_source().metadata().predicate(),
                Some(&Predicate::And(vec![caller, eq("tenant", 11)]))
            );
            // All matching branches are guarded; original key OR is retained.
            assert_eq!(
                p.sensitive_source().final_source().args(),
                &[
                    Value::I64(9),
                    Value::I64(11),
                    Value::I64(1),
                    Value::I64(expected_arg),
                    Value::I64(11)
                ]
            );
        }
        for p in [
            Predicate::Or(vec![eq("tenant", 11), eq("tenant", 22)]),
            Predicate::And(vec![eq("id", 1), eq("id", 2), eq("tenant", 22)]),
            Predicate::In(col("tenant"), vec![Value::I64(11), Value::I64(22)]),
        ] {
            assert_error(&c, &update(Some(p)), PolicyError::TenantMismatch);
        }
        for p in [
            Predicate::Not(Box::new(eq("tenant", 11))),
            Predicate::Compare(col("tenant"), Compare::Gt, Value::I64(0)),
            Predicate::IsNull(col("tenant")),
        ] {
            assert_error(&c, &update(Some(p)), PolicyError::TenantPredicate);
        }
        assert_error(
            &c,
            &update(Some(Predicate::Compare(
                col("tenant"),
                Compare::Eq,
                Value::Null(SqlType::I64),
            ))),
            PolicyError::TenantMismatch,
        );
        assert!(
            prepare(
                &c,
                &update(Some(Predicate::In(
                    col("tenant"),
                    vec![Value::I64(11), Value::I64(11)]
                )))
            )
            .is_ok()
        );
    }
}
#[test]
fn c09_c10_single_insert_all_values_and_protected_writes() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, Some(11), vec![]);
        for n in [11, 22] {
            assert_error(
                &c,
                &Draft::update(table(), vec![data("value", 9), data("tenant", n)])
                    .filter(eq("id", 1))
                    .freeze(),
                PolicyError::TenantWrite,
            );
        }
        for (input, ok) in [
            (Input::Value(Value::I64(11)), true),
            (Input::Value(Value::I64(22)), false),
            (Input::Value(Value::Null(SqlType::I64)), false),
            (Input::Value(Value::I32(11)), false),
            (Input::Missing, false),
        ] {
            let op =
                Draft::insert(table(), vec![data("value", 9), (col("tenant"), input)]).freeze();
            if ok {
                assert_eq!(
                    prepare(&c, &op)
                        .unwrap()
                        .sensitive_source()
                        .final_source()
                        .args(),
                    &[Value::I64(9), Value::I64(11)]
                );
            } else {
                assert_error(&c, &op, PolicyError::TenantMismatch);
            }
        }
        let duplicate =
            Draft::insert(table(), vec![data("tenant", 11), data("tenant", 22)]).freeze();
        assert_error(&c, &duplicate, PolicyError::TenantMismatch);
        let c = context(
            d,
            Some(11),
            vec![
                PolicyDeclaration::Readonly(col("ro")),
                PolicyDeclaration::Generated(col("generated")),
            ],
        );
        for s in ["ro", "generated"] {
            for op in [
                Draft::insert(table(), vec![data("value", 9), data(s, 1)]).freeze(),
                Draft::update(table(), vec![data("value", 9), data(s, 1)])
                    .filter(eq("id", 1))
                    .freeze(),
            ] {
                assert_error(&c, &op, PolicyError::ProtectedWrite);
            }
        }
        assert!(prepare(&c, &update(Some(eq("id", 1)))).is_ok());
    }
}
#[test]
fn soft_delete_is_outer_filter_and_delete_remains_physical() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(
            d,
            Some(11),
            vec![PolicyDeclaration::SoftDelete(col("deleted"))],
        );
        let caller = Predicate::Or(vec![eq("id", 1), eq("id", 2)]);
        let op = Draft::delete(table()).filter(caller.clone()).freeze();
        let p = prepare(&c, &op).unwrap();
        let expected = Predicate::And(vec![
            caller,
            eq("tenant", 11),
            Predicate::Compare(col("deleted"), Compare::Eq, Value::Bool(false)),
        ]);
        assert_eq!(
            p.sensitive_source().final_source().metadata().predicate(),
            Some(&expected)
        );
        assert_eq!(
            p.sensitive_source().final_source().args(),
            &[
                Value::I64(1),
                Value::I64(2),
                Value::I64(11),
                Value::Bool(false)
            ]
        );
        assert!(
            p.sensitive_source()
                .final_source()
                .sql()
                .starts_with("DELETE FROM")
        );
        assert_eq!(p.inspect().completeness(), Completeness::Partial);
        assert_ne!(p.inspect().risk(), Risk::Low);
        let insert = Draft::insert(table(), vec![data("value", 9)]).freeze();
        assert!(
            prepare(&c, &insert)
                .unwrap()
                .sensitive_source()
                .final_source()
                .metadata()
                .predicate()
                .is_none()
        );
    }
}
#[test]
fn pii_and_required_filter_positive_negative_all_branches() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, Some(11), vec![PolicyDeclaration::Pii(col("secret"))]);
        assert!(prepare(&c, &update(Some(eq("id", 1)))).is_ok());
        for op in [
            Draft::select(table(), vec![Projection::new(col("secret"), None)]).freeze(),
            Draft::select(table(), vec![Projection::new(col("id"), None)])
                .filter(Predicate::IsNull(col("secret")))
                .count()
                .unwrap()
                .freeze(),
            Draft::update(
                table(),
                vec![(
                    col("secret"),
                    Input::Value(Value::Text("CANARY_VALUE".into())),
                )],
            )
            .filter(eq("id", 1))
            .freeze(),
            Draft::insert(
                table(),
                vec![(
                    col("secret"),
                    Input::Value(Value::Text("CANARY_VALUE".into())),
                )],
            )
            .freeze(),
            Draft::delete(table()).filter(eq("id", 1)).freeze(),
            Draft::select(table(), vec![Projection::new(col("id"), None)])
                .order(vec![Order {
                    column: col("secret"),
                    direction: Direction::Asc,
                }])
                .freeze(),
        ] {
            assert_error(&c, &op, PolicyError::PiiAccess);
        }
        let c = context(
            d,
            Some(11),
            vec![PolicyDeclaration::RequiredFilter(col("id"))],
        );
        for p in [
            eq("id", 1),
            Predicate::Or(vec![eq("id", 1), eq("id", 2)]),
            Predicate::And(vec![eq("id", 1), Predicate::Not(Box::new(eq("value", 2)))]),
        ] {
            assert!(prepare(&c, &update(Some(p))).is_ok());
        }
        for p in [
            None,
            Some(Predicate::Or(vec![eq("id", 1), eq("value", 2)])),
            Some(Predicate::Not(Box::new(eq("id", 1)))),
            Some(Predicate::Compare(col("id"), Compare::Gt, Value::I64(1))),
            Some(Predicate::Compare(
                col("id"),
                Compare::Eq,
                Value::Null(SqlType::I64),
            )),
            Some(Predicate::IsNull(col("id"))),
        ] {
            assert_error(&c, &update(p), PolicyError::RequiredFilter);
        }
        assert_error(
            &c,
            &Draft::insert(table(), vec![data("id", 1)]).freeze(),
            PolicyError::RequiredFilter,
        );
    }
}
#[test]
fn c02_c03_c04_c05_structural_block_broad_partial_and_unknown_preserved() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(
            d,
            Some(11),
            vec![PolicyDeclaration::SoftDelete(col("deleted"))],
        );
        for op in [update(None), Draft::delete(table()).freeze()] {
            let p = prepare(&c, &op).unwrap();
            let i = p.inspect();
            assert!(i.sensitive_source().metadata().predicate().is_some());
            assert_eq!(i.risk(), Risk::Blocked);
            assert_eq!(
                i.findings().iter().map(Finding::code).collect::<Vec<_>>(),
                vec![FindingCode::MissingWhere]
            );
            assert!(!i.findings()[0].suppressible());
        }
        let c = context(d, Some(11), vec![]);
        for predicate in [
            eq("tenant", 11),
            Predicate::Compare(col("id"), Compare::Gt, Value::I64(0)),
            Predicate::In(col("id"), vec![Value::I64(1), Value::I64(2)]),
            Predicate::Or(vec![eq("id", 1), eq("value", 2)]),
        ] {
            let p = prepare(&c, &update(Some(predicate))).unwrap();
            assert_eq!(p.inspect().scope(), Scope::Broad);
            assert_eq!(p.inspect().risk(), Risk::High);
        }
        let p = prepare(
            &c,
            &update(Some(Predicate::And(vec![
                eq("id", 1),
                Predicate::Not(Box::new(eq("value", 2))),
            ]))),
        )
        .unwrap();
        assert_eq!(p.inspect().scope(), Scope::Single);
        assert_eq!(p.inspect().completeness(), Completeness::Partial);
        assert_eq!(p.inspect().risk(), Risk::High);
    }
}
#[test]
fn c08_c11_c12_context_coverage_identity_and_snapshot_isolation() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let op = update(Some(eq("id", 1)));
        assert_error(&context(d, None, vec![]), &op, PolicyError::MissingTenant);
        assert_error(
            &configured(d, "A", Some(11), None, vec![]),
            &op,
            PolicyError::MissingSchema,
        );
        assert_error(
            &configured(d, "A", Some(11), Some(vec![]), vec![]),
            &op,
            PolicyError::Coverage,
        );
        let a = context(d, Some(11), vec![]);
        let old = prepare(&a, &op).unwrap();
        let b = configured(
            d,
            "B",
            Some(22),
            Some(vec![facts(d)]),
            vec![PolicyDeclaration::Pii(col("secret"))],
        );
        let b_p = prepare(&b, &op).unwrap();
        assert_eq!(
            old.sensitive_source().final_source().args(),
            &[Value::I64(9), Value::I64(1), Value::I64(11)]
        );
        assert_eq!(
            b_p.sensitive_source().final_source().args(),
            &[Value::I64(9), Value::I64(1), Value::I64(22)]
        );
        assert!(
            !old.sensitive_source()
                .context()
                .same_snapshot(b_p.sensitive_source().context())
        );
        assert_eq!(prepare(&a, &op).unwrap(), old);
        let alias = Table::new(vec![id("catalog"), id("items")], Some(id("i"))).unwrap();
        let aliased = Draft::select(
            alias.clone(),
            vec![Projection::new(
                alias.column(id("id"), SqlType::I64, false),
                None,
            )],
        )
        .freeze();
        assert_error(&a, &aliased, PolicyError::Unsupported);
        let unqualified = Table::new(vec![id("items")], None).unwrap();
        assert_error(
            &a,
            &Draft::delete(unqualified).freeze(),
            PolicyError::Unsupported,
        );
        let wrong = table().column(id("id"), SqlType::I32, false);
        assert_error(
            &a,
            &update(Some(Predicate::Compare(wrong, Compare::Eq, Value::I32(1)))),
            PolicyError::Coverage,
        );
        let other = Table::new(vec![id("other"), id("items")], None).unwrap();
        let f = ScopeFacts::application_asserted(
            other.clone(),
            d,
            vec![],
            vec![],
            TableContext::OrdinaryBaseTableWithoutEffects,
        )
        .unwrap();
        let c = configured(d, "A", Some(11), Some(vec![facts(d), f]), vec![]);
        assert_error(&c, &Draft::delete(other).freeze(), PolicyError::Identity);
    }
}
#[test]
fn bounded_before_clone_and_after_guard_injection_and_branch_expansion() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, Some(11), vec![]);
        let mut deep = eq("id", 1);
        for _ in 0..17 {
            deep = Predicate::And(vec![deep]);
        }
        let op = update(Some(deep));
        assert_error(&c, &op, PolicyError::Budget);
        assert_eq!(op.finalize(d).unwrap_err(), PlanError::Budget);
        let mut boundary = eq("id", 1);
        for _ in 0..16 {
            boundary = Predicate::And(vec![boundary]);
        }
        let op = update(Some(boundary));
        assert!(op.finalize(d).is_ok());
        assert_error(&c, &op, PolicyError::Budget);
        assert_error(
            &c,
            &update(Some(Predicate::Or(vec![eq("id", 1); 257]))),
            PolicyError::Budget,
        );
        let expansion = Predicate::And(
            (0..9)
                .map(|_| Predicate::Or(vec![eq("value", 1), eq("id", 2)]))
                .collect(),
        );
        assert_error(&c, &update(Some(expansion)), PolicyError::Budget);
        let nodes = Predicate::And(vec![Predicate::Or(vec![eq("id", 1); 256]); 4]);
        assert_error(&c, &update(Some(nodes)), PolicyError::Budget);
        let args = Predicate::And(vec![Predicate::In(col("id"), vec![Value::I64(1); 256]); 4]);
        assert_error(&c, &update(Some(args)), PolicyError::Budget);
        let full_args = update(Some(Predicate::And(vec![
            Predicate::In(col("id"), vec![Value::I64(1); 256]),
            Predicate::In(col("id"), vec![Value::I64(1); 256]),
            Predicate::In(col("id"), vec![Value::I64(1); 256]),
            Predicate::In(col("id"), vec![Value::I64(1); 255]),
        ])));
        assert!(full_args.finalize(d).is_ok());
        assert_error(&c, &full_args, PolicyError::Budget);
        let cols = Draft::insert(table(), vec![data("value", 9); 257]).freeze();
        assert_error(&c, &cols, PolicyError::Budget);
        let text = Draft::insert(
            table(),
            vec![(col("secret"), Input::Value(Value::Text("x".repeat(4097))))],
        )
        .freeze();
        assert_error(&c, &text, PolicyError::Budget);
        // 256 original pairs fit the raw budget; tenant replenishment may not exceed it.
        let t = table();
        let columns: Vec<_> = (0..256)
            .map(|n| t.column(id(&format!("c{n}")), SqlType::I64, false))
            .collect();
        let mut registry = columns.clone();
        registry.push(col("tenant"));
        // Schema itself caps coverage at 256, refusing before preparation.
        assert_eq!(
            ScopeFacts::application_asserted(
                t,
                d,
                registry,
                vec![],
                TableContext::OrdinaryBaseTableWithoutEffects
            )
            .unwrap_err(),
            PlanError::Budget
        );
    }
}
#[test]
fn c15_fixed_errors_redacted_preparation_and_sensitive_boundary() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, Some(987654321), vec![]);
        let op = update(Some(eq("id", 123456789)));
        let p = prepare(&c, &op).unwrap();
        let output = format!("{p:?}{:?}{:?}", p.inspect(), p.sensitive_source());
        for secret in [
            "987654321",
            "123456789",
            "CANARY_SUBJECT",
            "items",
            "catalog",
            "UPDATE",
        ] {
            assert!(!output.contains(secret));
        }
        for e in [
            PolicyError::InvalidPlan(PlanError::Type),
            PolicyError::MissingSchema,
            PolicyError::Identity,
            PolicyError::Coverage,
            PolicyError::Unsupported,
            PolicyError::MissingTenant,
            PolicyError::TenantMismatch,
            PolicyError::TenantPredicate,
            PolicyError::TenantWrite,
            PolicyError::PiiAccess,
            PolicyError::RequiredFilter,
            PolicyError::ProtectedWrite,
            PolicyError::Budget,
        ] {
            assert!(e.to_string().starts_with("QUENT_POLICY_"));
            assert!(std::error::Error::source(&e).is_none());
            for secret in ["987654321", "CANARY_SUBJECT", "items"] {
                assert!(!format!("{e:?}{e}").contains(secret));
            }
        }
        assert!(
            p.sensitive_source()
                .final_source()
                .args()
                .contains(&Value::I64(987654321))
        );
        assert_eq!(
            p.sensitive_source().context().sensitive_source().subject(),
            Some("CANARY_SUBJECT")
        );
    }
}

#[test]
fn policy_family_matrix_and_unsupported_context_cannot_be_promoted() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(
            d,
            Some(11),
            vec![
                PolicyDeclaration::SoftDelete(col("deleted")),
                PolicyDeclaration::RequiredFilter(col("id")),
            ],
        );
        let read =
            Draft::select(table(), vec![Projection::new(col("id"), None)]).filter(eq("id", 1));
        for op in [
            read.clone().freeze(),
            read.count().unwrap().freeze(),
            update(Some(eq("id", 1))),
            Draft::delete(table()).filter(eq("id", 1)).freeze(),
        ] {
            let p = prepare(&c, &op).unwrap();
            let src = p.sensitive_source();
            let final_ = src.final_source();
            assert!(
                final_
                    .args()
                    .ends_with(&[Value::I64(1), Value::I64(11), Value::Bool(false)])
            );
            assert_eq!(p.inspect().sensitive_source().metadata(), final_.metadata());
        }
        let f = ScopeFacts::application_asserted(
            table(),
            d,
            vec![col("tenant"), col("id"), col("value")],
            vec![],
            TableContext::Unsupported,
        )
        .unwrap();
        assert_error(
            &configured(d, "A", Some(11), Some(vec![f]), vec![]),
            &update(Some(eq("id", 1))),
            PolicyError::Unsupported,
        );
        let f = ScopeFacts::application_asserted(
            table(),
            d,
            vec![col("tenant"), col("id")],
            vec![],
            TableContext::OrdinaryBaseTableWithoutEffects,
        )
        .unwrap();
        assert_error(
            &configured(d, "A", Some(11), Some(vec![f]), vec![]),
            &update(Some(eq("id", 1))),
            PolicyError::Coverage,
        );
        for name in ["Tenant", "tenant`", "TENANT"] {
            let shadow = table().column(id(name), SqlType::I64, false);
            let f = ScopeFacts::application_asserted(
                table(),
                d,
                vec![col("tenant"), shadow.clone(), col("id")],
                vec![],
                TableContext::OrdinaryBaseTableWithoutEffects,
            )
            .unwrap();
            let c = configured(d, "A", Some(11), Some(vec![f]), vec![]);
            assert_error(
                &c,
                &Draft::update(table(), vec![(shadow, Input::Value(Value::I64(22)))])
                    .filter(eq("id", 1))
                    .freeze(),
                PolicyError::Unsupported,
            );
        }
        // An unqualified declaration cannot disappear on the qualified path.
        let unqualified = Table::new(vec![id("items")], None).unwrap();
        let c = context(
            d,
            Some(11),
            vec![PolicyDeclaration::Pii(unqualified.column(
                id("secret"),
                SqlType::Text,
                false,
            ))],
        );
        assert_error(&c, &update(Some(eq("id", 1))), PolicyError::Unsupported);
    }
}

#[test]
fn finite_truth_oracle_outer_guard_matches_caller_and_trusted_tenant() {
    fn truth(p: &Predicate, tenant: i64, key: i64) -> bool {
        match p {
            Predicate::And(ps) => ps.iter().all(|p| truth(p, tenant, key)),
            Predicate::Or(ps) => ps.iter().any(|p| truth(p, tenant, key)),
            Predicate::Not(p) => !truth(p, tenant, key),
            Predicate::Compare(c, Compare::Eq, Value::I64(v)) => {
                if c == &col("tenant") {
                    tenant == *v
                } else {
                    assert_eq!(c, &col("id"));
                    key == *v
                }
            }
            _ => panic!("outside finite test oracle"),
        }
    }
    let atoms = [
        eq("tenant", 11),
        eq("id", 1),
        eq("id", 2),
        Predicate::Not(Box::new(eq("id", 1))),
    ];
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, Some(11), vec![]);
        for a in &atoms {
            for b in &atoms {
                for e in &atoms {
                    let caller =
                        Predicate::Or(vec![Predicate::And(vec![a.clone(), b.clone()]), e.clone()]);
                    let p = prepare(&c, &update(Some(caller.clone()))).unwrap();
                    let source = p.sensitive_source();
                    let final_ = source.final_source();
                    let tree = final_.metadata().predicate().unwrap();
                    for tenant in [10, 11, 12] {
                        for key in [0, 1, 2] {
                            assert_eq!(
                                truth(tree, tenant, key),
                                tenant == 11 && truth(&caller, tenant, key)
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn command_insert_complete_columns_and_original_input_binding() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, Some(11), vec![]);
        let pairs = vec![
            data("id", 1),
            data("value", 0),
            (col("deleted"), Input::Value(Value::Bool(false))),
            (col("secret"), Input::Value(Value::Text(String::new()))),
            data("ro", 0),
            data("generated", 0),
        ];
        let implicit = Draft::insert(table(), pairs.clone()).freeze();
        let p = c.prepare().prepare_command_operation(&implicit).unwrap();
        assert_eq!(p.inspect().risk(), Risk::Low);
        assert_eq!(p.inspect().scope(), Scope::Single);
        assert_eq!(p.inspect().completeness(), Completeness::Complete);
        assert_eq!(
            c.prepare()
                .prepare_operation(&implicit)
                .unwrap()
                .inspect()
                .risk(),
            Risk::High
        );
        let mut explicit = pairs.clone();
        explicit.push(data("tenant", 11));
        let q = c
            .prepare()
            .prepare_command_operation(&Draft::insert(table(), explicit).freeze())
            .unwrap();
        assert_eq!(
            p.sensitive_source().final_source().sql(),
            q.sensitive_source().final_source().sql()
        );
        assert_eq!(
            p.sensitive_source().final_source().args(),
            q.sensitive_source().final_source().args()
        );
        let pc = p
            .sensitive_source()
            .canonical_command_input(&table())
            .unwrap();
        let qc = q
            .sensitive_source()
            .canonical_command_input(&table())
            .unwrap();
        assert_ne!(pc.sensitive_bytes(), qc.sensitive_bytes());
        assert_eq!(
            pc.sensitive_bytes(),
            p.sensitive_source()
                .canonical_command_input(&table())
                .unwrap()
                .sensitive_bytes()
        );
        assert_eq!(format!("{pc:?}"), "CommandCanonical([REDACTED])");
        for omit in 0..pairs.len() {
            let mut v = pairs.clone();
            v.remove(omit);
            assert!(
                c.prepare()
                    .prepare_command_operation(&Draft::insert(table(), v).freeze())
                    .is_err()
            );
        }
        for policy in [
            PolicyDeclaration::Readonly(col("ro")),
            PolicyDeclaration::Generated(col("generated")),
            PolicyDeclaration::Pii(col("secret")),
        ] {
            assert!(
                context(d, Some(11), vec![policy])
                    .prepare()
                    .prepare_command_operation(&implicit)
                    .is_err()
            );
        }
        let key = p
            .sensitive_source()
            .canonical_command_key(b"namespace", &[1; 16])
            .unwrap();
        assert_ne!(
            key.sensitive_bytes(),
            p.sensitive_source()
                .canonical_command_key(b"namespace2", &[1; 16])
                .unwrap()
                .sensitive_bytes()
        );
        assert_ne!(
            key.sensitive_bytes(),
            p.sensitive_source()
                .canonical_command_key(b"namespace", &[2; 16])
                .unwrap()
                .sensitive_bytes()
        );
    }
}
