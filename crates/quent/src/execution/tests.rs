#[cfg(feature = "output-tracing")]
use crate::output_capture;
async fn poll_once<F: std::future::Future>(
    mut f: std::pin::Pin<&mut F>,
) -> std::task::Poll<F::Output> {
    std::future::poll_fn(|cx| std::task::Poll::Ready(f.as_mut().poll(cx))).await
}
use super::*;
#[macro_use]
mod authorized;
mod authorized_network;
mod command;
mod command_network;
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn table() -> Table {
    Table::new(vec![id("quent_dispatch_test"), id("items")], None).unwrap()
}
fn col(name: &str, ty: SqlType, nullable: bool) -> Column {
    table().column(id(name), ty, nullable)
}
fn key() -> Column {
    col("id", SqlType::I64, false)
}
fn tenant() -> Column {
    col("tenant", SqlType::I64, false)
}
fn eq(c: Column, v: Value) -> Predicate {
    Predicate::Compare(c, Compare::Eq, v)
}
fn filter(n: i64) -> Predicate {
    eq(key(), Value::I64(n))
}
fn fields(d: Dialect) -> Vec<Column> {
    let mut cs = vec![
        key(),
        tenant(),
        col("small", SqlType::I16, true),
        col("normal", SqlType::I32, true),
        col("big", SqlType::I64, true),
        col("flag", SqlType::Bool, true),
        col("txt", SqlType::Text, true),
        col("optional", SqlType::I64, true),
    ];
    if d == Dialect::Mysql {
        for (n, ty) in [
            ("tiny", SqlType::I8),
            ("u_tiny", SqlType::U8),
            ("u_small", SqlType::U16),
            ("u_normal", SqlType::U32),
            ("u_big", SqlType::U64),
        ] {
            cs.push(col(n, ty, true));
        }
    }
    cs
}
fn context(d: Dialect, t: i64, subject: &str, extras: Vec<PolicyDeclaration>) -> DatabaseContext {
    context_variant(d, t, subject, extras, 0)
}
fn context_variant(
    d: Dialect,
    t: i64,
    subject: &str,
    extras: Vec<PolicyDeclaration>,
    variant: u8,
) -> DatabaseContext {
    let mut cs = fields(d);
    if variant == 3 {
        cs.reverse();
    }
    context_schema(d, t, subject, extras, variant, cs)
}
fn context_schema(
    d: Dialect,
    t: i64,
    subject: &str,
    extras: Vec<PolicyDeclaration>,
    variant: u8,
    cs: Vec<Column>,
) -> DatabaseContext {
    let binding = DatabaseBinding::application_supplied(
        DatabaseIdentity::new(if variant == 1 {
            "other_db"
        } else {
            "dedicated_test"
        })
        .unwrap(),
        TargetAddress::new(if variant == 2 {
            "other_route"
        } else {
            "dedicated_loopback"
        })
        .unwrap(),
        id("quent_dispatch_test"),
        d,
    );
    let g = Generation::new(1).unwrap();
    let facts = ScopeFacts::application_asserted(
        table(),
        d,
        cs,
        vec![Key::Primary(vec![tenant(), key()])],
        TableContext::OrdinaryBaseTableWithoutEffects,
    )
    .unwrap();
    let mut policies = vec![PolicyDeclaration::Tenant(tenant())];
    policies.extend(extras);
    DatabaseContext::new(
        Settings::new(
            binding.clone(),
            SettingsIdentity::new(if variant == 4 {
                "other_settings"
            } else {
                "settings"
            })
            .unwrap(),
            if variant == 5 {
                Generation::new(2).unwrap()
            } else {
                g
            },
            PolicySet::new(
                binding.clone(),
                PolicyIdentity::new(if variant == 6 {
                    "other_policy"
                } else {
                    "policy"
                })
                .unwrap(),
                if variant == 7 {
                    Generation::new(2).unwrap()
                } else {
                    g
                },
                policies,
            )
            .unwrap(),
        )
        .unwrap(),
        ApplicationContext::application_supplied(
            binding.clone(),
            Some(
                SchemaSnapshot::application_supplied(
                    binding.clone(),
                    SchemaIdentity::new(if variant == 8 {
                        "other_schema"
                    } else {
                        "schema"
                    })
                    .unwrap(),
                    if variant == 9 {
                        Generation::new(2).unwrap()
                    } else {
                        g
                    },
                    vec![facts],
                )
                .unwrap(),
            ),
            Some(SubjectIdentity::new(subject).unwrap()),
            Some(TenantSnapshot::application_supplied(binding, tenant(), Value::I64(t)).unwrap()),
        )
        .unwrap(),
    )
    .unwrap()
}
fn select(d: Dialect) -> Draft {
    Draft::select(
        table(),
        fields(d)
            .into_iter()
            .map(|c| Projection::new(c, None))
            .collect(),
    )
}
fn write(v: Value, c: Column) -> Vec<(Column, Input)> {
    vec![(c, Input::Value(v))]
}
fn changed() -> Column {
    col("txt", SqlType::Text, true)
}

#[test]
fn common_gate_all_families_and_contexts() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, 11, "subject", vec![]);
        for op in [
            select(d).freeze(),
            select(d).count().unwrap().freeze(),
            Draft::update(table(), write(Value::Text(String::new()), changed()))
                .filter(filter(1))
                .freeze(),
            Draft::delete(table()).filter(filter(1)).freeze(),
        ] {
            assert!(prepare(&c, &c, &op, d).is_ok());
            assert_eq!(
                prepare(&c, &context(d, 22, "subject", vec![]), &op, d).unwrap_err(),
                ExecutionError::ContextMismatch
            );
            assert_eq!(
                prepare(&c, &context(d, 11, "other", vec![]), &op, d).unwrap_err(),
                ExecutionError::ContextMismatch
            );
        }
        assert_eq!(
            prepare(
                &c,
                &c,
                &Draft::insert(table(), write(Value::I64(1), key())).freeze(),
                d
            )
            .unwrap_err(),
            ExecutionError::RiskRefused
        );
    }
}

// Authorization fake is deliberately not a real network/authentication service.
struct AuthorizationProbe(std::sync::Arc<std::sync::atomic::AtomicUsize>);
impl crate::authorization::TrustedAuthorizationAdapter for AuthorizationProbe {
    fn authorize<'a>(
        &'a self,
        r: crate::authorization::AuthorizationRequest<'a>,
    ) -> std::pin::Pin<
        Box<
            dyn std::future::Future<Output = crate::authorization::AuthorizationDecision>
                + Send
                + 'a,
        >,
    > {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Box::pin(async move { r.respond(crate::authorization::AdapterResponse::AllowCandidate) })
    }
}

