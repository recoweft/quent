//! QT-C15 output boundary; C01/C08/C12/C13/C14/C16 remain finite evidence only.
use quent_core::*;
use serde_json::{Value as Json, json};
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn table() -> Table {
    Table::new(vec![id("canary_schema"), id("canary_table")], None).unwrap()
}
fn col(s: &str, ty: SqlType, nullable: bool) -> Column {
    table().column(id(s), ty, nullable)
}
fn key() -> Column {
    col("canary_key", SqlType::I64, false)
}
fn text() -> Column {
    col("canary_text", SqlType::Text, true)
}
fn version() -> Column {
    col("canary_version", SqlType::I32, false)
}
fn eq(n: i64) -> Predicate {
    Predicate::Compare(key(), Compare::Eq, Value::I64(n))
}
fn facts(d: Dialect, context: TableContext) -> ScopeFacts {
    ScopeFacts::application_asserted(
        table(),
        d,
        vec![key(), text(), version(), tenant()],
        vec![Key::Primary(vec![key()])],
        context,
    )
    .unwrap()
}
fn tenant() -> Column {
    col("canary_tenant", SqlType::I64, false)
}
fn context(d: Dialect) -> DatabaseContext {
    context_tenant(d, None)
}
fn context_tenant(d: Dialect, t: Option<i64>) -> DatabaseContext {
    let b = DatabaseBinding::application_supplied(
        DatabaseIdentity::new("CANARY_DB").unwrap(),
        TargetAddress::new("/CANARY/PATH").unwrap(),
        id("canary_schema"),
        d,
    );
    let g = Generation::new(1).unwrap();
    DatabaseContext::new(
        Settings::new(
            b.clone(),
            SettingsIdentity::new("CANARY_SETTINGS").unwrap(),
            g,
            PolicySet::new(
                b.clone(),
                PolicyIdentity::new("CANARY_POLICY").unwrap(),
                g,
                if t.is_some() {
                    vec![PolicyDeclaration::Tenant(tenant())]
                } else {
                    vec![]
                },
            )
            .unwrap(),
        )
        .unwrap(),
        ApplicationContext::application_supplied(
            b.clone(),
            Some(
                SchemaSnapshot::application_supplied(
                    b.clone(),
                    SchemaIdentity::new("CANARY_SCHEMA").unwrap(),
                    g,
                    vec![facts(d, TableContext::OrdinaryBaseTableWithoutEffects)],
                )
                .unwrap(),
            ),
            Some(SubjectIdentity::new("CANARY_SUBJECT").unwrap()),
            t.map(|n| {
                TenantSnapshot::application_supplied(b.clone(), tenant(), Value::I64(n)).unwrap()
            }),
        )
        .unwrap(),
    )
    .unwrap()
}
fn safe(v: &PlanView) -> Json {
    #[cfg(feature = "output-tracing")]
    output_capture::check(v, &v.to_json());
    let s = v.to_json();
    assert_eq!(s, v.to_json());
    assert_eq!(s, v.to_string());
    for out in [&s, &format!("{v:?}")] {
        for canary in [
            "canary",
            "CANARY",
            "SELECT ",
            "UPDATE ",
            "DELETE ",
            "NULL",
            "987654321",
            "subject",
            "tenant",
            "reason",
            "source",
            "/tmp/",
            "\\u",
            "snippet",
        ] {
            assert!(!out.contains(canary), "{canary}: {out}");
        }
    }
    let j: Json = serde_json::from_str(&s).unwrap();
    let keys = j
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        vec![
            "assumptions",
            "caller_missing_where",
            "completeness",
            "evidence",
            "family",
            "findings",
            "has_limit",
            "kind",
            "order_count",
            "precision",
            "projection",
            "projection_count",
            "returning_count",
            "risk",
            "scope",
            "scope_upper_bound",
            "version",
            "versioned",
            "write_count"
        ]
    );
    assert!(j["findings"].as_array().unwrap().len() <= 7);
    assert_eq!(j["kind"], "quent_plan_view");
    assert_eq!(j["version"], 1);
    j
}
#[test]
fn independent_schema_and_raw_inspected_prepared_correspondence_both_dialects() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let op = Draft::select(
            table(),
            vec![Projection::new(text(), Some(id("canary_alias")))],
        )
        .filter(eq(987654321))
        .limit(987654321)
        .freeze();
        let p = op.finalize(d).unwrap();
        let expected = json!({"kind":"quent_plan_view","version":1,"precision":"unevaluated","risk":"unevaluated","scope":"unknown","scope_upper_bound":null,"completeness":"unknown","assumptions":"none","evidence":null,"findings":[],"family":"select","projection":"columns","projection_count":1,"write_count":0,"order_count":0,"versioned":false,"returning_count":0,"caller_missing_where":false,"has_limit":true});
        assert_eq!(safe(&p.public_view()), expected);
        let fs = facts(d, TableContext::OrdinaryBaseTableWithoutEffects);
        let i = p.inspect(Some(&fs));
        let mut inspected = expected.clone();
        for (k, v) in [
            ("precision", json!("conditional_conservative")),
            ("risk", json!("low")),
            ("scope", json!("single")),
            ("scope_upper_bound", json!(1)),
            ("completeness", json!("complete")),
            ("assumptions", json!("application_asserted")),
            ("evidence", json!("complete_key_tuples")),
            ("findings", json!(["read_projection"])),
        ] {
            inspected[k] = v;
        }
        assert_eq!(safe(&i.public_view()), inspected);
        let c = context(d);
        let before = format!("{:?}", c);
        let prep = c.prepare().prepare_operation(&op).unwrap();
        assert_eq!(safe(&prep.public_view()), inspected);
        assert_eq!(prep.public_view(), prep.inspect().public_view());
        assert_eq!(format!("{:?}", c), before);
        assert_eq!(safe(&p.public_view()), expected);
        let sql = p.sensitive_source().sql().to_owned();
        let args = p.sensitive_source().args().to_vec();
        let mut copy = i.public_view();
        copy.risk = ViewRisk::Blocked;
        copy.scope = Scope::Unknown;
        assert_eq!(i.risk(), Risk::Low);
        assert_eq!(i.scope(), Scope::Single);
        assert_eq!(i.public_view().to_json(), prep.public_view().to_json());
        assert_eq!(p.sensitive_source().sql(), sql);
        assert_eq!(p.sensitive_source().args(), args);
    }
}
#[test]
fn finite_unknown_partial_budget_block_and_version_returning() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let fs = facts(d, TableContext::OrdinaryBaseTableWithoutEffects);
        let update = || {
            Draft::update(
                table(),
                vec![(
                    text(),
                    Input::Value(Value::Text("CANARY_VALUE_\"\\\n/tmp/source".into())),
                )],
            )
        };
        let plan = update().filter(eq(987654321)).freeze().finalize(d).unwrap();
        let missing = safe(&plan.inspect(None).public_view());
        assert_eq!(missing["risk"], "high");
        assert_eq!(missing["evidence"], "missing_facts");
        assert_eq!(missing["completeness"], "unknown");
        let unsupported = facts(d, TableContext::Unsupported);
        assert_eq!(
            safe(&plan.inspect(Some(&unsupported)).public_view())["evidence"],
            "unsupported_context"
        );
        let partial = update()
            .filter(Predicate::And(vec![eq(1), Predicate::IsNull(text())]))
            .freeze()
            .finalize(d)
            .unwrap();
        let j = safe(&partial.inspect(Some(&fs)).public_view());
        assert_eq!(j["scope"], "single");
        assert_eq!(j["completeness"], "partial");
        assert_eq!(j["risk"], "high");
        let budget = update()
            .filter(Predicate::Or(
                (0..256)
                    .map(|_| Predicate::Or(vec![eq(1), eq(2)]))
                    .collect(),
            ))
            .freeze()
            .finalize(d)
            .unwrap();
        let j = safe(&budget.inspect(Some(&fs)).public_view());
        assert_eq!(j["evidence"], "analysis_budget");
        assert_eq!(j["scope"], "unknown");
        assert_eq!(j["risk"], "high");
        let p = update()
            .optimistic_version(version(), Value::I32(987654321))
            .unwrap()
            .freeze()
            .finalize(d)
            .unwrap();
        let j = safe(&p.inspect(Some(&fs)).public_view());
        assert_eq!(j["risk"], "blocked");
        assert_eq!(j["findings"], json!(["missing_where"]));
        assert_eq!(j["caller_missing_where"], true);
        assert_eq!(j["versioned"], true);
        assert_eq!(j["write_count"], 2);
        let op = update()
            .filter(eq(1))
            .returning(vec![Projection::new(key(), Some(id("canary_alias")))])
            .freeze();
        match op.finalize(d) {
            Ok(p) => {
                let j = safe(&p.inspect(Some(&fs)).public_view());
                assert_eq!(j["returning_count"], 1);
                assert_eq!(j["projection_count"], 1);
                assert_eq!(j["risk"], "low");
            }
            Err(e) => assert_eq!(e.public_view().code, DiagnosticCode::PlanUnsupported),
        }
        for value in [
            Value::Null(SqlType::Text),
            Value::Text("".into()),
            Value::Text("CANARY_TENANT_SUBJECT_reason_source_path".into()),
        ] {
            let p = Draft::insert(table(), vec![(text(), Input::Value(value))])
                .freeze()
                .finalize(d)
                .unwrap();
            let j = safe(&p.inspect(Some(&fs)).public_view());
            assert_eq!(j["risk"], "high");
            assert_eq!(j["scope"], "not_applicable");
        }
    }
}
#[test]
fn diagnostic_fixed_mapping_and_no_error_chain() {
    use std::error::Error;
    for (v, code) in [
        (PlanError::Identifier.public_view(), "plan_identifier"),
        (PlanError::Budget.public_view(), "plan_budget"),
        (ContextError::Binding.public_view(), "context_binding"),
        (
            PolicyError::TenantMismatch.public_view(),
            "policy_tenant_mismatch",
        ),
        (
            PolicyError::InvalidPlan(PlanError::Type).public_view(),
            "plan_type",
        ),
    ] {
        assert_eq!(
            serde_json::from_str::<Json>(&v.to_json()).unwrap(),
            json!({"kind":"quent_diagnostic_view","version":1,"code":code,"state":null})
        );
        assert_eq!(v.to_string(), v.to_json());
        assert!(v.source().is_none());
        assert!(!format!("{v:?}").contains("CANARY"));
    }
}

