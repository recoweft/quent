use super::*;
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::task::{Context, Poll, Waker};
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

struct Adapter {
    calls: Arc<AtomicUsize>,
    response: AdapterResponse,
}
impl TrustedAuthorizationAdapter for Adapter {
    fn authorize<'a>(
        &'a self,
        r: AuthorizationRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = AuthorizationDecision> + Send + 'a>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert_eq!(
            r.sensitive_registration(),
            ("CANARY_PROVIDER", 1, b"CANARY_CONFIG".as_slice())
        );
        assert!(
            r.sensitive_preparation()
                .sensitive_source()
                .final_source()
                .sql()
                .contains("UPDATE")
        );
        assert!(!format!("{r:?}").contains("CANARY"));
        Box::pin(async move { r.respond(self.response) })
    }
}
fn register(
    c: DatabaseContext,
    response: AdapterResponse,
) -> (AuthorizationContext, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    let a = AuthorizationContext::register(
        c,
        "CANARY_PROVIDER".into(),
        1,
        b"CANARY_CONFIG".to_vec(),
        Adapter {
            calls: calls.clone(),
            response,
        },
    )
    .unwrap();
    (a, calls)
}
fn broad() -> Operation {
    update(Some(eq("tenant", 1)))
}
fn candidate(c: &AuthorizationContext) -> AuthorizationCandidate {
    c.candidate(
        &broad(),
        AuthorizationScope::MultiRowUpdate,
        Duration::from_secs(1),
    )
    .unwrap()
}
fn ready<F: Future>(f: F) -> F::Output {
    let mut f = Box::pin(f);
    match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("unexpected pending"),
    }
}
#[test]
fn exact_context_provider_subject_tenant_and_derivation() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let db = context(d, Some(1), vec![]);
        let (a, calls) = register(db.clone(), AdapterResponse::AllowCandidate);
        let sibling = a.with_database(db.clone()).unwrap();
        let nested = sibling.with_database(db.clone()).unwrap();
        assert!(ready(acquire_authorization(candidate(&a), &nested)).is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        // Same labels/generation/config are insufficient: registration instance differs.
        let (other, other_calls) = register(db, AdapterResponse::AllowCandidate);
        assert_eq!(
            ready(acquire_authorization(candidate(&a), &other)).unwrap_err(),
            AuthorizationError::Mismatch
        );
        assert_eq!(other_calls.load(Ordering::SeqCst), 0);
        let variants = [
            configured(d, "B", Some(1), Some(vec![facts(d)]), vec![]),
            context(d, Some(2), vec![]),
            context(d, Some(1), vec![PolicyDeclaration::Readonly(col("ro"))]),
            context(
                d,
                Some(1),
                vec![PolicyDeclaration::Generated(col("generated"))],
            ),
        ];
        for db in variants {
            match a.with_database(db) {
                Ok(changed) => assert_eq!(
                    ready(acquire_authorization(candidate(&a), &changed)).unwrap_err(),
                    AuthorizationError::Mismatch
                ),
                Err(e) => assert_eq!(e, AuthorizationError::Mismatch),
            }
        }
        let changed = configured(d, "A", Some(1), Some(vec![]), vec![]);
        let changed = a.with_database(changed).unwrap();
        assert_eq!(
            ready(acquire_authorization(candidate(&a), &changed)).unwrap_err(),
            AuthorizationError::Mismatch
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn finite_responses_and_redaction() {
    use std::error::Error;
    for (response, expected) in [
        (AdapterResponse::Deny, AuthorizationError::Denied),
        (AdapterResponse::Error, AuthorizationError::ProviderError),
        (AdapterResponse::Cancelled, AuthorizationError::Cancelled),
        (AdapterResponse::Timeout, AuthorizationError::Timeout),
    ] {
        let (a, calls) = register(context(Dialect::Postgres, Some(1), vec![]), response);
        let c = candidate(&a);
        assert_eq!(format!("{c:?}"), "AuthorizationCandidate([REDACTED])");
        let e = ready(acquire_authorization(c, &a)).unwrap_err();
        assert_eq!(e, expected);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(e.source().is_none());
        assert!(!format!("{e:?}{e}").contains("CANARY"));
    }
}
#[test]
fn structural_policy_missing_provider_and_no_implicit_calls() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let db = context(d, Some(1), vec![]);
        assert_eq!(
            AuthorizationContext::without_provider(db.clone())
                .candidate(
                    &broad(),
                    AuthorizationScope::MultiRowUpdate,
                    Duration::from_secs(1)
                )
                .unwrap_err(),
            AuthorizationError::MissingProvider
        );
        let (a, calls) = register(db.clone(), AdapterResponse::AllowCandidate);
        for op in [
            update(None),
            update(Some(eq("tenant", 2))),
            Draft::insert(table(), vec![data("value", 1)]).freeze(),
            update(Some(Predicate::Not(Box::new(eq("id", 1))))),
        ] {
            assert!(
                a.candidate(
                    &op,
                    AuthorizationScope::MultiRowUpdate,
                    Duration::from_secs(1)
                )
                .is_err()
            );
        }
        let low = update(Some(Predicate::And(vec![eq("tenant", 1), eq("id", 1)])));
        assert!(
            a.candidate(
                &low,
                AuthorizationScope::MultiRowUpdate,
                Duration::from_secs(1)
            )
            .is_err()
        );
        let p = db.prepare().prepare_operation(&broad()).unwrap();
        let _ = p.inspect().public_view().to_json();
        let _ = broad().finalize(d).unwrap().public_view().to_json();
        #[cfg(feature = "operation-spec")]
        {
            use crate::operation_spec::*;
            let columns = [
                "id",
                "tenant",
                "value",
                "deleted",
                "secret",
                "ro",
                "generated",
            ]
            .map(|name| {
                let c = col(name);
                TypedColumn::application_supplied(
                    name,
                    c.sensitive_name().clone(),
                    ColumnType::Native(c.sql_type()),
                    c.nullable(),
                    if name == "tenant" {
                        ColumnRole::Tenant
                    } else {
                        ColumnRole::Ordinary
                    },
                    true,
                )
                .unwrap()
            })
            .to_vec();
            let mapping = TypedModelContract::application_asserted(
                "items",
                table(),
                columns,
                ModelCoverage::Complete,
                &db,
            )
            .unwrap();
            let source=SpecSource::parse(br#"{"kind":"quent_operation_spec","version":1,"model":"items","fields":["id"],"filter":{"op":"eq","field":"id","value":{"type":"i64","value":1}},"order":[],"limit":10}"#).unwrap();
            let _ = source
                .compile(&mapping, &db)
                .unwrap()
                .public_view()
                .to_json();
        }
        #[cfg(feature = "review")]
        {
            let _ = crate::review::inspection(&p.inspect()).to_json();
            let _ = crate::review::plan_source(include_bytes!(
                "../../../../fixtures/review-v1/select.json"
            ))
            .unwrap()
            .to_json();
        }
        // Candidate construction and drop also do not contact the provider.
        drop(candidate(&a));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(ready(acquire_authorization(candidate(&a), &a)).is_ok());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
#[test]
fn exclusive_deadlines_and_bounds() {
    let (a, calls) = register(
        context(Dialect::Postgres, Some(1), vec![]),
        AdapterResponse::AllowCandidate,
    );
    let now = Instant::now();
    let c = a
        .candidate_at(
            &broad(),
            AuthorizationScope::MultiRowUpdate,
            Duration::from_secs(1),
            now,
        )
        .unwrap();
    assert!(check_expiry(&c, now).is_ok());
    assert_eq!(
        check_expiry(&c, c.expires),
        Err(AuthorizationError::Expired)
    );
    assert_eq!(
        check_expiry(&c, c.expires + Duration::from_nanos(1)),
        Err(AuthorizationError::Expired)
    );
    for ttl in [Duration::ZERO, Duration::from_secs(301), Duration::MAX] {
        assert_eq!(
            a.candidate(&broad(), AuthorizationScope::MultiRowUpdate, ttl)
                .unwrap_err(),
            AuthorizationError::InvalidTtl
        );
    }
    let c = a
        .candidate_at(
            &broad(),
            AuthorizationScope::MultiRowUpdate,
            Duration::from_nanos(1),
            now,
        )
        .unwrap();
    assert_eq!(
        ready(acquire_authorization(c, &a)).unwrap_err(),
        AuthorizationError::Expired
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}
struct Saved {
    saved: Mutex<Option<AuthorizationDecision>>,
    calls: Arc<AtomicUsize>,
}
impl TrustedAuthorizationAdapter for Saved {
    fn authorize<'a>(
        &'a self,
        r: AuthorizationRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = AuthorizationDecision> + Send + 'a>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let mut saved = self.saved.lock().unwrap();
        if let Some(old) = saved.take() {
            return Box::pin(async { old });
        }
        *saved = Some(r.respond(AdapterResponse::AllowCandidate));
        Box::pin(std::future::pending())
    }
}
#[test]
fn unpolled_drop_pending_cancel_and_cross_request_decision() {
    let calls = Arc::new(AtomicUsize::new(0));
    let a = AuthorizationContext::register(
        context(Dialect::Postgres, Some(1), vec![]),
        "provider".into(),
        1,
        vec![],
        Saved {
            saved: Mutex::new(None),
            calls: calls.clone(),
        },
    )
    .unwrap();
    drop(acquire_authorization(candidate(&a), &a));
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let mut f = Box::pin(acquire_authorization(candidate(&a), &a));
    assert!(
        f.as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
            .is_pending()
    );
    drop(f);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        ready(acquire_authorization(candidate(&a), &a)).unwrap_err(),
        AuthorizationError::Mismatch
    );
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
struct Delayed(Arc<AtomicUsize>);
impl TrustedAuthorizationAdapter for Delayed {
    fn authorize<'a>(
        &'a self,
        r: AuthorizationRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = AuthorizationDecision> + Send + 'a>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            std::thread::sleep(Duration::from_millis(110));
            r.respond(AdapterResponse::AllowCandidate)
        })
    }
}
#[test]
fn expires_while_awaiting_provider() {
    let calls = Arc::new(AtomicUsize::new(0));
    let a = AuthorizationContext::register(
        context(Dialect::Postgres, Some(1), vec![]),
        "provider".into(),
        1,
        vec![],
        Delayed(calls.clone()),
    )
    .unwrap();
    let c = a
        .candidate(
            &broad(),
            AuthorizationScope::MultiRowUpdate,
            Duration::from_millis(100),
        )
        .unwrap();
    assert_eq!(
        ready(acquire_authorization(c, &a)).unwrap_err(),
        AuthorizationError::Expired
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn exact_subject_schema_settings_operation_and_scope() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let db = context(d, Some(1), vec![]);
        let (a, calls) = register(db.clone(), AdapterResponse::AllowCandidate);
        let b = binding(d, "A");
        let app = ApplicationContext::application_supplied(
            b.clone(),
            Some(
                SchemaSnapshot::application_supplied(
                    b.clone(),
                    SchemaIdentity::new("schema").unwrap(),
                    Generation::new(1).unwrap(),
                    vec![facts(d)],
                )
                .unwrap(),
            ),
            Some(SubjectIdentity::new("OTHER_SUBJECT").unwrap()),
            Some(
                TenantSnapshot::application_supplied(b.clone(), col("tenant"), Value::I64(1))
                    .unwrap(),
            ),
        )
        .unwrap();
        let other = a.with_database(db.with_application(app).unwrap()).unwrap();
        assert_eq!(
            ready(acquire_authorization(candidate(&a), &other)).unwrap_err(),
            AuthorizationError::Mismatch
        );
        let g = Generation::new(2).unwrap();
        let settings = Settings::new(
            b.clone(),
            SettingsIdentity::new("settings").unwrap(),
            g,
            PolicySet::new(
                b.clone(),
                PolicyIdentity::new("policy").unwrap(),
                g,
                vec![PolicyDeclaration::Tenant(col("tenant"))],
            )
            .unwrap(),
        )
        .unwrap();
        let other = a
            .with_database(db.with_settings(settings).unwrap())
            .unwrap();
        assert_eq!(
            ready(acquire_authorization(candidate(&a), &other)).unwrap_err(),
            AuthorizationError::Mismatch
        );
        let c1 = candidate(&a);
        let c2 = a
            .candidate(
                &update(Some(Predicate::And(vec![eq("tenant", 1), eq("value", 99)]))),
                AuthorizationScope::MultiRowUpdate,
                Duration::from_secs(1),
            )
            .unwrap();
        assert_ne!(c1.prepared, c2.prepared);
        let c3 = a
            .candidate(
                &Draft::update(table(), vec![data("value", 10)])
                    .filter(eq("tenant", 1))
                    .freeze(),
                AuthorizationScope::MultiRowUpdate,
                Duration::from_secs(1),
            )
            .unwrap();
        assert_ne!(c1.prepared, c3.prepared);
        let delete = Draft::delete(table()).filter(eq("tenant", 1)).freeze();
        assert!(
            a.candidate(
                &delete,
                AuthorizationScope::MultiRowUpdate,
                Duration::from_secs(1)
            )
            .is_err()
        );
        let c = a
            .candidate(
                &delete,
                AuthorizationScope::MultiRowDelete,
                Duration::from_secs(1),
            )
            .unwrap();
        assert_eq!(c.scope, AuthorizationScope::MultiRowDelete);
        assert_eq!(c.prepared.inspect().risk(), Risk::Destructive);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
#[cfg(feature = "output-tracing")]
#[test]
fn provider_error_has_no_trace_or_nested_source() {
    let capture = crate::output_capture::Capture::default();
    tracing::subscriber::with_default(capture.clone(), || {
        let (a, _) = register(
            context(Dialect::Postgres, Some(1), vec![]),
            AdapterResponse::Error,
        );
        let e = ready(acquire_authorization(candidate(&a), &a)).unwrap_err();
        assert_eq!(e, AuthorizationError::ProviderError);
    });
    assert!(capture.0.lock().unwrap().is_empty());
    tracing::subscriber::with_default(capture.clone(), || {
        tracing::event!(tracing::Level::INFO, detection_control = true)
    });
    assert_eq!(capture.0.lock().unwrap().len(), 1);
}

#[test]
fn registration_validation_and_identical_generation_different_schema() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let db = context(d, Some(1), vec![]);
        for (identity, generation, config) in [
            ("".into(), 1, vec![]),
            ("name".into(), 0, vec![]),
            ("name".into(), 1, vec![0; 4097]),
            ("with space".into(), 1, vec![]),
        ] {
            assert_eq!(
                AuthorizationContext::register(
                    db.clone(),
                    identity,
                    generation,
                    config,
                    Delayed(Arc::new(AtomicUsize::new(0)))
                )
                .unwrap_err(),
                AuthorizationError::InvalidRegistration
            );
        }
        let (a, calls) = register(db, AdapterResponse::AllowCandidate);
        let f = ScopeFacts::application_asserted(
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
            vec![Key::Primary(vec![col("id")])],
            TableContext::OrdinaryBaseTableWithoutEffects,
        )
        .unwrap();
        let changed = a
            .with_database(configured(d, "A", Some(1), Some(vec![f]), vec![]))
            .unwrap();
        assert_eq!(
            ready(acquire_authorization(candidate(&a), &changed)).unwrap_err(),
            AuthorizationError::Mismatch
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
