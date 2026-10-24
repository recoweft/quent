use quent_core::*;
fn gen_(n: u64) -> Generation {
    Generation::new(n).unwrap()
}
fn binding(db: &str, dialect: Dialect) -> DatabaseBinding {
    DatabaseBinding::application_supplied(
        DatabaseIdentity::new(db).unwrap(),
        TargetAddress::new(format!("route-{db}")).unwrap(),
        Ident::new("catalog").unwrap(),
        dialect,
    )
}
fn column(name: &str, ty: SqlType, nullable: bool) -> Column {
    Table::new(vec![Ident::new("items").unwrap()], None)
        .unwrap()
        .column(Ident::new(name).unwrap(), ty, nullable)
}
fn policies(b: &DatabaseBinding, n: u64, pii: bool) -> PolicySet {
    let mut ds = vec![PolicyDeclaration::Tenant(column(
        "tenant",
        SqlType::I64,
        false,
    ))];
    if pii {
        ds.push(PolicyDeclaration::Pii(column(
            "secret",
            SqlType::Text,
            true,
        )));
    }
    PolicySet::new(
        b.clone(),
        PolicyIdentity::new("policy").unwrap(),
        gen_(n),
        ds,
    )
    .unwrap()
}
fn settings(b: &DatabaseBinding, n: u64, pii: bool) -> Settings {
    Settings::new(
        b.clone(),
        SettingsIdentity::new("settings").unwrap(),
        gen_(n),
        policies(b, n, pii),
    )
    .unwrap()
}
fn app(b: &DatabaseBinding, subject: &str, tenant: i64, generation: u64) -> ApplicationContext {
    ApplicationContext::application_supplied(
        b.clone(),
        Some(
            SchemaSnapshot::application_supplied(
                b.clone(),
                SchemaIdentity::new("schema").unwrap(),
                gen_(generation),
                vec![],
            )
            .unwrap(),
        ),
        Some(SubjectIdentity::new(subject).unwrap()),
        Some(
            TenantSnapshot::application_supplied(
                b.clone(),
                column("tenant", SqlType::I64, false),
                Value::I64(tenant),
            )
            .unwrap(),
        ),
    )
    .unwrap()
}
#[test]
fn c11_isolation_derivation_input_and_old_generation_both_dialects() {
    for dialect in [Dialect::Postgres, Dialect::Mysql] {
        let a = binding("A", dialect);
        let b = binding("B", dialect);
        let original_settings = settings(&a, 1, false);
        assert_eq!(original_settings.mode(), SettingsMode::Strict);
        let original =
            DatabaseContext::new(original_settings.clone(), app(&a, "subject-A", 11, 1)).unwrap();
        let sibling = original.prepare();
        let nested = sibling.prepare();
        let other =
            DatabaseContext::new(settings(&b, 1, true), app(&b, "subject-B", 22, 1)).unwrap();
        let changed = original
            .with_application(app(&a, "subject-C", 33, 2))
            .unwrap();
        let newer = original_settings
            .next(gen_(2), policies(&a, 2, true))
            .unwrap();
        let new_db = original.with_settings(newer).unwrap();
        // Independent literal expectations, not copies of observed snapshots.
        for c in [sibling.snapshot(), nested.snapshot(), &original] {
            let s = c.sensitive_source();
            assert_eq!(
                s.target(),
                ("A", "route-A", &Ident::new("catalog").unwrap(), dialect)
            );
            assert_eq!(s.subject(), Some("subject-A"));
            assert_eq!(
                s.tenant(),
                Some((&column("tenant", SqlType::I64, false), &Value::I64(11)))
            );
            assert_eq!(s.settings_version(), ("settings", 1));
            assert_eq!(s.policy_version(), ("policy", 1));
            assert_eq!(
                s.declarations(),
                &[PolicyDeclaration::Tenant(column(
                    "tenant",
                    SqlType::I64,
                    false
                ))]
            );
        }
        let b_prepared = other.prepare().prepare();
        assert_eq!(
            b_prepared.snapshot().sensitive_source().subject(),
            Some("subject-B")
        );
        assert_eq!(b_prepared.snapshot().sensitive_source().target().0, "B");
        assert_eq!(
            b_prepared.snapshot().sensitive_source().tenant().unwrap().1,
            &Value::I64(22)
        );
        assert_eq!(other.sensitive_source().subject(), Some("subject-B"));
        assert_eq!(
            other.sensitive_source().tenant().unwrap().1,
            &Value::I64(22)
        );
        assert_eq!(other.sensitive_source().declarations().len(), 2);
        assert_eq!(changed.sensitive_source().subject(), Some("subject-C"));
        assert_eq!(changed.sensitive_source().schema().unwrap().1, 2);
        assert_eq!(
            new_db.sensitive_source().settings_version(),
            ("settings", 2)
        );
        assert_eq!(new_db.sensitive_source().policy_version(), ("policy", 2));
        assert!(!original.same_snapshot(&new_db));
        assert!(!original.same_snapshot(&changed));
        assert!(!original.same_snapshot(&other));
        // Reused identity/generation labels alone cannot hide changed content.
        let relabeled =
            DatabaseContext::new(settings(&a, 1, true), app(&a, "subject-A", 11, 1)).unwrap();
        assert!(!original.same_snapshot(&relabeled));
        let subject_only = original
            .with_application(app(&a, "different-subject", 11, 1))
            .unwrap();
        let tenant_only = original
            .with_application(app(&a, "subject-A", 12, 1))
            .unwrap();
        assert!(!original.same_snapshot(&subject_only));
        assert!(!original.same_snapshot(&tenant_only));

        assert!(original.with_settings(original_settings.clone()).is_err());
        assert!(
            original_settings
                .next(gen_(2), policies(&a, 1, false))
                .is_err()
        );
        // Owned constructor inputs cannot later mutate declarations/identity/values.
        let mut input = vec![PolicyDeclaration::Pii(column(
            "secret",
            SqlType::Text,
            true,
        ))];
        let p = PolicySet::new(
            a.clone(),
            PolicyIdentity::new("owned").unwrap(),
            gen_(1),
            input.clone(),
        )
        .unwrap();
        input.clear();
        let mut subject = String::from("owned-subject");
        let identity = SubjectIdentity::new(subject.clone()).unwrap();
        subject.clear();
        let c = ApplicationContext::application_supplied(a.clone(), None, Some(identity), None)
            .unwrap();
        let db = DatabaseContext::new(
            Settings::new(
                a.clone(),
                SettingsIdentity::new("owned").unwrap(),
                gen_(1),
                p,
            )
            .unwrap(),
            c,
        )
        .unwrap();
        assert_eq!(db.sensitive_source().subject(), Some("owned-subject"));
        assert_eq!(
            db.sensitive_source().declarations(),
            &[PolicyDeclaration::Pii(column(
                "secret",
                SqlType::Text,
                true
            ))]
        );
    }
}
#[test]
fn c12_exact_target_and_context_missing_never_become_live_or_permission() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let a = binding("A", d);
        let b = binding("B", d);
        let s = settings(&a, 1, false);
        let db = DatabaseContext::new(s.clone(), app(&a, "subject", 1, 1)).unwrap();
        assert_eq!(
            DatabaseContext::new(s.clone(), app(&b, "subject", 1, 1)).unwrap_err(),
            ContextError::Binding
        );
        assert!(db.with_application(app(&b, "subject", 1, 1)).is_err());
        assert!(
            Settings::new(
                a.clone(),
                SettingsIdentity::new("settings").unwrap(),
                gen_(1),
                policies(&b, 1, false)
            )
            .is_err()
        );
        for other in [
            binding(
                "A",
                if d == Dialect::Postgres {
                    Dialect::Mysql
                } else {
                    Dialect::Postgres
                },
            ),
            DatabaseBinding::application_supplied(
                DatabaseIdentity::new("A").unwrap(),
                TargetAddress::new("other-route").unwrap(),
                Ident::new("catalog").unwrap(),
                d,
            ),
            DatabaseBinding::application_supplied(
                DatabaseIdentity::new("A").unwrap(),
                TargetAddress::new("route-A").unwrap(),
                Ident::new("other_catalog").unwrap(),
                d,
            ),
        ] {
            assert!(DatabaseContext::new(s.clone(), app(&other, "subject", 1, 1)).is_err());
        }
        let absent = ApplicationContext::application_supplied(a.clone(), None, None, None).unwrap();
        let missing = DatabaseContext::new(s, absent).unwrap();
        assert_eq!(
            missing.presence(),
            ContextPresence {
                schema: false,
                subject: false,
                tenant: false
            }
        );
        assert!(!missing.same_snapshot(&db));
        assert!(
            ApplicationContext::application_supplied(
                a.clone(),
                None,
                None,
                Some(
                    TenantSnapshot::application_supplied(
                        b,
                        column("tenant", SqlType::I64, false),
                        Value::I64(1)
                    )
                    .unwrap()
                )
            )
            .is_err()
        );
        let schema = SchemaSnapshot::application_supplied(
            binding("B", d),
            SchemaIdentity::new("schema").unwrap(),
            gen_(1),
            vec![],
        )
        .unwrap();
        assert!(ApplicationContext::application_supplied(a, Some(schema), None, None).is_err());
    }
}
#[test]
fn unsupported_types_policies_and_budgets_have_fixed_refusals() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let b = binding("A", d);
        for (ty, v) in [
            (SqlType::Text, Value::Text("1".into())),
            (SqlType::U64, Value::U64(1)),
            (SqlType::Bool, Value::Bool(true)),
            (SqlType::I64, Value::Null(SqlType::I64)),
        ] {
            assert!(
                TenantSnapshot::application_supplied(b.clone(), column("tenant", ty, false), v)
                    .is_err()
            );
        }
        assert!(
            TenantSnapshot::application_supplied(
                b.clone(),
                column("tenant", SqlType::I64, true),
                Value::I64(1)
            )
            .is_err()
        );
        assert_eq!(
            TenantSnapshot::application_supplied(
                b.clone(),
                column("tenant", SqlType::I64, false),
                Value::I32(1)
            )
            .unwrap_err(),
            ContextError::Type
        );
        for (ty, v) in [
            (SqlType::I16, Value::I16(i16::MIN)),
            (SqlType::I32, Value::I32(i32::MAX)),
            (SqlType::I64, Value::I64(0)),
        ] {
            assert!(
                TenantSnapshot::application_supplied(b.clone(), column("tenant", ty, false), v)
                    .is_ok()
            );
        }
        assert_eq!(
            TenantSnapshot::application_supplied(
                b.clone(),
                column("tenant", SqlType::I8, false),
                Value::I8(1)
            )
            .is_ok(),
            d == Dialect::Mysql
        );
        let p = PolicyDeclaration::Tenant(column("tenant", SqlType::I64, false));
        for declarations in [
            vec![p.clone(), p.clone()],
            vec![
                p.clone(),
                PolicyDeclaration::Pii(column("tenant", SqlType::Text, true)),
            ],
            vec![
                p.clone(),
                PolicyDeclaration::Tenant(column("other", SqlType::I64, false)),
            ],
        ] {
            assert_eq!(
                PolicySet::new(
                    b.clone(),
                    PolicyIdentity::new("p").unwrap(),
                    gen_(1),
                    declarations
                )
                .unwrap_err(),
                ContextError::Policy
            );
        }
        assert_eq!(
            PolicySet::new(
                b.clone(),
                PolicyIdentity::new("p").unwrap(),
                gen_(1),
                vec![p; 257]
            )
            .unwrap_err(),
            ContextError::Budget
        );
        assert!(
            PolicySet::new(
                b,
                PolicyIdentity::new("p").unwrap(),
                gen_(1),
                vec![PolicyDeclaration::SoftDelete(column(
                    "deleted",
                    SqlType::I64,
                    false
                ))]
            )
            .is_err()
        );
    }
    for s in ["", "x y", "x\0y", &"x".repeat(129)] {
        assert_eq!(SubjectIdentity::new(s).unwrap_err(), ContextError::Identity);
    }
    assert_eq!(Generation::new(0).unwrap_err(), ContextError::Generation);
}
#[test]
fn schema_exact_contents_not_only_identity_generation_and_assertions_stay_asserted() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let b = binding("A", d);
        let t = Table::new(vec![Ident::new("items").unwrap()], None).unwrap();
        let c = column("tenant", SqlType::I64, false);
        let facts = ScopeFacts::application_asserted(
            t.clone(),
            d,
            vec![c.clone()],
            vec![Key::Primary(vec![c.clone()])],
            TableContext::OrdinaryBaseTableWithoutEffects,
        )
        .unwrap();
        let schema = SchemaSnapshot::application_supplied(
            b.clone(),
            SchemaIdentity::new("s").unwrap(),
            gen_(1),
            vec![facts.clone()],
        )
        .unwrap();
        let context = DatabaseContext::new(
            settings(&b, 1, false),
            ApplicationContext::application_supplied(b.clone(), Some(schema), None, None).unwrap(),
        )
        .unwrap();
        let plan = Draft::select(t, vec![Projection::new(c.clone(), None)])
            .filter(Predicate::Compare(c, Compare::Eq, Value::I64(1)))
            .freeze()
            .finalize(d)
            .unwrap();
        let src = context.sensitive_source();
        let retained = &src.schema().unwrap().2[0];
        assert_eq!(
            plan.inspect(Some(retained)).assumptions(),
            Assumptions::ApplicationAsserted
        );
        let bad_column = column("tenant", SqlType::I32, false);
        let bad_facts = ScopeFacts::application_asserted(
            Table::new(vec![Ident::new("items").unwrap()], None).unwrap(),
            d,
            vec![bad_column],
            vec![],
            TableContext::Unsupported,
        )
        .unwrap();
        let bad_schema = SchemaSnapshot::application_supplied(
            b.clone(),
            SchemaIdentity::new("s").unwrap(),
            gen_(1),
            vec![bad_facts],
        )
        .unwrap();
        assert_eq!(
            context
                .with_application(
                    ApplicationContext::application_supplied(
                        b.clone(),
                        Some(bad_schema),
                        None,
                        None
                    )
                    .unwrap()
                )
                .unwrap_err(),
            ContextError::Type
        );
        let empty = SchemaSnapshot::application_supplied(
            b.clone(),
            SchemaIdentity::new("s").unwrap(),
            gen_(1),
            vec![],
        )
        .unwrap();
        let changed = context
            .with_application(
                ApplicationContext::application_supplied(b.clone(), Some(empty), None, None)
                    .unwrap(),
            )
            .unwrap();
        assert!(!changed.same_snapshot(&context));
        assert!(
            SchemaSnapshot::application_supplied(
                b.clone(),
                SchemaIdentity::new("s").unwrap(),
                gen_(1),
                vec![facts.clone(), facts.clone()]
            )
            .is_err()
        );
        assert!(
            SchemaSnapshot::application_supplied(
                b.clone(),
                SchemaIdentity::new("s").unwrap(),
                gen_(1),
                vec![facts; 17]
            )
            .is_err()
        );
        let wrong = ScopeFacts::application_asserted(
            Table::new(vec![Ident::new("items").unwrap()], None).unwrap(),
            if d == Dialect::Postgres {
                Dialect::Mysql
            } else {
                Dialect::Postgres
            },
            vec![],
            vec![],
            TableContext::Unsupported,
        )
        .unwrap();
        assert!(
            SchemaSnapshot::application_supplied(
                b,
                SchemaIdentity::new("s").unwrap(),
                gen_(1),
                vec![wrong]
            )
            .is_err()
        );
    }
}
#[test]
fn c15_all_carriers_redact_canary_and_errors_have_no_source() {
    let canary = "CANARY_TENANT_SUBJECT_PATH";
    let b = binding(canary, Dialect::Postgres);
    let s = settings(&b, 1, true);
    let a = app(&b, canary, 987654321, 1);
    let db = DatabaseContext::new(s.clone(), a.clone()).unwrap();
    let text = format!(
        "{b:?}{s:?}{a:?}{db:?}{:?}{:?}{:?}{:?}{:?}{:?}{:?}",
        db.prepare(),
        db.sensitive_source(),
        SubjectIdentity::new(canary).unwrap(),
        gen_(987654321),
        policies(&b, 1, true),
        TenantSnapshot::application_supplied(
            b.clone(),
            column("tenant", SqlType::I64, false),
            Value::I64(987654321)
        )
        .unwrap(),
        SchemaSnapshot::application_supplied(
            b.clone(),
            SchemaIdentity::new(canary).unwrap(),
            gen_(1),
            vec![]
        )
        .unwrap()
    );
    let identities = format!(
        "{:?}{:?}{:?}{:?}{:?}{:?}{:?}",
        DatabaseIdentity::new(canary).unwrap(),
        TargetAddress::new(canary).unwrap(),
        SchemaIdentity::new(canary).unwrap(),
        PolicyIdentity::new(canary).unwrap(),
        SettingsIdentity::new(canary).unwrap(),
        SubjectIdentity::new(canary).unwrap(),
        PolicyDeclaration::Pii(column(canary, SqlType::Text, true))
    );
    assert!(!identities.contains(canary));
    for secret in [
        canary,
        "987654321",
        "route-",
        "catalog",
        "secret",
        "schema",
        "subject",
    ] {
        assert!(!text.contains(secret), "redaction failed");
    }
    for e in [
        ContextError::Identity,
        ContextError::Generation,
        ContextError::Binding,
        ContextError::Policy,
        ContextError::Type,
        ContextError::Unsupported,
        ContextError::Budget,
    ] {
        assert!(std::error::Error::source(&e).is_none());
        assert!(e.to_string().starts_with("QUENT_CONTEXT_"));
    }
}