// These tests fail if explicitly selected without dedicated target opt-in. Normal
// DB-free CI commands explicitly exclude live_; required live CI selects them.
macro_rules! atomic_cases {
    ($session:ident, $borrowed:ident, $connection:ty, $options:ident, $d:ident, $c:ident, $observer:ident, $create:literal, $seed:literal) => {{
        let d=$d; let options=&$options; let observer=&mut $observer; let c=&$c;
        // Reset this run's disposable fixture only. Existing regression assertions ran first.
        sqlx::query("DROP TABLE quent_dispatch_test.items").execute(&mut *observer).await.unwrap();
        sqlx::query($create).execute(&mut *observer).await.unwrap();
        sqlx::query($seed).execute(&mut *observer).await.unwrap();
        let one=ExpectedRows::exactly(1).unwrap();
        let zero=ExpectedRows::exactly(0).unwrap();
        let two=ExpectedRows::exactly(2).unwrap();
        let make=|n| Draft::update(table(),write(Value::Text("atomic".into()),changed())).filter(filter(n)).freeze();
        let observe= "SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1";
        let mut s=$session::connect(options.clone(),c.clone()).await.unwrap();
        let provider_calls=std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let auth=crate::authorization::AuthorizationContext::register(c.clone(),"provider".into(),1,vec![],AuthorizationProbe(provider_calls.clone())).unwrap();
        let high=Draft::update(table(),write(Value::Text("CANARY_AUTH".into()),changed())).filter(eq(tenant(),Value::I64(11))).freeze();
        let candidate=auth.candidate(&high,crate::authorization::AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
        assert_eq!(provider_calls.load(std::sync::atomic::Ordering::SeqCst),0);
        let acquisition=crate::authorization::acquire_authorization(candidate,&auth).await.unwrap();
        assert_eq!(provider_calls.load(std::sync::atomic::Ordering::SeqCst),1);
        assert_eq!(s.execute(c,high).await.unwrap_err(),ExecutionError::RiskRefused);
        assert_eq!(s.execute(c,Draft::delete(table()).filter(eq(tenant(),Value::I64(11))).freeze()).await.unwrap_err(),ExecutionError::RiskRefused);
        assert!(s.recorder.calls.is_empty());assert!(s.recorder.events.is_empty());
        drop(acquisition);
        let mut ordinary=$session::connect(options.clone(),c.clone()).await.unwrap();
        ordinary.execute(c,make(1)).await.unwrap();
        assert_eq!(ordinary.recorder.calls.len(),1);
        assert_eq!(provider_calls.load(std::sync::atomic::Ordering::SeqCst),1);
        let op=make(1);
        // Public display mutation cannot change the private shared/atomic gate.
        let prep=c.prepare().prepare_operation(&op).unwrap();
        #[cfg(all(feature="local-details",target_os="linux"))] export_does_not_change_gate(&prep);
        let mut public=prep.public_view(); public.risk=ViewRisk::Blocked;
        public.scope=Scope::Unknown; public.completeness=Completeness::Unknown;
        let _=public.to_json(); let _=format!("{public:?} {public}");
        assert!(s.recorder.calls.is_empty()); assert!(s.recorder.events.is_empty());
        assert_eq!(prep.inspect().risk(),Risk::Low);
        let blocked=Draft::delete(table()).freeze();
        let mut fake=c.prepare().prepare_operation(&blocked).unwrap().public_view();
        fake.risk=ViewRisk::Low; fake.scope=Scope::Single; fake.completeness=Completeness::Complete;
        let _=fake.to_json();
        assert_eq!(s.validate_atomic(c,&blocked,one,MAX_TTL).unwrap_err(),ExecutionError::RiskRefused);
        assert!(s.recorder.calls.is_empty()); assert!(s.recorder.events.is_empty());
        let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        drop(s.execute_atomic(c,&op,one,h));
        assert!(s.is_available()); assert!(s.recorder.events.is_empty());
        let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        let r=s.execute_atomic(c,&op,one,h).await.unwrap();
        assert_eq!(r.state(),ResultState::Committed);
        assert_eq!(provider_calls.load(std::sync::atomic::Ordering::SeqCst),1);
        #[cfg(feature="output-tracing")] output_capture::check(&r.public_view(),&r.public_view().to_json());
        let mut view=r.public_view(); assert_eq!(view.state,ViewState::Committed);
        view.state=ViewState::OutcomeUnknown; let _=view.to_json();
        assert_eq!(r.state(),ResultState::Committed);
        assert_eq!(s.recorder.events, ["begin","begun","send","decoded","expectation","commit","committed"]);
        assert_eq!(sqlx::query(observe).fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"atomic");
        let cb=context(d,22,"subject_b",vec![]);let mut sb=$session::connect(options.clone(),cb.clone()).await.unwrap();
        let opb=make(2);let hb=sb.validate_atomic(&cb,&opb,one,MAX_TTL).unwrap();sb.execute_atomic(&cb,&opb,one,hb).await.unwrap();
        assert_eq!(sqlx::query(observe).fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"atomic");
        assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=22 AND id=2").fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"atomic");
        let protected=context(d,11,"subject",vec![PolicyDeclaration::Readonly(changed())]);
        let blocked=$session::connect(options.clone(),protected.clone()).await.unwrap();assert!(blocked.validate_atomic(&protected,&op,one,MAX_TTL).is_err());assert!(blocked.recorder.events.is_empty());
        // Both drivers report an unchanged matched UPDATE as one, not changed zero.
        s.recorder=Recorder::default();
        let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        let r=s.execute_atomic(c,&op,one,h).await.unwrap();
        assert_eq!(r.affected_rows(), Some(if d==Dialect::Postgres {AffectedRows::PostgresMatched(1)}else{AffectedRows::MysqlFoundRows(1)}));
        for (n,expected) in [(1,zero),(1,two),(999,one)] {
            sqlx::query("UPDATE quent_dispatch_test.items SET txt='before' WHERE tenant=11 AND id=1").execute(&mut *observer).await.unwrap();
            let op=make(n); s.recorder=Recorder::default();
            let h=s.validate_atomic(c,&op,expected,MAX_TTL).unwrap();
            let e=s.execute_atomic(c,&op,expected,h).await.unwrap_err();
            assert_eq!(e,AtomicFailure{cause:ExecutionError::ExpectationMismatch,disposition:AtomicDisposition::RolledBack});
            assert_eq!(e.public_view(), DiagnosticView { code:DiagnosticCode::ExecutionExpectationMismatch, state:Some(ViewState::RolledBack) });
            assert_eq!(s.recorder.events,["begin","begun","send","decoded","expectation","rollback","rolled_back"]);
            assert_eq!(sqlx::query(observe).fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"before");
        }
        let op=make(999); let h=s.validate_atomic(c,&op,zero,MAX_TTL).unwrap();
        assert_eq!(s.execute_atomic(c,&op,zero,h).await.unwrap().state(),ResultState::Committed);
        // Version is an additional condition: the original key and tenant proof stays necessary.
        let versioned=|version| {
            let draft=Draft::update(table(),write(Value::Text("version".into()),changed())).filter(filter(1)).optimistic_version(key(),Value::I64(version)).unwrap();
            (if d==Dialect::Postgres {draft.returning(vec![Projection::new(key(),None),Projection::new(changed(),None)])}else{draft}).freeze()
        };
        let op=versioned(0); s.recorder=Recorder::default();
        let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        assert_eq!(s.execute_atomic(c,&op,one,h).await.unwrap_err(),AtomicFailure{cause:ExecutionError::Conflict,disposition:AtomicDisposition::RolledBack});
        assert_eq!(sqlx::query(observe).fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"before");
        let op=versioned(1); let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        s.execute_atomic(c,&op,one,h).await.unwrap();
        assert_eq!(sqlx::query("SELECT id,txt FROM quent_dispatch_test.items WHERE tenant=11").fetch_one(&mut *observer).await.unwrap().get::<i64,_>(0),2);
        sqlx::query("UPDATE quent_dispatch_test.items SET id=1,txt='before' WHERE tenant=11").execute(&mut *observer).await.unwrap();
        for value in [Value::I64(i64::MAX),Value::Null(SqlType::I64),Value::U64(0)] {
            assert!(Draft::update(table(),write(Value::Text("x".into()),changed())).filter(filter(1)).optimistic_version(key(),value).is_err());
        }
        let op=Draft::update(table(),write(Value::Text("x".into()),changed())).optimistic_version(key(),Value::I64(1)).unwrap().freeze();
        let before=s.recorder.calls.len(); assert!(s.validate_atomic(c,&op,one,MAX_TTL).is_err()); assert_eq!(before,s.recorder.calls.len());
        // Exact binding covers expectations, version and final projection.
        let op=make(1); let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        assert_eq!(s.execute_atomic(c,&op,two,h).await.unwrap_err().disposition(),AtomicDisposition::NotStarted);
        let other=context(d,22,"subject",vec![]); let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        assert_eq!(s.execute_atomic(&other,&op,one,h).await.unwrap_err().cause(),ExecutionError::ContextMismatch);
        let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        assert_eq!(s.execute_atomic(c,&versioned(1),one,h).await.unwrap_err().cause(),ExecutionError::OperationMismatch);
        let mut h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap(); h.operation.expires=Instant::now();
        assert_eq!(s.execute_atomic(c,&op,one,h).await.unwrap_err().cause(),ExecutionError::Expired);
        // Expiry after begin: runner's final gate rechecks after that await.
        s.recorder=Recorder::default(); let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap(); s.final_now=Some(h.operation.expires);
        assert_eq!(s.execute_atomic(c,&op,one,h).await.unwrap_err().disposition(),AtomicDisposition::RolledBack);
        assert!(s.recorder.calls.is_empty()); assert!(!s.recorder.events.contains(&"commit"));
        // Borrowed normal result stays uncommitted; atomic entry rejects without send.
        let mut raw=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
        let mut tx=raw.begin().await.unwrap();
        let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();
        let _physical_id=borrowed.test_connection_id().await;
        assert_eq!(borrowed.validate_atomic(c,&op,one,MAX_TTL).unwrap_err(),ExecutionError::Unsupported);
        assert!(borrowed.recorder.calls.is_empty()); assert!(borrowed.recorder.events.is_empty());
        let r=borrowed.execute(c,op.clone()).await.unwrap(); assert_eq!(r.state(),ResultState::Uncommitted);
        assert_eq!(r.public_view().state,ViewState::Uncommitted);
        assert_eq!(sqlx::query(observe).fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"before");
        let stale=borrowed.validate(c,&op,MAX_TTL).unwrap();
        drop(borrowed); tx.rollback().await.unwrap();
        assert_eq!(s.execute_validated(c,&op,stale).await.unwrap_err(),ExecutionError::ContextMismatch);
        let mut tx=raw.begin().await.unwrap(); let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();
        let from_session=s.validate(c,&op,MAX_TTL).unwrap();
        assert_eq!(borrowed.execute_validated(c,&op,from_session).await.unwrap_err(),ExecutionError::ContextMismatch);
        let old=borrowed.validate(c,&op,MAX_TTL).unwrap(); drop(borrowed); tx.rollback().await.unwrap();
        let mut tx=raw.begin().await.unwrap(); let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();
        assert_eq!(borrowed.execute_validated(c,&op,old).await.unwrap_err(),ExecutionError::ContextMismatch);
        drop(borrowed); tx.rollback().await.unwrap();
        let mut tx=raw.begin().await.unwrap();let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();
        assert_eq!(borrowed.validate_atomic(c,&op,one,MAX_TTL).unwrap_err(),ExecutionError::Unsupported);
        borrowed.execute(c,op.clone()).await.unwrap();
        assert!(!borrowed.recorder.events.contains(&"commit"));assert!(!borrowed.recorder.events.contains(&"rollback"));
        drop(borrowed);tx.commit().await.unwrap();
        assert_eq!(sqlx::query(observe).fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"atomic");
        sqlx::query("UPDATE quent_dispatch_test.items SET txt='before' WHERE tenant=11 AND id=1").execute(&mut *observer).await.unwrap();
        let ret=Draft::update(table(),write(Value::Text("returning".into()),changed())).filter(filter(1)).returning(vec![Projection::new(key(),None),Projection::new(changed(),None)]).freeze();
        if d==Dialect::Mysql {
            s.recorder=Recorder::default(); assert!(s.validate_atomic(c,&ret,one,MAX_TTL).is_err());
            assert!(s.recorder.calls.is_empty()); assert!(s.recorder.events.is_empty());
        } else {
            let h=s.validate_atomic(c,&ret,one,MAX_TTL).unwrap(); let r=s.execute_atomic(c,&ret,one,h).await.unwrap();
            assert_eq!(r.sensitive_rows(),&[vec![Value::I64(1),Value::Text("returning".into())]]);
            // PII projection and protected version assignment refuse via the same gate.
            let pii=context(d,11,"subject",vec![PolicyDeclaration::Pii(changed())]);
            let ps=$session::connect(options.clone(),pii.clone()).await.unwrap(); assert!(ps.validate_atomic(&pii,&ret,one,MAX_TTL).is_err());
            for policy in [PolicyDeclaration::Readonly(key()),PolicyDeclaration::Generated(key())] {
                let pc=context(d,11,"subject",vec![policy]); let ps=$session::connect(options.clone(),pc.clone()).await.unwrap();
                assert!(ps.validate_atomic(&pc,&versioned(1),one,MAX_TTL).is_err());
            }
        }
        for policy in [PolicyDeclaration::Tenant(tenant()),PolicyDeclaration::Readonly(key()),PolicyDeclaration::Generated(key())] {
            let pc=context(d,11,"subject",if matches!(policy,PolicyDeclaration::Tenant(_)){vec![]}else{vec![policy]});
            let ps=$session::connect(options.clone(),pc.clone()).await.unwrap();
            let version_col=if pc.same_snapshot(c){tenant()}else{key()};
            let op=Draft::update(table(),write(Value::Text("x".into()),changed())).filter(filter(1)).optimistic_version(version_col,Value::I64(11)).unwrap().freeze();
            assert!(ps.validate_atomic(&pc,&op,one,MAX_TTL).is_err()); assert!(ps.recorder.calls.is_empty());
        }
        if d==Dialect::Postgres {
            let protected=col("small",SqlType::I16,true);
            let ret=Draft::update(table(),write(Value::Text("x".into()),changed())).filter(filter(1)).returning(vec![Projection::new(protected.clone(),None),Projection::new(tenant(),None)]).freeze();
            let pc=context(d,11,"subject",vec![PolicyDeclaration::Pii(protected.clone())]);let ps=$session::connect(options.clone(),pc.clone()).await.unwrap();
            assert_eq!(ps.validate_atomic(&pc,&ret,one,MAX_TTL).unwrap_err(),ExecutionError::Policy(PolicyError::PiiAccess));
            let pc=context(d,11,"subject",vec![PolicyDeclaration::Readonly(protected)]);let mut ps=$session::connect(options.clone(),pc.clone()).await.unwrap();
            let h=ps.validate_atomic(&pc,&ret,one,MAX_TTL).unwrap();let r=ps.execute_atomic(&pc,&ret,one,h).await.unwrap();assert_eq!(r.sensitive_rows()[0][1],Value::I64(11));
        }
        // Finite test-only completion failures. These are not network accidents.
        for fault in [AtomicFault::Begin,AtomicFault::Commit,AtomicFault::CommitReplyLost,AtomicFault::Rollback,AtomicFault::RollbackReplyLost] {
            sqlx::query("UPDATE quent_dispatch_test.items SET txt='before' WHERE tenant=11 AND id=1").execute(&mut *observer).await.unwrap();
            let mut faulted=$session::connect(options.clone(),c.clone()).await.unwrap();
            let op=make(1); let expected=if matches!(fault,AtomicFault::Rollback|AtomicFault::RollbackReplyLost){zero}else{one};
            let h=faulted.validate_atomic(c,&op,expected,MAX_TTL).unwrap(); faulted.atomic_fault=Some(fault);
            let e=faulted.execute_atomic(c,&op,expected,h).await.unwrap_err();
            assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown); assert_eq!(e.public_view().state,Some(ViewState::OutcomeUnknown)); assert!(!faulted.is_available());
            #[cfg(feature="output-tracing")] output_capture::check(&e.public_view(),&e.public_view().to_json());
            assert!(std::error::Error::source(&e).is_none());
            assert_eq!(faulted.execute(c,op).await.unwrap_err(),ExecutionError::SessionUnavailable);
            assert!(faulted.recorder.calls.len()<=1);
            drop(faulted);
            let actual=sqlx::query(observe).fetch_one(&mut *observer).await.unwrap().get::<String,_>(0);
            assert_eq!(actual,if fault==AtomicFault::CommitReplyLost {"atomic"}else{"before"});
        }
        // Constructed but unpolled was tested above. Here begin completed and a
        // test-only final barrier prevents the business send; drop is unconfirmed.
        let mut canceled=$session::connect(options.clone(),c.clone()).await.unwrap();
        let op=make(1); let h=canceled.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        canceled.final_barrier=Some(Arc::new(tokio::sync::Notify::new()));
        let trace=canceled.recorder.live.clone();
        let mut f=Box::pin(canceled.execute_atomic(c,&op,one,h));
        tokio::time::timeout(Duration::from_secs(10),async {
            loop {assert!(poll_once(f.as_mut()).await.is_pending());if trace.lock().unwrap().contains(&"begun"){break;} tokio::task::yield_now().await;}
        }).await.unwrap(); drop(f);
        assert_eq!(*trace.lock().unwrap(),["begin","begun"]); assert!(!canceled.is_available()); drop(canceled);
        // Real row-lock cancellation, timeout and termination of only this run's
        // exact business connection. Observer queries use the dedicated test role.
        for mode in [0,1,2] {
            sqlx::query("UPDATE quent_dispatch_test.items SET txt='before' WHERE tenant=11 AND id=1").execute(&mut *observer).await.unwrap();
            let mut locker=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
            let mut lock=locker.begin().await.unwrap();
            sqlx::query("UPDATE quent_dispatch_test.items SET txt=txt WHERE tenant=11 AND id=1").execute(&mut *lock).await.unwrap();
            let mut canceled=$session::connect(options.clone(),c.clone()).await.unwrap();
            let connection_id=canceled.test_connection_id().await;
            let op=make(1); let h=canceled.validate_atomic(c,&op,one,MAX_TTL).unwrap();
            let trace=canceled.recorder.live.clone();
            let mut f=Box::pin(canceled.execute_atomic(c,&op,one,h));
            let waiting=if d==Dialect::Postgres {"SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user AND wait_event_type='Lock' AND query LIKE 'UPDATE %'"} else {"SELECT COUNT(*) FROM performance_schema.data_lock_waits w JOIN performance_schema.threads t ON t.THREAD_ID=w.REQUESTING_THREAD_ID WHERE t.PROCESSLIST_ID=?"};
            tokio::time::timeout(Duration::from_secs(10),async {
                loop {assert!(poll_once(f.as_mut()).await.is_pending()); let r=sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(connection_id).fetch_one(&mut *observer).await.unwrap();if r.get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}
            }).await.expect("atomic UPDATE not observed in server lock wait");
            assert_eq!(*trace.lock().unwrap(),["begin","begun","send"]);
            if mode==2 {
                if d==Dialect::Postgres {
                    sqlx::query("SELECT pg_terminate_backend(CAST($1 AS INT))").bind(connection_id).execute(&mut *observer).await.unwrap();
                } else {
                    // ID is obtained directly from the dedicated connection; no user SQL input.
                    sqlx::query(sqlx::AssertSqlSafe(format!("KILL CONNECTION {connection_id}"))).execute(&mut *observer).await.unwrap();
                }
                let e=tokio::time::timeout(Duration::from_secs(10),f.as_mut()).await.unwrap().unwrap_err();
                assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown); assert_eq!(e.public_view().state,Some(ViewState::OutcomeUnknown));
            } else if mode==1 {assert!(tokio::time::timeout(Duration::from_millis(1),f.as_mut()).await.is_err());}
            drop(f); assert!(!canceled.is_available());
            assert!(!trace.lock().unwrap().contains(&"commit")); assert!(!trace.lock().unwrap().contains(&"rolled_back"));
            drop(canceled); lock.rollback().await.unwrap();
            assert_eq!(sqlx::query(observe).fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"before");
            println!("PR3 {:?}: real atomic lock observed; mode={} (drop/timeout/disconnect); no commit/confirmed rollback; session quarantined; observer unchanged",d,mode);
        }
        // Deliberately inaccurate asserted key in a disposable fixture: no gate bypass.
        // This demonstrates why application schema assertions are conditional evidence.
        let drop_key=if d==Dialect::Postgres {"ALTER TABLE quent_dispatch_test.items DROP CONSTRAINT items_pkey"} else {"ALTER TABLE quent_dispatch_test.items DROP PRIMARY KEY"};
        sqlx::query(sqlx::AssertSqlSafe(drop_key.to_owned())).execute(&mut *observer).await.unwrap();
        sqlx::query("UPDATE quent_dispatch_test.items SET txt='before',optional=1 WHERE tenant=11").execute(&mut *observer).await.unwrap();
        sqlx::query("INSERT INTO quent_dispatch_test.items(id,tenant,txt,optional) VALUES(1,11,'before',NULL)").execute(&mut *observer).await.unwrap();
        s.recorder=Recorder::default(); let op=make(1); let h=s.validate_atomic(c,&op,one,MAX_TTL).unwrap();
        assert_eq!(s.execute_atomic(c,&op,one,h).await.unwrap_err().cause(),ExecutionError::ExpectationMismatch);
        assert_eq!(s.recorder.events.iter().filter(|e|**e=="rollback").count(),1);
        assert!(!s.recorder.events.contains(&"commit"));
        for row in sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11").fetch_all(&mut *observer).await.unwrap() {assert_eq!(row.get::<String,_>(0),"before");}
        if d==Dialect::Postgres {
            let ret=Draft::update(table(),write(Value::Text("full returning".into()),changed())).filter(filter(1)).returning(vec![Projection::new(key(),None),Projection::new(tenant(),None)]).freeze();
            let h=s.validate_atomic(c,&ret,two,MAX_TTL).unwrap();let result=s.execute_atomic(c,&ret,two,h).await.unwrap();assert_eq!(result.sensitive_rows().len(),2);
            assert_eq!(result.sensitive_rows(),&[vec![Value::I64(1),Value::I64(11)],vec![Value::I64(1),Value::I64(11)]]);
            sqlx::query("UPDATE quent_dispatch_test.items SET txt='before' WHERE tenant=11").execute(&mut *observer).await.unwrap();
            let mut columns=fields(d); let last=col("optional",SqlType::I64,false); columns[7]=last.clone();
            let badc=context_schema(d,11,"subject",vec![],0,columns);
            let mut bads=$session::connect(options.clone(),badc.clone()).await.unwrap();
            let op=Draft::update(table(),write(Value::Text("bad decode".into()),changed())).filter(filter(1)).returning(vec![Projection::new(key(),None),Projection::new(last,None)]).freeze();
            let h=bads.validate_atomic(&badc,&op,two,MAX_TTL).unwrap();
            let e=bads.execute_atomic(&badc,&op,two,h).await.unwrap_err(); assert_eq!(e.cause(),ExecutionError::Decode);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);
            assert!(!bads.recorder.events.contains(&"commit"));
            assert_eq!(bads.recorder.events.iter().filter(|e|**e=="row_decoded").count(),1,"first full row decoded before later-row NULL error");
            for row in sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11").fetch_all(&mut *observer).await.unwrap(){assert_eq!(row.get::<String,_>(0),"before");}
            // Physical native mismatch in the later column.
            let mut columns=fields(d); let last=col("small",SqlType::I32,true); columns[2]=last.clone();
            let badc=context_schema(d,11,"subject",vec![],0,columns); let mut bads=$session::connect(options.clone(),badc.clone()).await.unwrap();
            let op=Draft::update(table(),write(Value::Text("bad type".into()),changed())).filter(filter(1)).returning(vec![Projection::new(key(),None),Projection::new(last,None)]).freeze();
            let h=bads.validate_atomic(&badc,&op,two,MAX_TTL).unwrap(); assert_eq!(bads.execute_atomic(&badc,&op,two,h).await.unwrap_err().cause(),ExecutionError::Decode);
            // Business write is small; a different returned text column exceeds the result budget.
            sqlx::query("UPDATE quent_dispatch_test.items SET txt=repeat('x',4097) WHERE tenant=11").execute(&mut *observer).await.unwrap();
            let op=Draft::update(table(),write(Value::I32(7),col("normal",SqlType::I32,true))).filter(filter(1)).returning(vec![Projection::new(changed(),None)]).freeze();
            let h=s.validate_atomic(c,&op,two,MAX_TTL).unwrap(); assert_eq!(s.execute_atomic(c,&op,two,h).await.unwrap_err().cause(),ExecutionError::ResultLimit);
            for row in sqlx::query("SELECT normal FROM quent_dispatch_test.items WHERE tenant=11").fetch_all(&mut *observer).await.unwrap(){assert_ne!(row.get::<Option<i32>,_>(0),Some(7));}
        }
        if d==Dialect::Postgres {
            sqlx::query("UPDATE quent_dispatch_test.items SET txt=repeat('x',4096) WHERE tenant=11").execute(&mut *observer).await.unwrap();
            sqlx::query("INSERT INTO quent_dispatch_test.items(id,tenant,txt) SELECT 1,11,repeat('x',4096) FROM generate_series(1,254)").execute(&mut *observer).await.unwrap();
            let op=Draft::update(table(),write(Value::I32(7),col("normal",SqlType::I32,true))).filter(filter(1)).returning(vec![Projection::new(key(),None),Projection::new(changed(),None)]).freeze();
            let expected=ExpectedRows::exactly(256).unwrap();let h=s.validate_atomic(c,&op,expected,MAX_TTL).unwrap();
            assert_eq!(s.execute_atomic(c,&op,expected,h).await.unwrap_err().cause(),ExecutionError::ResultLimit);
            sqlx::query("UPDATE quent_dispatch_test.items SET txt='small' WHERE tenant=11").execute(&mut *observer).await.unwrap();
            sqlx::query("INSERT INTO quent_dispatch_test.items(id,tenant,txt) VALUES(1,11,'small')").execute(&mut *observer).await.unwrap();
            let h=s.validate_atomic(c,&op,expected,MAX_TTL).unwrap();assert_eq!(s.execute_atomic(c,&op,expected,h).await.unwrap_err().cause(),ExecutionError::ResultLimit);
            for row in sqlx::query("SELECT normal FROM quent_dispatch_test.items WHERE tenant=11").fetch_all(&mut *observer).await.unwrap(){assert_ne!(row.get::<Option<i32>,_>(0),Some(7));}
            // Restore exactly two rows for the following DELETE check.
            sqlx::query("DELETE FROM quent_dispatch_test.items WHERE tenant=11").execute(&mut *observer).await.unwrap();
            sqlx::query("INSERT INTO quent_dispatch_test.items(id,tenant,txt) VALUES(1,11,'before'),(1,11,'before')").execute(&mut *observer).await.unwrap();
        }
        let draft=Draft::delete(table()).filter(filter(1));
        let delete=(if d==Dialect::Postgres {draft.returning(vec![Projection::new(key(),None),Projection::new(tenant(),None)])}else{draft}).freeze();
        let h=s.validate_atomic(c,&delete,one,MAX_TTL).unwrap();assert_eq!(s.execute_atomic(c,&delete,one,h).await.unwrap_err().disposition(),AtomicDisposition::RolledBack);
        let h=s.validate_atomic(c,&delete,two,MAX_TTL).unwrap();let r=s.execute_atomic(c,&delete,two,h).await.unwrap();
        if d==Dialect::Postgres {assert_eq!(r.sensitive_rows().len(),2);}
        assert_eq!(sqlx::query("SELECT COUNT(*) FROM quent_dispatch_test.items WHERE tenant=11").fetch_one(&mut *observer).await.unwrap().get::<i64,_>(0),0);
        println!("PR3 {:?}: production begin/send/full consumption/expectation/commit once; unchanged matched=1; mismatch 0/1/multiple, version conflict rollback; independent observer verified; borrowed uncommitted and cross-Tx refusal; RETURNING {}",d,if d==Dialect::Postgres {"full consumption/decode/budget"}else{"unsupported, zero send"});
    }};
}
#[cfg(feature = "operation-spec")]
fn spec_mapping(c: &DatabaseContext, d: Dialect) -> crate::operation_spec::TypedModelContract {
    spec_mapping_columns(c, fields(d))
}
#[cfg(feature = "operation-spec")]
fn spec_mapping_columns(
    c: &DatabaseContext,
    fields: Vec<Column>,
) -> crate::operation_spec::TypedModelContract {
    use crate::operation_spec::*;
    let columns = fields
        .into_iter()
        .map(|c| {
            TypedColumn::application_supplied(
                c.sensitive_name().sensitive_text(),
                c.sensitive_name().clone(),
                ColumnType::Native(c.sql_type()),
                c.nullable(),
                if c == tenant() {
                    ColumnRole::Tenant
                } else {
                    ColumnRole::Ordinary
                },
                true,
            )
            .unwrap()
        })
        .collect();
    TypedModelContract::application_asserted("items", table(), columns, ModelCoverage::Complete, c)
        .unwrap()
}
#[cfg(feature = "operation-spec")]
fn read_spec(d: Dialect) -> crate::operation_spec::SpecSource {
    let names: Vec<_> = fields(d)
        .iter()
        .map(|c| c.sensitive_name().sensitive_text().to_owned())
        .collect();
    crate::operation_spec::SpecSource::parse(&serde_json::to_vec(&serde_json::json!({"kind":"quent_operation_spec","version":1,"model":"items","fields":names,"filter":{"op":"eq","field":"id","value":{"type":"i64","value":1}},"order":[],"limit":10})).unwrap()).unwrap()
}
macro_rules! live_suite {
    ($name:ident, $feature:literal, $session:ident, $borrowed:ident, $connection:ty, $options:ty, $dialect:expr, $dsn:literal, $create:literal, $seed:literal) => {
        #[cfg(feature=$feature)]
        #[test]
        fn $name() {
            use sqlx::{Connection, ConnectOptions, Row};
            assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(), Ok("quent_dispatch_test"), "dedicated disposable DB opt-in required");
            let dsn = std::env::var($dsn).expect("explicit dedicated DB DSN required");
            let options: $options = dsn.parse().expect("invalid dedicated test DSN");
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            // Observe actual product connect/dispatch/Tx/error emissions too, not only
            // explicit view emission. Nested view checks use their own recorder.
            #[cfg(feature="output-tracing")] let lifetime_capture=output_capture::Capture::default();
            // Setup is deliberately separate raw SQL, never counted as dispatch.
            runtime.block_on(async {
                let mut setup = <$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                sqlx::query($create).execute(&mut setup).await.unwrap();
                sqlx::query($seed).execute(&mut setup).await.unwrap();
            });
            #[cfg(feature="output-tracing")] let subscriber=tracing::subscriber::set_default(lifetime_capture.clone());
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||runtime.block_on(async {
                let d = $dialect; let c = context(d,11,"subject",vec![]);
                let mut session = $session::connect(options.clone(),c.clone()).await.unwrap();
                #[cfg(feature="output-tracing")]
                if d==Dialect::Postgres {
                    // Deliberately hostile physical side effect, solely an output
                    // probe, NOT support evidence for trigger-backed table semantics.
                    let mut raw=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                    sqlx::query("CREATE FUNCTION quent_dispatch_test.output_canary() RETURNS trigger LANGUAGE plpgsql AS $$ BEGIN RAISE NOTICE 'CANARY_SQL_name_tenant_subject_reason_source_/CANARY/path'; RAISE EXCEPTION 'CANARY_ARBITRARY_DRIVER_CAUSE'; RETURN NEW; END $$").execute(&mut raw).await.unwrap();
                    sqlx::query("CREATE TRIGGER output_canary BEFORE UPDATE ON quent_dispatch_test.items FOR EACH ROW EXECUTE FUNCTION quent_dispatch_test.output_canary()").execute(&mut raw).await.unwrap();
                    let mut probe=$session::connect(options.clone(),c.clone()).await.unwrap();
                    let e=probe.execute(&c,Draft::update(table(),write(Value::Text("CANARY_VALUE".into()),changed())).filter(filter(1)).freeze()).await.unwrap_err();
                    assert_eq!(e,ExecutionError::OutcomeUnknown);assert!(std::error::Error::source(&e).is_none());
                    assert!(!format!("{e} {e:?}").contains("CANARY"));
                    output_capture::check(&e.public_view(),&e.public_view().to_json());
                    // Positive control: the real driver would expose this notice
                    // through an application-owned raw query without Quent's scope.
                    let raw_capture=output_capture::Capture::default();
                    {
                        let _raw_subscriber=tracing::subscriber::set_default(raw_capture.clone());
                        assert!(sqlx::query("UPDATE quent_dispatch_test.items SET txt=txt WHERE tenant=11 AND id=1").execute(&mut raw).await.is_err());
                    }
                    assert!(raw_capture.0.lock().unwrap().iter().any(|s|s.contains("CANARY_SQL_name_tenant_subject_reason_source")),"driver notice positive control missing");
                    sqlx::query("DROP TRIGGER output_canary ON quent_dispatch_test.items").execute(&mut raw).await.unwrap();
                    sqlx::query("DROP FUNCTION quent_dispatch_test.output_canary()").execute(&mut raw).await.unwrap();
                }
                #[cfg(feature="operation-spec")]
                {
                    let mapping=spec_mapping(&c,d);
                    let compiled=read_spec(d).compile(&mapping,&c).unwrap();
                    let inspection=compiled.inspect();
                    assert_eq!(inspection.scope(),Scope::Single);
                    assert_eq!(compiled.public_view(),inspection.public_view());
                    let record=inspection.sensitive_source();
                    let expected_sql=if d==Dialect::Postgres { "SELECT \"items\".\"id\", \"items\".\"tenant\", \"items\".\"small\", \"items\".\"normal\", \"items\".\"big\", \"items\".\"flag\", \"items\".\"txt\", \"items\".\"optional\" FROM \"quent_dispatch_test\".\"items\" WHERE ((\"items\".\"id\" = $1) AND (\"items\".\"tenant\" = $2)) LIMIT $3" } else { "SELECT `items`.`id`, `items`.`tenant`, `items`.`small`, `items`.`normal`, `items`.`big`, `items`.`flag`, `items`.`txt`, `items`.`optional`, `items`.`tiny`, `items`.`u_tiny`, `items`.`u_small`, `items`.`u_normal`, `items`.`u_big` FROM `quent_dispatch_test`.`items` WHERE ((`items`.`id` = ?) AND (`items`.`tenant` = ?)) LIMIT ?" };
                    assert_eq!(record.sql(),expected_sql);
                    let address=record.sql().as_ptr() as usize;
                    let args=vec![Value::I64(1),Value::I64(11),Value::I64(10)];
                    assert_eq!(record.args(),args);
                    let mut reads=$session::connect(options.clone(),c.clone()).await.unwrap();
                    let r=reads.execute_compiled_read(&c,&mapping,compiled).await.unwrap();
                    let mut expected=vec![Value::I64(1),Value::I64(11),Value::I16(i16::MIN),Value::I32(i32::MAX),Value::I64(i64::MIN),Value::Bool(false),Value::Text(String::new()),Value::Null(SqlType::I64)];
                    if d==Dialect::Mysql {expected.extend([Value::I8(i8::MIN),Value::U8(u8::MAX),Value::U16(u16::MAX),Value::U32(u32::MAX),Value::U64(u64::MAX)]);}
                    assert_eq!(r.sensitive_rows(),&[expected]);
                    assert_eq!(reads.recorder.calls,vec![(expected_sql.to_owned(),args)]);
                    assert_eq!(reads.recorder.sql_addresses,vec![address],"the inspected heap SQL allocation is the dispatched record; no re-render");
                    // Application assertions can be wrong: native decode still refuses
                    // unexpected NULL, with one send, no retry and quarantined session.
                    let mut declared=fields(d);
                    let optional=declared.iter_mut().find(|c|c.sensitive_name().sensitive_text()=="optional").unwrap();
                    *optional=col("optional",SqlType::I64,false);
                    let wrong=context_schema(d,11,"subject",vec![],0,declared.clone());
                    let wrong_mapping=spec_mapping_columns(&wrong,declared);
                    let compiled=read_spec(d).compile(&wrong_mapping,&wrong).unwrap();
                    let mut bad=$session::connect(options.clone(),wrong.clone()).await.unwrap();
                    assert_eq!(bad.execute_compiled_read(&wrong,&wrong_mapping,compiled).await.unwrap_err(),ExecutionError::Decode);
                    assert_eq!(bad.recorder.calls.len(),1);assert!(!bad.is_available());
                    let compiled=read_spec(d).compile(&wrong_mapping,&wrong).unwrap();
                    assert_eq!(bad.execute_compiled_read(&wrong,&wrong_mapping,compiled).await.unwrap_err(),ExecutionError::SessionUnavailable);
                    assert_eq!(bad.recorder.calls.len(),1);
                    let before=reads.recorder.calls.len();
                    for variant in 1..=9 {
                        let changed=context_variant(d,11,"subject",vec![],variant);
                        let compiled=read_spec(d).compile(&mapping,&c).unwrap();
                        assert_eq!(reads.execute_compiled_read(&changed,&mapping,compiled).await.unwrap_err(),ExecutionError::ContextMismatch);
                    }
                    let changed=context(d,22,"subject",vec![]);
                    let compiled=read_spec(d).compile(&mapping,&c).unwrap();
                    assert_eq!(reads.execute_compiled_read(&changed,&mapping,compiled).await.unwrap_err(),ExecutionError::ContextMismatch);
                    let changed_mapping=spec_mapping(&changed,d);
                    let compiled=read_spec(d).compile(&mapping,&c).unwrap();
                    assert_eq!(reads.execute_compiled_read(&c,&changed_mapping,compiled).await.unwrap_err(),ExecutionError::ContextMismatch);
                    assert_eq!(reads.recorder.calls.len(),before);
                    // Typed NULL/NOT remain partial scope analysis, never execution authority.
                    let mut wire=read_spec(d).sensitive_source().clone();
                    wire["filter"]=serde_json::json!({"op":"not","arg":{"op":"is_null","field":"optional"}});
                    let partial=crate::operation_spec::SpecSource::parse(&serde_json::to_vec(&wire).unwrap()).unwrap().compile(&mapping,&c).unwrap();
                    assert!(matches!(reads.execute_compiled_read(&c,&mapping,partial).await,Err(ExecutionError::RiskRefused|ExecutionError::Incomplete)));
                    assert_eq!(reads.recorder.calls.len(),before);
                    let compiled=read_spec(d).compile(&mapping,&c).unwrap();
                    drop(reads.execute_compiled_read(&c,&mapping,compiled));
                    assert_eq!(reads.recorder.calls.len(),before);
                    println!("QT-AI-06 typed {:?}: identical record allocation/SQL/ordered args, native+NULL rows, 1 send; context/mapping/high refusals and unpolled future 0 sends",d);
                }
                // PR2 uses the real production dispatcher, separate finite recorder.
                let mut delayed = $session::connect(options.clone(), c.clone()).await.unwrap();
                let op = select(d).filter(filter(1)).freeze();
                for ttl in [Duration::ZERO, MAX_TTL + Duration::from_nanos(1), Duration::MAX] {
                    assert_eq!(delayed.validate(&c, &op, ttl).unwrap_err(), ExecutionError::InvalidTtl);
                }
                let h = delayed.validate(&c, &op, MAX_TTL).unwrap();
                assert_eq!(format!("{h:?}"), "ValidatedOperation([REDACTED])");
                drop(delayed.execute_validated(&c, &op, h));
                assert!(delayed.recorder.calls.is_empty());
                let mut mutations = vec![
                    select(d).filter(filter(2)).freeze(),
                    select(d).filter(Predicate::And(vec![filter(1)])).freeze(),
                    select(d).filter(eq(col("id",SqlType::I32,false),Value::I32(1))).freeze(),
                    select(d).filter(eq(key(),Value::Null(SqlType::I64))).freeze(),
                    select(d).filter(filter(1)).limit(1).freeze(),
                    select(d).filter(filter(1)).order(vec![Order{column:key(),direction:Direction::Desc}]).freeze(),
                    Draft::select(table(),vec![Projection::new(key(),None)]).filter(filter(1)).freeze(),
                    Draft::update(table(),vec![(changed(),Input::Missing)]).filter(filter(1)).freeze(),
                    Draft::delete(table()).filter(filter(1)).freeze(),
                    select(d).filter(Predicate::And(vec![filter(1);257])).freeze(),
                ];
                let mut deep=filter(1); for _ in 0..18 { deep=Predicate::Not(Box::new(deep)); }
                mutations.push(select(d).filter(deep).freeze());
                for changed in mutations {
                    let h=delayed.validate(&c,&op,MAX_TTL).unwrap();
                    assert_eq!(delayed.execute_validated(&c,&changed,h).await.unwrap_err(),ExecutionError::OperationMismatch);
                }
                let write_original=Draft::update(table(),vec![(changed(),Input::Value(Value::Text("original".into()))),(col("normal",SqlType::I32,true),Input::Value(Value::I32(1)))]).filter(filter(1)).freeze();
                for pairs in [
                    vec![(changed(),Input::Value(Value::Text("different".into()))),(col("normal",SqlType::I32,true),Input::Value(Value::I32(1)))],
                    vec![(col("normal",SqlType::I32,true),Input::Value(Value::I32(1))),(changed(),Input::Value(Value::Text("original".into())))],
                    vec![(changed(),Input::Missing)], vec![(changed(),Input::Value(Value::Null(SqlType::Text)))]
                ] {
                    let h=delayed.validate(&c,&write_original,MAX_TTL).unwrap();
                    let changed_write=Draft::update(table(),pairs).filter(filter(1)).freeze();
                    assert_eq!(delayed.execute_validated(&c,&changed_write,h).await.unwrap_err(),ExecutionError::OperationMismatch);
                }
                for current in (1..=9).map(|v|context_variant(d,11,"subject",vec![],v)).chain([
                    context(d,22,"subject",vec![]), context(d,11,"other",vec![]),
                    context(d,11,"subject",vec![PolicyDeclaration::Readonly(changed())]),
                    context(if d==Dialect::Postgres {Dialect::Mysql} else {Dialect::Postgres},11,"subject",vec![])
                ]) {
                    let h=delayed.validate(&c,&op,MAX_TTL).unwrap();
                    assert_eq!(delayed.execute_validated(&current,&op,h).await.unwrap_err(),ExecutionError::ContextMismatch);
                }
                let mut other=$session::connect(options.clone(),c.clone()).await.unwrap();
                let h=delayed.validate(&c,&op,MAX_TTL).unwrap();
                assert_eq!(other.execute_validated(&c,&op,h).await.unwrap_err(),ExecutionError::ContextMismatch);
                assert!(other.recorder.calls.is_empty());
                drop(other);
                let mut reconnected=$session::connect(options.clone(),c.clone()).await.unwrap();
                let original=$session::connect(options.clone(),c.clone()).await.unwrap();
                let h=original.validate(&c,&op,MAX_TTL).unwrap();
                drop(original);
                assert_eq!(reconnected.execute_validated(&c,&op,h).await.unwrap_err(),ExecutionError::ContextMismatch);
                assert!(reconnected.recorder.calls.is_empty());
                let mut expired=delayed.validate(&c,&op,MAX_TTL).unwrap();
                expired.expires=Instant::now(); // test-only expired-before-entry case
                assert_eq!(delayed.execute_validated(&c,&op,expired).await.unwrap_err(),ExecutionError::Expired);
                // First gate passes in real time; a test-only finite barrier at the
                // actual final gate deterministically advances to exclusive expiry.
                for offset in [0,1] {
                    let h=delayed.validate(&c,&op,MAX_TTL).unwrap();
                    delayed.final_now=Some(h.expires+Duration::from_nanos(offset));
                    let barrier=Arc::new(tokio::sync::Notify::new());
                    delayed.final_barrier=Some(barrier.clone());
                    let mut f=Box::pin(delayed.execute_validated(&c,&op,h));
                    assert!(poll_once(f.as_mut()).await.is_pending());
                    barrier.notify_one();
                    assert_eq!(f.await.unwrap_err(),ExecutionError::Expired);
                }
                // Polled cancellation BEFORE final gate preserves session availability.
                let h=delayed.validate(&c,&op,MAX_TTL).unwrap();
                delayed.final_barrier=Some(Arc::new(tokio::sync::Notify::new()));
                let mut f=Box::pin(delayed.execute_validated(&c,&op,h));
                assert!(poll_once(f.as_mut()).await.is_pending()); drop(f);
                assert!(delayed.recorder.calls.is_empty());
                let h=delayed.validate(&c,&op,MAX_TTL).unwrap();
                delayed.final_now=Some(h.expires-Duration::from_nanos(1));
                delayed.execute_validated(&c,&op,h).await.unwrap();
                assert_eq!(delayed.recorder.calls.len(),1);
                // Exact original record is retained rather than rendered again.
                let h=delayed.validate(&c,&op,MAX_TTL).unwrap();
                let source=h.prepared.sensitive_source(); let final_source=source.final_source();
                let expected=(final_source.sql().to_owned(), final_source.args().to_vec());
                delayed.execute_validated(&c,&op,h).await.unwrap();
                assert_eq!(delayed.recorder.calls[1],expected);
                for refused in [Draft::delete(table()).freeze(),Draft::delete(table()).filter(eq(tenant(),Value::I64(11))).freeze(),select(d).filter(Predicate::Not(Box::new(filter(1)))).freeze()] {
                    assert!(delayed.validate(&c,&refused,MAX_TTL).is_err());
                }
                assert_eq!(delayed.recorder.calls.len(),2);
                // Dedicated lock setup is not a business dispatch. Observe the
                // server waiting before dropping/timing out the real send future.
                for timeout in [false,true] {
                    let mut locker=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                    let mut observer=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                    let mut tx=locker.begin().await.unwrap();
                    sqlx::query("UPDATE quent_dispatch_test.items SET txt=txt WHERE tenant=11 AND id=1").execute(&mut *tx).await.unwrap();
                    let mut canceled=$session::connect(options.clone(),c.clone()).await.unwrap();
                    let connection_id=canceled.test_connection_id().await;
                    let op=Draft::update(table(),write(Value::Text("canceled_attempt".into()),changed())).filter(filter(1)).freeze();
                    let h=canceled.validate(&c,&op,MAX_TTL).unwrap();
                    let mut f=Box::pin(canceled.execute_validated(&c,&op,h));
                    let waiting=if d==Dialect::Postgres {
                        "SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user AND wait_event_type='Lock' AND query LIKE 'UPDATE %'"
                    } else { "SELECT COUNT(*) FROM performance_schema.data_lock_waits w JOIN performance_schema.threads t ON t.THREAD_ID=w.REQUESTING_THREAD_ID WHERE t.PROCESSLIST_ID=?" };
                    tokio::time::timeout(Duration::from_secs(10),async {
                        loop {
                            assert!(poll_once(f.as_mut()).await.is_pending());
                            let row=sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(connection_id).fetch_one(&mut observer).await.unwrap();
                            if row.get::<i64,_>(0)>0 {break;}
                            tokio::task::yield_now().await;
                        }
                    }).await.expect("business UPDATE was not observed waiting on dedicated row lock");
                    if timeout {
                        assert!(tokio::time::timeout(Duration::from_millis(1),f.as_mut()).await.is_err());
                    }
                    drop(f);
                    assert_eq!(canceled.recorder.calls.len(),1);
                    assert_eq!(canceled.execute(&c,select(d).freeze()).await.unwrap_err(),ExecutionError::SessionUnavailable);
                    assert_eq!(canceled.recorder.calls.len(),1);
                    // Cancellation returns no Result and makes no rollback promise.
                    drop(canceled);
                    tx.rollback().await.unwrap(); // only the explicit SET txt=txt setup
                    println!("PR2 {:?}: server lock wait observed; {} consumed one dispatch; session unavailable; business write outcome unconfirmed",d,if timeout {"timeout"} else {"drop"});
                }
                // Reset only dedicated setup data; no rollback inference about cancel.
                let mut reset=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                sqlx::query("UPDATE quent_dispatch_test.items SET txt='' WHERE tenant=11 AND id=1").execute(&mut reset).await.unwrap();
                println!("PR2 {:?}: delayed exact sends=2; mismatch/expiry/unpolled/pre-send cancel sends=0",d);
                // Positive projection, signed extrema/zero/false/empty/typed NULL.
                let r = session.execute(&c,select(d).filter(filter(1)).freeze()).await.unwrap();
                assert_eq!(r.sensitive_rows().len(),1);
                let mut expected = vec![Value::I64(1),Value::I64(11),Value::I16(i16::MIN),Value::I32(i32::MAX),Value::I64(i64::MIN),Value::Bool(false),Value::Text(String::new()),Value::Null(SqlType::I64)];
                if d == Dialect::Mysql { expected.extend([Value::I8(i8::MIN),Value::U8(u8::MAX),Value::U16(u16::MAX),Value::U32(u32::MAX),Value::U64(u64::MAX)]); }
                assert_eq!(r.sensitive_rows(), &[expected]);
                assert_eq!(session.recorder.calls.len(),1);
                let expected_sql = if d == Dialect::Postgres { "SELECT \"items\".\"id\", \"items\".\"tenant\", \"items\".\"small\", \"items\".\"normal\", \"items\".\"big\", \"items\".\"flag\", \"items\".\"txt\", \"items\".\"optional\" FROM \"quent_dispatch_test\".\"items\" WHERE ((\"items\".\"id\" = $1) AND (\"items\".\"tenant\" = $2))" } else { "SELECT `items`.`id`, `items`.`tenant`, `items`.`small`, `items`.`normal`, `items`.`big`, `items`.`flag`, `items`.`txt`, `items`.`optional`, `items`.`tiny`, `items`.`u_tiny`, `items`.`u_small`, `items`.`u_normal`, `items`.`u_big` FROM `quent_dispatch_test`.`items` WHERE ((`items`.`id` = ?) AND (`items`.`tenant` = ?))" };
                assert_eq!(session.recorder.calls[0].0,expected_sql);
                assert_eq!(session.recorder.calls[0].1,vec![Value::I64(1),Value::I64(11)]);
                // Tenant isolation and outer guard on every OR arm (C06).
                let r = session.execute(&c,select(d).filter(Predicate::Or(vec![Predicate::And(vec![eq(tenant(),Value::I64(11)),filter(1)]),filter(2)])).freeze()).await.unwrap();
                assert_eq!(r.sensitive_rows().len(),1);
                let r = session.execute(&c,select(d).count().unwrap().freeze()).await.unwrap();
                assert!(matches!(r.sensitive_count(),Some(Value::I64(1))));
                let c2=context(d,22,"subject",vec![]);
                let mut b=$session::connect(options.clone(),c2.clone()).await.unwrap();
                let r=b.execute(&c2,select(d).freeze()).await.unwrap();
                assert_eq!(r.sensitive_rows()[0][1],Value::I64(22));
                // Every supported adapter binding type, independent extrema, then
                // typed NULL for every nullable native type. INSERT remains refused.
                let mut native = vec![Value::I16(i16::MAX),Value::I32(i32::MIN),Value::I64(i64::MAX),Value::Bool(true),Value::Text("native".into()),Value::I64(0)];
                if d == Dialect::Mysql { native.extend([Value::I8(i8::MAX),Value::U8(u8::MAX),Value::U16(u16::MAX),Value::U32(u32::MAX),Value::U64(u64::MAX)]); }
                let native_columns: Vec<_> = fields(d).into_iter().skip(2).collect();
                let assignments = native_columns.iter().cloned().zip(native.iter().cloned().map(Input::Value)).collect();
                session.execute(&c,Draft::update(table(),assignments).filter(filter(1)).freeze()).await.unwrap();
                let r=session.execute(&c,select(d).filter(filter(1)).freeze()).await.unwrap();
                assert_eq!(&r.sensitive_rows()[0][2..],native.as_slice());
                assert_eq!(session.recorder.calls[session.recorder.calls.len()-2].1, native.iter().cloned().chain([Value::I64(1),Value::I64(11)]).collect::<Vec<_>>());
                let zero: Vec<_> = native.iter().map(|v|match v { Value::I8(_)=>Value::I8(0),Value::I16(_)=>Value::I16(0),Value::I32(_)=>Value::I32(0),Value::I64(_)=>Value::I64(0),Value::U8(_)=>Value::U8(0),Value::U16(_)=>Value::U16(0),Value::U32(_)=>Value::U32(0),Value::U64(_)=>Value::U64(0),Value::Bool(_)=>Value::Bool(false),Value::Text(_)=>Value::Text(String::new()),Value::Null(_)=>unreachable!() }).collect();
                let assignments=native_columns.iter().cloned().zip(zero.iter().cloned().map(Input::Value)).collect();
                session.execute(&c,Draft::update(table(),assignments).filter(filter(1)).freeze()).await.unwrap();
                let r=session.execute(&c,select(d).filter(filter(1)).freeze()).await.unwrap();
                assert_eq!(&r.sensitive_rows()[0][2..],zero.as_slice());
                let nulls: Vec<_> = native.iter().map(|v|Value::Null(v.sql_type())).collect();
                let assignments=native_columns.into_iter().zip(nulls.iter().cloned().map(Input::Value)).collect();
                session.execute(&c,Draft::update(table(),assignments).filter(filter(1)).freeze()).await.unwrap();
                let r=session.execute(&c,select(d).filter(filter(1)).freeze()).await.unwrap();
                assert_eq!(&r.sensitive_rows()[0][2..],nulls.as_slice());
                // Rejections observed at the production dispatch point: zero delta.
                let before=session.recorder.calls.len();
                for op in [Draft::insert(table(),write(Value::I64(3),key())).freeze(),Draft::update(table(),write(Value::Text("canary_value".into()),changed())).freeze(),Draft::delete(table()).freeze(),Draft::delete(table()).filter(eq(tenant(),Value::I64(11))).freeze(),Draft::update(table(),write(Value::Text("x".into()),changed())).filter(eq(tenant(),Value::I64(11))).freeze(),select(d).filter(eq(changed(),Value::Text("x".into()))).freeze(),select(d).filter(Predicate::Not(Box::new(filter(1)))).freeze(),Draft::delete(table()).filter(Predicate::Compare(key(),Compare::Gt,Value::I64(0))).freeze(),Draft::delete(table()).filter(Predicate::In(key(),vec![Value::I64(1),Value::I64(2)])).freeze(),select(d).filter(eq(tenant(),Value::I64(22))).freeze(),Draft::update(table(),write(Value::I64(11),tenant())).filter(filter(1)).freeze()] {
                    assert!(session.execute(&c,op).await.is_err());
                }
                let aliased=Table::new(vec![id("quent_dispatch_test"),id("items")],Some(id("alias"))).unwrap();
                let unsupported=Draft::select(aliased.clone(),vec![Projection::new(aliased.column(id("id"),SqlType::I64,false),None)]).freeze();
                assert_eq!(session.execute(&c,unsupported).await.unwrap_err(),ExecutionError::Policy(PolicyError::Unsupported));
                assert_eq!(session.execute(&c2,select(d).freeze()).await.unwrap_err(),ExecutionError::ContextMismatch);
                assert_eq!(session.recorder.calls.len(),before);
                let other_subject=context(d,11,"other_subject",vec![]);
                assert_eq!(session.execute(&other_subject,select(d).freeze()).await.unwrap_err(),ExecutionError::ContextMismatch);
                let other_dialect=context(if d==Dialect::Postgres { Dialect::Mysql } else { Dialect::Postgres },11,"subject",vec![]);
                assert_eq!(session.execute(&other_dialect,select(d).freeze()).await.unwrap_err(),ExecutionError::ContextMismatch);
                let protected=context(d,11,"subject",vec![PolicyDeclaration::Pii(changed())]);
                assert_eq!(session.execute(&protected,select(d).freeze()).await.unwrap_err(),ExecutionError::ContextMismatch);
                let mut protected_session=$session::connect(options.clone(),protected.clone()).await.unwrap();
                assert!(protected_session.execute(&protected,select(d).freeze()).await.is_err());
                assert!(protected_session.recorder.calls.is_empty());
                let readonly=context(d,11,"subject",vec![PolicyDeclaration::Readonly(changed())]);
                let mut ro=$session::connect(options.clone(),readonly.clone()).await.unwrap();
                assert_eq!(ro.execute(&readonly,Draft::update(table(),write(Value::Text("x".into()),changed())).filter(filter(1)).freeze()).await.unwrap_err(),ExecutionError::Policy(PolicyError::ProtectedWrite));
                assert!(ro.recorder.calls.is_empty());

                let b=DatabaseBinding::application_supplied(DatabaseIdentity::new("dedicated_test").unwrap(),TargetAddress::new("dedicated_loopback").unwrap(),id("quent_dispatch_test"),d);
                let next=Settings::new(b.clone(),SettingsIdentity::new("settings").unwrap(),Generation::new(2).unwrap(),PolicySet::new(b.clone(),PolicyIdentity::new("policy").unwrap(),Generation::new(2).unwrap(),vec![PolicyDeclaration::Tenant(tenant())]).unwrap()).unwrap();
                let next=c.with_settings(next).unwrap();
                assert_eq!(session.execute(&next,select(d).freeze()).await.unwrap_err(),ExecutionError::ContextMismatch);
                let no_schema=c.with_application(ApplicationContext::application_supplied(b.clone(),None,Some(SubjectIdentity::new("subject").unwrap()),Some(TenantSnapshot::application_supplied(b.clone(),tenant(),Value::I64(11)).unwrap())).unwrap()).unwrap();
                let mut no_schema_session=$session::connect(options.clone(),no_schema.clone()).await.unwrap();
                assert_eq!(no_schema_session.execute(&no_schema,select(d).freeze()).await.unwrap_err(),ExecutionError::Policy(PolicyError::MissingSchema));
                assert!(no_schema_session.recorder.calls.is_empty());
                let missing=c.with_application(ApplicationContext::application_supplied(b,None,None,None).unwrap()).unwrap();
                let mut missing_session=$session::connect(options.clone(),missing.clone()).await.unwrap();
                assert_eq!(missing_session.execute(&missing,select(d).freeze()).await.unwrap_err(),ExecutionError::MissingSubject);
                assert!(missing_session.recorder.calls.is_empty());
                assert_eq!(session.recorder.calls.len(),before);
                // NULL = NULL / NOT UNKNOWN and empty IN retain SQL semantics.
                for p in [eq(col("optional",SqlType::I64,true),Value::Null(SqlType::I64)),Predicate::Not(Box::new(eq(col("optional",SqlType::I64,true),Value::Null(SqlType::I64)))),Predicate::In(key(),vec![])] {
                    assert!(session.execute(&c,select(d).filter(p).freeze()).await.unwrap().sensitive_rows().is_empty());
                }
                // Permitted write with exact ordered native args; independent DB read.
                let r=session.execute(&c,Draft::update(table(),write(Value::Text("changed".into()),changed())).filter(filter(1)).freeze()).await.unwrap();
                let expected_affected=if d==Dialect::Postgres { AffectedRows::PostgresMatched(1) } else { AffectedRows::MysqlFoundRows(1) };
                assert_eq!(r.affected_rows(),Some(expected_affected));
                assert_eq!(session.recorder.calls.last().unwrap().1,vec![Value::Text("changed".into()),Value::I64(1),Value::I64(11)]);
                let r=session.execute(&c,Draft::update(table(),write(Value::Text("changed".into()),changed())).filter(filter(1)).freeze()).await.unwrap();
                assert_eq!(r.affected_rows(),Some(expected_affected)); // unchanged MySQL FOUND_ROWS
                let mut observer=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                let rows=sqlx::query("SELECT tenant, txt FROM quent_dispatch_test.items ORDER BY tenant").fetch_all(&mut observer).await.unwrap();
                assert_eq!(rows.len(),2); assert_eq!(rows[0].get::<String,_>(1),"changed"); assert_eq!(rows[1].get::<String,_>(1),"other");
                let mut oversized=$session::connect(options.clone(),c.clone()).await.unwrap();
                let text= "x".repeat(MAX_TEXT_BYTES+1);
                let update_sql=if d==Dialect::Postgres { "UPDATE quent_dispatch_test.items SET txt=$1 WHERE tenant=11" } else { "UPDATE quent_dispatch_test.items SET txt=? WHERE tenant=11" };
                sqlx::query(sqlx::AssertSqlSafe(update_sql.to_owned())).bind(text).execute(&mut observer).await.unwrap();
                assert_eq!(oversized.execute(&c,select(d).filter(filter(1)).freeze()).await.unwrap_err(),ExecutionError::ResultLimit);
                sqlx::query("UPDATE quent_dispatch_test.items SET txt='' WHERE tenant=11").execute(&mut observer).await.unwrap();
                if d==Dialect::Mysql {
                    let mut invalid_bool=$session::connect(options.clone(),c.clone()).await.unwrap();
                    sqlx::query("UPDATE quent_dispatch_test.items SET flag=2 WHERE tenant=11").execute(&mut observer).await.unwrap();
                    assert_eq!(invalid_bool.execute(&c,select(d).filter(filter(1)).freeze()).await.unwrap_err(),ExecutionError::Decode);
                    sqlx::query("UPDATE quent_dispatch_test.items SET flag=NULL WHERE tenant=11").execute(&mut observer).await.unwrap();
                }
                // Complete finite metadata must match actual native result types.
                // A deliberately stale application assertion is not authenticated.
                let mut wrong = $session::connect(options.clone(),c.clone()).await.unwrap();
                let alter=if d==Dialect::Postgres { "ALTER TABLE quent_dispatch_test.items ALTER COLUMN small TYPE BIGINT" } else { "ALTER TABLE quent_dispatch_test.items MODIFY small BIGINT NULL" };
                sqlx::query(sqlx::AssertSqlSafe(alter.to_owned())).execute(&mut observer).await.unwrap();
                assert_eq!(wrong.execute(&c,select(d).filter(filter(1)).freeze()).await.unwrap_err(),ExecutionError::Decode);
                assert_eq!(wrong.recorder.calls.len(),1);
                // Nonnullable NULL mismatch is refused too, using only finite
                // projection metadata (the actual nullable optional column).
                let required=col("optional",SqlType::I64,false);
                let wrong_op=Draft::select(table(),vec![Projection::new(required,None)]).filter(filter(1)).freeze();
                assert!(session.execute(&c,wrong_op).await.is_err()); // schema coverage, before send
                let mut limited=$session::connect(options.clone(),c.clone()).await.unwrap();
                // Setup bulk rows uses a dialect-specific static SQL statement.
                let setup_sql=if d==Dialect::Postgres { "INSERT INTO quent_dispatch_test.items (id,tenant) VALUES ($1,$2)" } else { "INSERT INTO quent_dispatch_test.items (id,tenant) VALUES (?,?)" };
                for n in 3..260_i64 {
                    sqlx::query(sqlx::AssertSqlSafe(setup_sql.to_owned())).bind(n).bind(11_i64).execute(&mut observer).await.unwrap();
                }
                let only_keys=Draft::select(table(),vec![Projection::new(key(),None)]).freeze();
                assert_eq!(limited.execute(&c,only_keys).await.unwrap_err(),ExecutionError::ResultLimit);
                assert_eq!(limited.recorder.calls.len(),1);
                // Allowed DELETE reports matched observation; it is not atomic API.
                let r=session.execute(&c,Draft::delete(table()).filter(filter(259)).freeze()).await.unwrap();
                assert_eq!(r.affected_rows(),Some(expected_affected));
                Box::pin(async { atomic_cases!($session, $borrowed, $connection, options, d, c, observer, $create, $seed); }).await;
                let mut failed_write=$session::connect(options.clone(),c.clone()).await.unwrap();
                // Type mismatch fails redacted; no success truncation of results.
                let bad=select(d).filter(filter(1)).freeze();
                let write_op=Draft::update(table(),write(Value::Text("x".into()),changed())).filter(filter(1)).freeze();
                let failed_handle=failed_write.validate(&c,&write_op,MAX_TTL).unwrap();
                let subsequent=failed_write.validate(&c,&write_op,MAX_TTL).unwrap();
                sqlx::query("DROP TABLE quent_dispatch_test.items").execute(&mut observer).await.unwrap();
                assert_eq!(failed_write.execute_validated(&c,&write_op,failed_handle).await.unwrap_err(),ExecutionError::OutcomeUnknown);
                assert_eq!(failed_write.execute_validated(&c,&write_op,subsequent).await.unwrap_err(),ExecutionError::SessionUnavailable);
                assert_eq!(failed_write.recorder.calls.len(),1);
                let n=session.recorder.calls.len();
                let e=session.execute(&c,bad).await.unwrap_err();
                assert_eq!(e,ExecutionError::ReadFailed);
                assert_eq!(session.recorder.calls.len(),n+1); // no automatic retry
                assert_eq!(session.execute(&c,select(d).freeze()).await.unwrap_err(),ExecutionError::SessionUnavailable);
                assert_eq!(session.recorder.calls.len(),n+1);
                assert!(!format!("{session:?} {e:?} {e}").contains("canary_value"));
                #[cfg(feature="output-tracing")] output_capture::check(&e.public_view(),&e.public_view().to_json());
                assert!(std::error::Error::source(&e).is_none());
            })));
            #[cfg(feature="output-tracing")] drop(subscriber);
            // Cleanup executes even after an assertion panic. Container wrapper also
            // removes its dedicated servers on command failure.
            runtime.block_on(async { let mut setup=<$connection>::connect_with(&options.disable_statement_logging()).await.unwrap(); sqlx::query("DROP TABLE IF EXISTS quent_dispatch_test.items").execute(&mut setup).await.unwrap(); });
            if let Err(panic) = result { std::panic::resume_unwind(panic); }
            #[cfg(feature="output-tracing")] lifetime_capture.assert_safe(0);
        }
    }
}
live_suite!(
    live_postgres,
    "postgres",
    PostgresSession,
    BorrowedPostgresTransaction,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    "CREATE TABLE quent_dispatch_test.items (id BIGINT NOT NULL, tenant BIGINT NOT NULL, small SMALLINT NULL, normal INT NULL, big BIGINT NULL, flag BOOLEAN NULL, txt TEXT NULL, optional BIGINT NULL, PRIMARY KEY(tenant,id))",
    "INSERT INTO quent_dispatch_test.items VALUES (1,11,-32768,2147483647,-9223372036854775808,false,'',NULL),(2,22,0,0,0,true,'other',0)"
);
live_suite!(
    live_mysql,
    "mysql",
    MysqlSession,
    BorrowedMysqlTransaction,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "CREATE TABLE quent_dispatch_test.items (id BIGINT NOT NULL, tenant BIGINT NOT NULL, small SMALLINT NULL, normal INT NULL, big BIGINT NULL, flag BOOLEAN NULL, txt TEXT NULL, optional BIGINT NULL, tiny TINYINT NULL, u_tiny TINYINT UNSIGNED NULL, u_small SMALLINT UNSIGNED NULL, u_normal INT UNSIGNED NULL, u_big BIGINT UNSIGNED NULL, PRIMARY KEY(tenant,id))",
    "INSERT INTO quent_dispatch_test.items VALUES (1,11,-32768,2147483647,-9223372036854775808,false,'',NULL,-128,255,65535,4294967295,18446744073709551615),(2,22,0,0,0,true,'other',0,0,0,0,0,0)"
);