#[test]
fn trusted_tenant_guard_cannot_erase_original_missing_where_or_leak_values() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context_tenant(d, Some(987654321));
        let op = Draft::delete(table()).freeze();
        let p = c.prepare().prepare_operation(&op).unwrap();
        let j = safe(&p.public_view());
        assert_eq!(j["caller_missing_where"], true);
        assert_eq!(j["risk"], "blocked");
        assert_eq!(j["findings"], json!(["missing_where"]));
        assert!(
            p.sensitive_source()
                .final_source()
                .args()
                .contains(&Value::I64(987654321))
        );
        let op = Draft::delete(table())
            .filter(Predicate::Compare(tenant(), Compare::Eq, Value::I64(9)))
            .freeze();
        let e = c.prepare().prepare_operation(&op).unwrap_err();
        assert_eq!(e, PolicyError::TenantMismatch);
        assert_eq!(e.public_view().code, DiagnosticCode::PolicyTenantMismatch);
    }
}
#[test]
fn scope_and_risk_registers_keep_independent_conservative_expectations() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let fs = facts(d, TableContext::OrdinaryBaseTableWithoutEffects);
        for (predicate, scope, upper, risk, complete) in [
            (
                Predicate::In(key(), vec![]),
                "zero",
                json!(0),
                "low",
                "complete",
            ),
            (
                Predicate::Compare(key(), Compare::Eq, Value::Null(SqlType::I64)),
                "zero",
                json!(0),
                "low",
                "complete",
            ),
            (
                Predicate::Or(vec![eq(1), eq(2)]),
                "finite_bound",
                json!(2),
                "high",
                "complete",
            ),
            (
                Predicate::Compare(key(), Compare::Gt, Value::I64(0)),
                "broad",
                Json::Null,
                "destructive",
                "complete",
            ),
            (
                Predicate::IsNull(text()),
                "unknown",
                Json::Null,
                "high",
                "partial",
            ),
        ] {
            let p = Draft::delete(table())
                .filter(predicate)
                .freeze()
                .finalize(d)
                .unwrap();
            let j = safe(&p.inspect(Some(&fs)).public_view());
            assert_eq!(j["scope"], scope);
            assert_eq!(j["scope_upper_bound"], upper);
            assert_eq!(j["risk"], risk);
            assert_eq!(j["completeness"], complete);
        }
        let p = Draft::select(table(), vec![Projection::new(key(), None)])
            .filter(eq(1))
            .count()
            .unwrap()
            .freeze()
            .finalize(d)
            .unwrap();
        let j = safe(&p.inspect(Some(&fs)).public_view());
        assert_eq!(j["family"], "count");
        assert_eq!(j["projection"], "count");
        assert_eq!(j["risk"], "medium");
    }
}

#[test]
fn complete_core_diagnostic_register_matches_wire_codes() {
    let mappings = [
        (PlanError::Identifier.public_view(), "plan_identifier"),
        (PlanError::Shape.public_view(), "plan_shape"),
        (PlanError::Type.public_view(), "plan_type"),
        (PlanError::Missing.public_view(), "plan_missing"),
        (PlanError::Unsupported.public_view(), "plan_unsupported"),
        (PlanError::Budget.public_view(), "plan_budget"),
        (ContextError::Identity.public_view(), "context_identity"),
        (ContextError::Generation.public_view(), "context_generation"),
        (ContextError::Binding.public_view(), "context_binding"),
        (ContextError::Policy.public_view(), "context_policy"),
        (ContextError::Type.public_view(), "context_type"),
        (
            ContextError::Unsupported.public_view(),
            "context_unsupported",
        ),
        (ContextError::Budget.public_view(), "context_budget"),
        (
            PolicyError::MissingSchema.public_view(),
            "policy_missing_schema",
        ),
        (PolicyError::Identity.public_view(), "policy_identity"),
        (PolicyError::Coverage.public_view(), "policy_coverage"),
        (PolicyError::Unsupported.public_view(), "policy_unsupported"),
        (
            PolicyError::MissingTenant.public_view(),
            "policy_missing_tenant",
        ),
        (
            PolicyError::TenantMismatch.public_view(),
            "policy_tenant_mismatch",
        ),
        (
            PolicyError::TenantPredicate.public_view(),
            "policy_tenant_predicate",
        ),
        (
            PolicyError::TenantWrite.public_view(),
            "policy_tenant_write",
        ),
        (PolicyError::PiiAccess.public_view(), "policy_pii_access"),
        (
            PolicyError::RequiredFilter.public_view(),
            "policy_required_filter",
        ),
        (
            PolicyError::ProtectedWrite.public_view(),
            "policy_protected_write",
        ),
        (PolicyError::Budget.public_view(), "policy_budget"),
    ];
    for (v, wire) in mappings {
        assert_eq!(
            serde_json::from_str::<Json>(&v.to_json()).unwrap(),
            json!({"kind":"quent_diagnostic_view","version":1,"code":wire,"state":null})
        );
        assert_eq!(v.to_json(), v.to_string());
        assert!(std::error::Error::source(&v).is_none());
    }
}