#[test]
fn redacted_results_errors_and_limits() {
    let r = ExecutionResult {
        state: ResultState::Observed,
        rows: vec![vec![Value::Text(
            "CANARY_SQL_identifier_reason_value".into(),
        )]],
        count: None,
        affected: None,
    };
    assert_eq!(format!("{r:?}"), "ExecutionResult([REDACTED])");
    for state in [
        ResultState::Observed,
        ResultState::Uncommitted,
        ResultState::Committed,
    ] {
        let r = ExecutionResult {
            state,
            rows: vec![vec![Value::Text("CANARY".into())]],
            count: None,
            affected: None,
        };
        assert!(!format!("{r:?}").contains("CANARY"));
    }
    assert_eq!(
        ExpectedRows::exactly(257).unwrap_err(),
        ExecutionError::Unsupported
    );
    for disposition in [
        AtomicDisposition::NotStarted,
        AtomicDisposition::RolledBack,
        AtomicDisposition::OutcomeUnknown,
    ] {
        let e = AtomicFailure {
            cause: ExecutionError::Decode,
            disposition,
        };
        #[cfg(feature = "output-tracing")]
        output_capture::check(&e.public_view(), &e.public_view().to_json());
        assert!(std::error::Error::source(&e).is_none());
        assert!(!format!("{e:?} {e}").contains("CANARY"));
    }
    for e in [
        ExecutionError::Connect,
        ExecutionError::ReadFailed,
        ExecutionError::Decode,
        ExecutionError::OutcomeUnknown,
    ] {
        #[cfg(feature = "output-tracing")]
        output_capture::check(&e.public_view(), &e.public_view().to_json());
        assert!(std::error::Error::source(&e).is_none());
        assert!(!format!("{e} {e:?}").contains("CANARY"));
    }
    let mut used = MAX_RESULT_BYTES;
    assert_eq!(
        check_result_size(r.sensitive_rows()[0].as_slice(), &mut used).unwrap_err(),
        ExecutionError::ResultLimit
    );
}