#[cfg(feature = "output-tracing")]
mod output_capture;
#[cfg(feature = "output-tracing")]
#[test]
fn actual_source_views_enter_closed_production_tracing() {
    let capture = output_capture::Capture::default();
    tracing::subscriber::with_default(capture.clone(), || {
        for d in [Dialect::Postgres, Dialect::Mysql] {
            let op = Draft::update(
                table(),
                vec![(
                    text(),
                    Input::Value(Value::Text(
                        "CANARY\n\r\u{1b}SQL_reason_/secret/path".into(),
                    )),
                )],
            )
            .filter(eq(987654321))
            .freeze();
            let plan = op.finalize(d).unwrap();
            let before = plan.sensitive_source().sql().to_owned();
            quent::output_tracing::emit(&plan.public_view());
            let facts = facts(d, TableContext::OrdinaryBaseTableWithoutEffects);
            quent::output_tracing::emit(&plan.inspect(Some(&facts)).public_view());
            let prepared = context(d).prepare().prepare_operation(&op).unwrap();
            quent::output_tracing::emit(&prepared.public_view());
            let error = context_tenant(d, Some(1))
                .prepare()
                .prepare_operation(
                    &Draft::update(
                        table(),
                        vec![(tenant(), Input::Value(Value::I64(987654321)))],
                    )
                    .filter(eq(1))
                    .freeze(),
                )
                .unwrap_err();
            quent::output_tracing::emit(&error.public_view());
            assert!(std::error::Error::source(&error).is_none());
            assert_eq!(before, plan.sensitive_source().sql());
        }
    });
    capture.assert_safe(8);
}