#[test]
fn expiry_and_bounded_original_input_are_pure_and_exact() {
    let now = Instant::now();
    for ttl in [
        Duration::ZERO,
        MAX_TTL + Duration::from_nanos(1),
        Duration::MAX,
    ] {
        assert_eq!(deadline(now, ttl), Err(ExecutionError::InvalidTtl));
    }
    assert_eq!(deadline(now, MAX_TTL).unwrap(), now + MAX_TTL);
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, 11, "subject", vec![]);
        let op = select(d).filter(filter(1)).freeze();
        let identity = Arc::new(());
        let h = ValidatedOperation {
            prepared: prepare(&c, &c, &op, d).unwrap(),
            session: identity.clone(),
            expires: now,
        };
        assert!(check_expiry(&h, now - Duration::from_nanos(1)).is_ok());
        assert_eq!(check_expiry(&h, now), Err(ExecutionError::Expired));
        assert_eq!(
            check_expiry(&h, now + Duration::from_nanos(1)),
            Err(ExecutionError::Expired)
        );
        assert!(recheck(&h, &identity, &c, &c, Some(&op)).is_ok());
        let gigantic = select(d)
            .filter(Predicate::And(vec![filter(1); 1025]))
            .freeze();
        assert_eq!(
            recheck(&h, &identity, &c, &c, Some(&gigantic)),
            Err(ExecutionError::OperationMismatch)
        );
        let write_op = Draft::update(
            table(),
            vec![
                (changed(), Input::Value(Value::Text("secret".into()))),
                (
                    col("normal", SqlType::I32, true),
                    Input::Value(Value::I32(1)),
                ),
            ],
        )
        .filter(filter(1))
        .freeze();
        let write_h = ValidatedOperation {
            prepared: prepare(&c, &c, &write_op, d).unwrap(),
            session: identity.clone(),
            expires: now,
        };
        for inputs in [
            vec![
                (
                    col("normal", SqlType::I32, true),
                    Input::Value(Value::I32(1)),
                ),
                (changed(), Input::Value(Value::Text("secret".into()))),
            ],
            vec![(changed(), Input::Missing)],
            vec![(changed(), Input::Value(Value::Null(SqlType::Text)))],
        ] {
            let different = Draft::update(table(), inputs).filter(filter(1)).freeze();
            assert_eq!(
                recheck(&write_h, &identity, &c, &c, Some(&different)),
                Err(ExecutionError::OperationMismatch)
            );
        }
    }
}