#[cfg(all(feature = "local-details", target_os = "linux"))]
#[test]
fn exact_retained_inspection_and_policy_source_files_preserve_block_and_projection() {
    use quent::local_details::*;
    use std::os::unix::fs::DirBuilderExt;
    let dir = std::env::temp_dir().join(format!("quent-retained-{}", std::process::id()));
    std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            if self.0.exists() {
                std::fs::remove_dir_all(&self.0).unwrap();
            }
        }
    }
    let _cleanup = Cleanup(dir.clone());
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let op = Draft::update(
            table(),
            vec![(
                text(),
                Input::Value(Value::Text("CANARY\n\u{1b}reason".into())),
            )],
        )
        .optimistic_version(version(), Value::I32(-7))
        .unwrap();
        let op = if d == Dialect::Postgres {
            op.returning(vec![Projection::new(text(), Some(id("canary_alias")))])
        } else {
            op
        }
        .freeze();
        let c = context_tenant(d, Some(987654321));
        let prepared = c.prepare().prepare_operation(&op).unwrap();
        let before = prepared.public_view().to_json();
        assert_eq!(prepared.inspect().risk(), Risk::Blocked);
        let inspection = prepared.inspect();
        for (i, source) in [
            inspection.sensitive_source(),
            prepared.sensitive_source().final_source(),
        ]
        .into_iter()
        .enumerate()
        {
            let path = dir.join(format!("{d:?}-{i}"));
            let result = create_new(
                LocalOptIn::acknowledge_sensitive_local_file(),
                Surface::LocalApplication,
                &path,
                source,
            );
            if std::env::var_os("CI").is_some() || std::env::var_os("GITHUB_ACTIONS").is_some() {
                assert_eq!(result.unwrap_err(), DetailError::EnvironmentDenied);
                assert!(!path.exists());
                continue;
            }
            result.unwrap();
            let j: Json = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
            assert!(j["sql"] == prepared.sensitive_source().final_source().sql());
            // Finalizer order: text assignment, successor, automatic tenant, expected version.
            assert!(
                j["args"]
                    == json!([{"type":"text","value":"CANARY\n\u{1b}reason"},{"type":"i32","value":"-6"},{"type":"i64","value":"987654321"},{"type":"i32","value":"-7"}])
            );
            let m = &j["metadata"];
            let table_json = json!({"path":["canary_schema","canary_table"],"alias":null});
            let text_col =
                json!({"table":table_json,"name":"canary_text","type":"text","nullable":true});
            let version_col =
                json!({"table":table_json,"name":"canary_version","type":"i32","nullable":false});
            let tenant_col =
                json!({"table":table_json,"name":"canary_tenant","type":"i64","nullable":false});
            assert!(
                *m == json!({"table":table_json,"family":"update","projection":if d==Dialect::Postgres {json!([{"column":text_col,"alias":"canary_alias"}])}else{Json::Null},"predicate":{"op":"and","children":[{"op":"eq","column":tenant_col,"value":{"type":"i64","value":"987654321"}}]},"write_data":[{"column":text_col,"input":{"type":"text","value":"CANARY\n\u{1b}reason"}},{"column":version_col,"input":{"type":"i32","value":"-6"}}],"order":[],"limit":null,"version":{"column":version_col,"expected":{"type":"i32","value":"-7"}},"caller_missing_where":true})
            );
            for forbidden in [
                "context",
                "subject",
                "handle",
                "permit",
                "credential",
                "executor",
                "policy",
            ] {
                assert!(j.get(forbidden).is_none());
            }
        }
        assert_eq!(prepared.public_view().to_json(), before);
        assert_eq!(prepared.inspect().risk(), Risk::Blocked);
    }
    std::fs::remove_dir_all(&dir).unwrap();
    assert!(!dir.exists());
}