#[test]
fn public_result_and_atomic_failure_preserve_states_without_values() {
    use serde_json::{Value as Json, json};
    for (state, wire) in [
        (ResultState::Observed, "observed"),
        (ResultState::Uncommitted, "uncommitted"),
        (ResultState::Committed, "committed"),
    ] {
        let r = ExecutionResult {
            state,
            rows: vec![vec![
                Value::Text("CANARY_SQL_alias_tenant_subject_reason_source_/tmp/path".into()),
                Value::Null(SqlType::Text),
            ]],
            count: Some(Value::I64(987654321)),
            affected: Some(AffectedRows::MysqlFoundRows(987654321)),
        };
        let v = r.public_view();
        #[cfg(feature = "output-tracing")]
        output_capture::check(&v, &v.to_json());
        assert_eq!(
            serde_json::from_str::<Json>(&v.to_json()).unwrap(),
            json!({"kind":"quent_result_view","version":1,"state":wire})
        );
        assert_eq!(v.to_json(), v.to_string());
        assert!(!format!("{v:?} {v}").contains("CANARY"));
    }
    for (disposition, wire) in [
        (AtomicDisposition::NotStarted, "not_started"),
        (AtomicDisposition::RolledBack, "rolled_back"),
        (AtomicDisposition::OutcomeUnknown, "outcome_unknown"),
    ] {
        let e = AtomicFailure {
            cause: ExecutionError::Conflict,
            disposition,
        };
        let v = e.public_view();
        #[cfg(feature = "output-tracing")]
        output_capture::check(&v, &v.to_json());
        assert_eq!(
            serde_json::from_str::<Json>(&v.to_json()).unwrap(),
            json!({"kind":"quent_diagnostic_view","version":1,"code":"execution_conflict","state":wire})
        );
        assert!(std::error::Error::source(&v).is_none());
        assert_eq!(v.to_json(), v.to_string());
    }
}

#[test]
fn complete_execution_diagnostic_register_matches_wire_codes() {
    let mappings = [
        (
            ExecutionError::ContextMismatch,
            "execution_context_mismatch",
        ),
        (
            ExecutionError::OperationMismatch,
            "execution_operation_mismatch",
        ),
        (ExecutionError::InvalidTtl, "execution_invalid_ttl"),
        (ExecutionError::Expired, "execution_expired"),
        (ExecutionError::MissingSubject, "execution_missing_subject"),
        (ExecutionError::RiskRefused, "execution_risk_refused"),
        (ExecutionError::Incomplete, "execution_incomplete"),
        (ExecutionError::Unsupported, "execution_unsupported"),
        (ExecutionError::Encode, "execution_encode"),
        (ExecutionError::Connect, "execution_connect"),
        (ExecutionError::ReadFailed, "execution_read_failed"),
        (ExecutionError::Decode, "execution_decode"),
        (ExecutionError::ResultLimit, "execution_result_limit"),
        (ExecutionError::OutcomeUnknown, "execution_outcome_unknown"),
        (
            ExecutionError::SessionUnavailable,
            "execution_session_unavailable",
        ),
        (
            ExecutionError::ExpectationMismatch,
            "execution_expectation_mismatch",
        ),
        (ExecutionError::Conflict, "execution_conflict"),
    ];
    for (e, wire) in mappings {
        let v = e.public_view();
        #[cfg(feature = "output-tracing")]
        output_capture::check(&v, &v.to_json());
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&v.to_json()).unwrap(),
            serde_json::json!({"kind":"quent_diagnostic_view","version":1,"code":wire,"state":null})
        );
        assert!(std::error::Error::source(&v).is_none());
    }
    assert_eq!(
        ExecutionError::Policy(PolicyError::InvalidPlan(PlanError::Missing)).public_view(),
        PlanError::Missing.public_view()
    );
}

#[cfg(all(feature = "local-details", target_os = "linux"))]
fn export_does_not_change_gate(prep: &PolicyPreparation) {
    use crate::local_details::*;
    use std::os::unix::fs::DirBuilderExt;
    let dir = std::env::temp_dir().join(format!(
        "quent-gate-export-{}-{:?}",
        std::process::id(),
        prep.sensitive_source().final_source().dialect()
    ));
    std::fs::DirBuilder::new().mode(0o700).create(&dir).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }
    let _cleanup = Cleanup(dir.clone());
    let path = dir.join("new");
    let before = prep.public_view().to_json();
    let r = create_new(
        LocalOptIn::acknowledge_sensitive_local_file(),
        Surface::LocalApplication,
        &path,
        prep.sensitive_source().final_source(),
    );
    if std::env::var_os("CI").is_some() || std::env::var_os("GITHUB_ACTIONS").is_some() {
        assert_eq!(r.unwrap_err(), DetailError::EnvironmentDenied);
        assert!(!path.exists());
    } else {
        r.unwrap();
        let j: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert!(j["sql"] == prep.sensitive_source().final_source().sql());
    }
    assert_eq!(before, prep.public_view().to_json());
}

#[cfg(feature = "codegen")]
mod generated;
