use super::*;
use crate::authorization::{
    AdapterResponse, AuthorizationDecision, AuthorizationRequest, TrustedAuthorizationAdapter,
};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

type ObservedRecord = (usize, String, Vec<Value>, Metadata);
#[derive(Default)]
pub(super) struct Probe {
    calls: AtomicUsize,
    responses: AtomicUsize,
    records: Mutex<Vec<ObservedRecord>>,
    pub(super) response: Mutex<Option<AdapterResponse>>,
    pub(super) barrier: Mutex<Option<Arc<tokio::sync::Notify>>>,
    saved: Mutex<Option<AuthorizationDecision>>,
    pub(super) save_next: std::sync::atomic::AtomicBool,
}
impl TrustedAuthorizationAdapter for Arc<Probe> {
    fn authorize<'a>(
        &'a self,
        request: AuthorizationRequest<'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = AuthorizationDecision> + Send + 'a>>
    {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(request.is_runtime_bound());
        let source = request.sensitive_preparation().sensitive_source();
        let record = source.final_source();
        self.records.lock().unwrap().push((
            record.sql().as_ptr() as usize,
            record.sql().to_owned(),
            record.args().to_vec(),
            record.metadata().clone(),
        ));
        let barrier = self.barrier.lock().unwrap().take();
        Box::pin(async move {
            if let Some(barrier) = barrier {
                barrier.notified().await;
            }
            if let Some(saved) = self.saved.lock().unwrap().take() {
                return saved;
            }
            let response = self
                .response
                .lock()
                .unwrap()
                .take()
                .unwrap_or(AdapterResponse::AllowCandidate);
            self.responses.fetch_add(1, Ordering::SeqCst);
            if self.save_next.swap(false, Ordering::SeqCst) {
                *self.saved.lock().unwrap() = Some(request.respond(response));
                std::future::pending().await
            } else {
                request.respond(response)
            }
        })
    }
}
impl Probe {
    pub(super) fn responses(&self) -> usize {
        self.responses.load(Ordering::SeqCst)
    }
    pub(super) fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
    pub(super) fn compare(&self, recorder: &Recorder, index: usize) {
        let records = self.records.lock().unwrap();
        let (address, sql, args, metadata) = &records[index];
        assert_eq!(&recorder.sql_addresses, &[*address]);
        assert_eq!(recorder.calls, vec![(sql.clone(), args.clone())]);
        assert!(
            recorder.metadata[0] == *metadata,
            "all final metadata identical"
        );
    }
}
macro_rules! authorized_cases {
    ($session:ident, $borrowed:ident, $connection:ty, $options:ident, $d:ident, $c:ident, $observer:ident, $create:literal, $seed:literal, $ssl:expr) => {{
        use $crate::execution::tests::authorized::Probe;
        use $crate::authorization::{AuthorizationScope::*, AdapterResponse::*};
        use std::sync::atomic::Ordering;
        let options=&$options;let c=&$c;let d=$d;let observer=&mut $observer;
        sqlx::query("DROP TABLE quent_dispatch_test.items").execute(&mut *observer).await.unwrap();
        sqlx::query($create).execute(&mut *observer).await.unwrap();
        sqlx::query($seed).execute(&mut *observer).await.unwrap();
        let probe=Arc::new(Probe::default());
        let auth=AuthorizationContext::register(c.clone(),"CANARY_provider".into(),1,b"CANARY_credential".to_vec(),probe.clone()).unwrap();
        let request=AuthorizationExecutionRequest::new();
        let other_request=AuthorizationExecutionRequest::new();
        let op=Draft::update(table(),write(Value::Text("CANARY_AUTHORIZED".into()),changed())).filter(eq(tenant(),Value::I64(11))).freeze();
        let delete=Draft::delete(table()).filter(eq(tenant(),Value::I64(11))).freeze();
        let mut s=$session::connect(options.clone(),c.clone()).await.unwrap();
        let other=$session::connect(options.clone(),c.clone()).await.unwrap();
        assert_eq!(s.validate(c,&op,MAX_TTL).unwrap_err(),ExecutionError::RiskRefused);
        assert_eq!(s.execute(c,op.clone()).await.unwrap_err(),ExecutionError::RiskRefused);
        let missing=AuthorizationContext::without_provider(c.clone());
        assert_eq!(s.authorization_candidate(&missing,&request,&op,MultiRowUpdate,MAX_TTL).unwrap_err(),ExecutionError::Authorization(AuthorizationError::MissingProvider));
        for bad in [Draft::update(table(),write(Value::Text("x".into()),changed())).freeze(),Draft::delete(table()).freeze(),Draft::update(table(),vec![(tenant(),Input::Value(Value::I64(11)))]).filter(filter(1)).freeze(),select(d).freeze()] {
            assert!(s.authorization_candidate(&auth,&request,&bad,MultiRowUpdate,MAX_TTL).is_err());
        }
        for bad in [
            Draft::insert(table(),write(Value::I64(3),key())).freeze(),
            Draft::update(table(),write(Value::U64(3),col("big",SqlType::U64,true))).filter(eq(tenant(),Value::I64(11))).freeze(),
            Draft::update(table(),write(Value::Null(SqlType::I64),key())).filter(eq(tenant(),Value::I64(11))).freeze(),
            Draft::update(table(),write(Value::Text("x".into()),changed())).filter(eq(tenant(),Value::I64(22))).freeze(),
            Draft::update(table(),write(Value::Text("x".into()),changed())).filter(eq(changed(),Value::Text("x".into()))).freeze(),
            Draft::update(table(),write(Value::Text("x".into()),changed())).filter(eq(tenant(),Value::I64(11))).optimistic_version(key(),Value::I64(1)).unwrap().freeze(),
        ] { assert!(s.authorization_candidate(&auth,&request,&bad,MultiRowUpdate,MAX_TTL).is_err()); }
        for policy in [PolicyDeclaration::Readonly(changed()),PolicyDeclaration::Generated(changed()),PolicyDeclaration::Pii(changed()),PolicyDeclaration::RequiredFilter(key())] {
            let protected=context(d,11,"subject",vec![policy]);let a=AuthorizationContext::register(protected.clone(),"CANARY_provider".into(),1,b"CANARY_credential".to_vec(),probe.clone()).unwrap();
            let protected=$session::connect(options.clone(),protected).await.unwrap();
            assert!(protected.authorization_candidate(&a,&request,&op,MultiRowUpdate,MAX_TTL).is_err());assert!(protected.recorder.calls.is_empty());
        }
        let missing_column=context_schema(d,11,"subject",vec![],0,fields(d).into_iter().filter(|c|c!=&changed()).collect());
        let incomplete=$session::connect(options.clone(),missing_column.clone()).await.unwrap();let incomplete_auth=AuthorizationContext::register(missing_column,"CANARY_provider".into(),1,b"CANARY_credential".to_vec(),probe.clone()).unwrap();
        assert!(incomplete.authorization_candidate(&incomplete_auth,&request,&op,MultiRowUpdate,MAX_TTL).is_err());assert!(incomplete.recorder.calls.is_empty());
        assert!(s.authorization_candidate(&auth,&request,&op,MultiRowDelete,MAX_TTL).is_err());
        assert_eq!(s.authorization_candidate(&auth,&AuthorizationExecutionRequest::requiring_live_freshness(),&op,MultiRowUpdate,MAX_TTL).unwrap_err(),ExecutionError::Unsupported);
        assert_eq!(probe.calls(),0);assert!(s.recorder.calls.is_empty());
        // Cross-session/request acquisition refuses BEFORE provider invocation.
        for wrong in [false,true] {
            let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
            let e=if wrong {other.acquire_authorized(&auth,&request,h).await.unwrap_err()}else{s.acquire_authorized(&auth,&other_request,h).await.unwrap_err()};
            assert_eq!(e,ExecutionError::Authorization(AuthorizationError::Mismatch));
        }
        assert_eq!(probe.calls(),0);
        for (response,error) in [(Deny,AuthorizationError::Denied),(Error,AuthorizationError::ProviderError),(Cancelled,AuthorizationError::Cancelled),(Timeout,AuthorizationError::Timeout)] {
            *probe.response.lock().unwrap()=Some(response);
            let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
            assert_eq!(s.acquire_authorized(&auth,&request,h).await.unwrap_err(),ExecutionError::Authorization(error));
            assert!(s.recorder.calls.is_empty());assert!(s.recorder.events.is_empty());
        }
        // Same-generation complete-content changes independently supplied after acquisition.
        for variant in 0..10 {
            let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
            let h=s.acquire_authorized(&auth,&request,h).await.unwrap();
            let changed=context_variant(d,11,if variant==0 {"other_subject"} else {"subject"},vec![],variant);
            let changed=if variant==1 || variant==2 {AuthorizationContext::register(changed,"CANARY_provider".into(),1,b"CANARY_credential".to_vec(),probe.clone()).unwrap()}else{auth.with_database(changed).unwrap()};
            assert!(s.execute_authorized(&changed,&request,&op,h).await.is_err());
            assert!(s.recorder.calls.is_empty());
        }
        for mode in 0..4 {
            let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
            let h=s.acquire_authorized(&auth,&request,h).await.unwrap();
            let e=match mode {
                0=>s.execute_authorized(&auth,&other_request,&op,h).await.unwrap_err(),
                1=>{let mut other=$session::connect(options.clone(),c.clone()).await.unwrap(); let e=other.execute_authorized(&auth,&request,&op,h).await.unwrap_err();assert!(other.recorder.calls.is_empty());e},
                2=>{let changed=Draft::update(table(),write(Value::Text("other_value".into()),changed())).filter(eq(tenant(),Value::I64(11))).freeze();s.execute_authorized(&auth,&request,&changed,h).await.unwrap_err()},
                _=>{let other=AuthorizationContext::register(c.clone(),"CANARY_provider".into(),1,b"CANARY_credential".to_vec(),probe.clone()).unwrap();s.execute_authorized(&other,&request,&op,h).await.unwrap_err()},
            };
            assert!(matches!(e,ExecutionError::Authorization(AuthorizationError::Mismatch)|ExecutionError::ContextMismatch|ExecutionError::OperationMismatch));assert!(s.recorder.calls.is_empty());
        }
        // Independently supplied same-generation content, not just generation labels.
        for changed in [context(d,22,"subject",vec![]),context(d,11,"subject",vec![PolicyDeclaration::Readonly(changed())])] {
            let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();
            let changed=auth.with_database(changed).unwrap();
            assert!(s.execute_authorized(&changed,&request,&op,h).await.is_err());assert!(s.recorder.calls.is_empty());
        }
        // Provider genuinely suspended and resumed; current is supplied separately at dispatch.
        let barrier=Arc::new(tokio::sync::Notify::new());*probe.barrier.lock().unwrap()=Some(barrier.clone());
        let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
        let mut f=Box::pin(s.acquire_authorized(&auth,&request,h));assert!(poll_once(f.as_mut()).await.is_pending());
        let after_await_current=auth.with_database(context(d,11,"changed_after_await",vec![])).unwrap();
        barrier.notify_one();let h=f.await.unwrap();
        assert!(s.execute_authorized(&after_await_current,&request,&op,h).await.is_err());assert!(s.recorder.calls.is_empty());
        // Saved provider decision is not reusable on a different request/invocation.
        probe.save_next.store(true,Ordering::SeqCst);
        let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
        let mut f=Box::pin(s.acquire_authorized(&auth,&request,h));assert!(poll_once(f.as_mut()).await.is_pending());drop(f);
        let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
        assert_eq!(s.acquire_authorized(&auth,&request,h).await.unwrap_err(),ExecutionError::Authorization(AuthorizationError::Mismatch));
        // Unpolled acquisition/dispatch and actual runtime timeout of pending provider.
        let calls=probe.calls();let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();drop(s.acquire_authorized(&auth,&request,h));assert_eq!(probe.calls(),calls);
        *probe.barrier.lock().unwrap()=Some(Arc::new(tokio::sync::Notify::new()));
        let responses=probe.responses.load(Ordering::SeqCst);
        let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
        assert!(tokio::time::timeout(Duration::from_millis(1),s.acquire_authorized(&auth,&request,h)).await.is_err());assert_eq!(probe.calls(),calls+1);assert_eq!(probe.responses.load(Ordering::SeqCst),responses);assert!(s.recorder.calls.is_empty());
        let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();drop(s.execute_authorized(&auth,&request,&op,h));assert!(s.recorder.calls.is_empty());
        // Actual provider await expiry: no injected product clock and no business send.
        let barrier=Arc::new(tokio::sync::Notify::new());*probe.barrier.lock().unwrap()=Some(barrier.clone());
        let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,Duration::from_millis(100)).unwrap();#[cfg(feature="review")] let _=h.review();
        let mut f=Box::pin(s.acquire_authorized(&auth,&request,h));assert!(poll_once(f.as_mut()).await.is_pending());
        tokio::time::sleep(Duration::from_millis(150)).await;barrier.notify_one();
        assert_eq!(f.await.unwrap_err(),ExecutionError::Authorization(AuthorizationError::Expired));assert!(s.recorder.calls.is_empty());
        // Real elapsed time after a controlled final boundary await, including begin.
        for atomic in [false,true] {
            let mut delayed=$session::connect(options.clone(),c.clone()).await.unwrap();
            let h=delayed.authorization_candidate(&auth,&request,&op,MultiRowUpdate,Duration::from_millis(500)).unwrap();#[cfg(feature="review")] let _=h.review();let h=delayed.acquire_authorized(&auth,&request,h).await.unwrap();
            let barrier=Arc::new(tokio::sync::Notify::new());delayed.final_barrier=Some(barrier.clone());
            if atomic {
                let trace=delayed.recorder.live.clone();let mut f=Box::pin(delayed.execute_authorized_atomic(&auth,&request,&op,ExpectedRows::exactly(1).unwrap(),h));
                tokio::time::timeout(Duration::from_secs(5),async {loop {assert!(poll_once(f.as_mut()).await.is_pending());if trace.lock().unwrap().contains(&"begun"){break;}tokio::task::yield_now().await;}}).await.unwrap();
                tokio::time::sleep(Duration::from_millis(550)).await;barrier.notify_one();let e=f.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);
            } else {
                let mut f=Box::pin(delayed.execute_authorized(&auth,&request,&op,h));assert!(poll_once(f.as_mut()).await.is_pending());tokio::time::sleep(Duration::from_millis(550)).await;barrier.notify_one();assert_eq!(f.await.unwrap_err(),ExecutionError::Expired);
            }
            assert!(delayed.recorder.calls.is_empty());
        }
        // Final gate uses the private production checker; equality is expired.
        for atomic in [false,true] {
            let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();
            s.final_now=Some(h.operation.expires);
            if atomic {let e=s.execute_authorized_atomic(&auth,&request,&op,ExpectedRows::exactly(1).unwrap(),h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(s.recorder.events,["begin","begun","expectation","rollback","rolled_back"]);}else{assert_eq!(s.execute_authorized(&auth,&request,&op,h).await.unwrap_err(),ExecutionError::Expired);}
            assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        }
        // Real expiry at acquisition entry refuses before an allow-capable adapter.
        let calls=probe.calls();let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,Duration::from_millis(1)).unwrap();#[cfg(feature="review")] let _=h.review();
        tokio::time::sleep(Duration::from_millis(3)).await;
        assert_eq!(s.acquire_authorized(&auth,&request,h).await.unwrap_err(),ExecutionError::Authorization(AuthorizationError::Expired));assert_eq!(probe.calls(),calls);
        // Public carrier/error summaries never traverse provider inputs or credentials.
        for response in [Deny,Error] {
            *probe.response.lock().unwrap()=Some(response);
            let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();
            assert_eq!(format!("{h:?}"),"ExecutableAuthorizationCandidate([REDACTED])");
            let e=s.acquire_authorized(&auth,&request,h).await.unwrap_err();
            assert!(std::error::Error::source(&e).is_none());
            let text=format!("{e:?} {e} {}",e.public_view().to_json());
            for secret in ["CANARY","provider","subject","quent_dispatch_test"] {assert!(!text.contains(secret));}
            #[cfg(feature="output-tracing")] output_capture::check(&e.public_view(),&e.public_view().to_json());
        }
        // Successful direct update and owned commit/mismatch rollback inspect/send same allocation.
        for mode in 0..3 {
            let op=if mode==2 {Draft::update(table(),write(Value::Text("CANARY_rollback_must_not_commit".into()),changed())).filter(eq(tenant(),Value::I64(11))).freeze()}else{op.clone()};
            let responses=probe.responses.load(Ordering::SeqCst);
            let index=probe.calls();let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();
            if mode==0 {assert_eq!(s.execute_authorized(&auth,&request,&op,h).await.unwrap().state(),ResultState::Observed);}
            else if mode==1 {assert_eq!(s.execute_authorized_atomic(&auth,&request,&op,ExpectedRows::exactly(1).unwrap(),h).await.unwrap().state(),ResultState::Committed);assert_eq!(s.recorder.events,["begin","begun","send","decoded","expectation","commit","committed"]);}
            else {let e=s.execute_authorized_atomic(&auth,&request,&op,ExpectedRows::exactly(2).unwrap(),h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::ExpectationMismatch);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);}
            assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"CANARY_AUTHORIZED");
            assert_eq!(probe.calls(),index+1);assert_eq!(probe.responses.load(Ordering::SeqCst),responses+1);
            probe.compare(&s.recorder,index);s.recorder=Recorder::default();
        }
        let finite=Draft::update(table(),write(Value::Text("CANARY_AUTHORIZED".into()),changed())).filter(Predicate::Or(vec![Predicate::And(vec![eq(tenant(),Value::I64(11)),filter(1)]),Predicate::And(vec![eq(tenant(),Value::I64(11)),filter(2)])])).freeze();
        assert_eq!(c.prepare().prepare_operation(&finite).unwrap().inspect().scope(),Scope::FiniteBound(2));
        let index=probe.calls();let h=s.authorization_candidate(&auth,&request,&finite,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();s.execute_authorized(&auth,&request,&finite,h).await.unwrap();probe.compare(&s.recorder,index);s.recorder=Recorder::default();
        // Native ordered arguments and typed NULL retain the very provider-inspected record.
        let native=Draft::update(table(),vec![(col("small",SqlType::I16,true),Input::Value(Value::I16(i16::MIN))),(col("optional",SqlType::I64,true),Input::Value(Value::Null(SqlType::I64)))]).filter(eq(tenant(),Value::I64(11))).freeze();
        let index=probe.calls();let h=s.authorization_candidate(&auth,&request,&native,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();
        assert_eq!(format!("{h:?}"),"AuthorizedOperation([REDACTED])");
        s.execute_authorized(&auth,&request,&native,h).await.unwrap();probe.compare(&s.recorder,index);s.recorder=Recorder::default();
        // Observe a real server lock wait before drop/timeout/disconnect, for BOTH
        // direct and owned authorized paths. These are post-send unknown outcomes,
        // never reported as a confirmed rollback merely because a future was dropped.
        for atomic in [false,true] {for mode in 0..3 {
            sqlx::query("UPDATE quent_dispatch_test.items SET txt='before' WHERE tenant=11 AND id=1").execute(&mut *observer).await.unwrap();
            let mut locker=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut lock=locker.begin().await.unwrap();
            sqlx::query("UPDATE quent_dispatch_test.items SET txt=txt WHERE tenant=11 AND id=1").execute(&mut *lock).await.unwrap();
            let mut canceled=$session::connect(options.clone(),c.clone()).await.unwrap();let connection_id=canceled.test_connection_id().await;
            let index=probe.calls();let h=canceled.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=canceled.acquire_authorized(&auth,&request,h).await.unwrap();
            let trace=canceled.recorder.live.clone();
            let mut f=Box::pin(async {if atomic {canceled.execute_authorized_atomic(&auth,&request,&op,ExpectedRows::exactly(1).unwrap(),h).await.map_err(|e|{assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown);e.cause()})}else{canceled.execute_authorized(&auth,&request,&op,h).await}});
            let waiting=if d==Dialect::Postgres {"SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user AND wait_event_type='Lock' AND query LIKE 'UPDATE %'"} else {"SELECT COUNT(*) FROM performance_schema.data_lock_waits w JOIN performance_schema.threads t ON t.THREAD_ID=w.REQUESTING_THREAD_ID WHERE t.PROCESSLIST_ID=?"};
            tokio::time::timeout(Duration::from_secs(10),async {loop {assert!(poll_once(f.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(connection_id).fetch_one(&mut *observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.expect("authorized business UPDATE not observed in server lock wait");
            assert_eq!(probe.calls(),index+1);assert_eq!(trace.lock().unwrap().iter().filter(|e|**e=="send").count(),1);
            if mode==1 {assert!(tokio::time::timeout(Duration::from_millis(1),f.as_mut()).await.is_err());}
            if mode==2 {
                if d==Dialect::Postgres {sqlx::query("SELECT pg_terminate_backend(CAST($1 AS INT))").bind(connection_id).execute(&mut *observer).await.unwrap();}else{sqlx::query(sqlx::AssertSqlSafe(format!("KILL CONNECTION {connection_id}"))).execute(&mut *observer).await.unwrap();}
                assert_eq!(tokio::time::timeout(Duration::from_secs(10),f.as_mut()).await.unwrap().unwrap_err(),ExecutionError::OutcomeUnknown);
            }
            drop(f);assert!(!canceled.is_available());assert!(!trace.lock().unwrap().contains(&"commit"));assert!(!trace.lock().unwrap().contains(&"rolled_back"));
            // For a dropped owned future the runner recorder is also dropped;
            // the shared live trace above retains the actual business send count.
            if !atomic {probe.compare(&canceled.recorder,index);}
            // Socket drop is not server termination. Explicitly terminate only this
            // fixture connection while the locker still holds the row, then observe
            // backend absence independently before allowing the UPDATE to resume.
            if mode!=2 {
                if d==Dialect::Postgres {sqlx::query("SELECT pg_terminate_backend(CAST($1 AS INT))").bind(connection_id).execute(&mut *observer).await.unwrap();}else{sqlx::query(sqlx::AssertSqlSafe(format!("KILL CONNECTION {connection_id}"))).execute(&mut *observer).await.unwrap();}
            }
            drop(canceled);
            let present=if d==Dialect::Postgres {"SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user"}else{"SELECT COUNT(*) FROM performance_schema.threads WHERE PROCESSLIST_ID=?"};
            tokio::time::timeout(Duration::from_secs(10),async {loop {if sqlx::query(sqlx::AssertSqlSafe(present.to_owned())).bind(connection_id).fetch_one(&mut *observer).await.unwrap().get::<i64,_>(0)==0 {break;}tokio::task::yield_now().await;}}).await.expect("fixture backend did not terminate before locker release");
            lock.rollback().await.unwrap();
            assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"before");
            println!("MAINT-CANCEL {:?}: wait -> cancel atomic={atomic} mode={mode} -> fixture termination -> backend absent -> locker ACKrollback; provider1/send1; available=false; commit0/rollbackACK0; observer=before; product outcome unknown",d);
        }}
        // Ordinary UPDATE completion boundary, distinct from native COMMIT.
        // Both controls run through SQLx extended/prepared execution. No injected
        // product error seam, server setting change, or automatic resend is used.
        for hold in [false,true] {
            use super::authorized_network::Proxy;
            sqlx::query("UPDATE quent_dispatch_test.items SET txt='before' WHERE tenant=11 AND id=1").execute(&mut *observer).await.unwrap();
            let target=std::net::SocketAddr::new(options.get_host().parse().unwrap(),options.get_port());
            let proxy=Proxy::new(target,d==Dialect::Postgres,hold);
            let plain=options.clone().host("127.0.0.1").port(proxy.address.port()).ssl_mode($ssl);
            let mut network=$session::connect(plain,c.clone()).await.unwrap();
            let index=probe.calls();let h=network.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=network.acquire_authorized(&auth,&request,h).await.unwrap();
            let trace=network.recorder.live.clone();
            let mut f=Box::pin(network.execute_authorized(&auth,&request,&op,h));
            if hold {
                tokio::time::timeout(Duration::from_secs(10),async {loop {
                    assert!(poll_once(f.as_mut()).await.is_pending());
                    let updated=sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut *observer).await.unwrap().get::<String,_>(0)=="CANARY_AUTHORIZED";
                    if updated && proxy.reply.load(Ordering::SeqCst) {break;}
                    tokio::task::yield_now().await;
                }}).await.expect("ordinary UPDATE applied/withheld response not independently observed");
                assert!(!trace.lock().unwrap().contains(&"decoded"));
                assert!(tokio::time::timeout(Duration::from_millis(1),f.as_mut()).await.is_err());
                drop(f);assert!(!network.is_available());
            } else {
                assert_eq!(tokio::time::timeout(Duration::from_secs(10),f.as_mut()).await.unwrap().unwrap().state(),ResultState::Observed);
                drop(f);assert!(network.is_available());assert!(trace.lock().unwrap().contains(&"decoded"));
            }
            assert_eq!(probe.calls(),index+1);probe.compare(&network.recorder,index);
            assert_eq!(proxy.sent.load(Ordering::SeqCst),1);assert!(proxy.reply.load(Ordering::SeqCst));assert_eq!(proxy.withheld.load(Ordering::SeqCst),hold);
            assert!(!trace.lock().unwrap().contains(&"commit"));assert!(!trace.lock().unwrap().contains(&"rolled_back"));
            assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut *observer).await.unwrap().get::<String,_>(0),"CANARY_AUTHORIZED");
            drop(network);drop(proxy);
            println!("MAINT-UPDATE {:?}: ordinary prepared execute1/server completion1; hold={hold}; provider1/send1; Tx0; observer=updated; client completion={}; no retry",d,!hold);
        }
        // Borrowed Tx has a fresh identity, uncommitted output, no completion ownership.
        let mut raw=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
        let mut tx=raw.begin().await.unwrap();let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();
        *probe.response.lock().unwrap()=Some(Deny);let calls=probe.calls();let rejected=borrowed.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();
        assert_eq!(borrowed.acquire_authorized(&auth,&request,rejected).await.unwrap_err(),ExecutionError::Authorization(AuthorizationError::Denied));assert_eq!(probe.calls(),calls+1);assert!(borrowed.recorder.events.is_empty());
        let h=s.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();assert!(borrowed.execute_authorized(&auth,&request,&op,h).await.is_err());assert!(borrowed.recorder.calls.is_empty());
        let index=probe.calls();let h=borrowed.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=borrowed.acquire_authorized(&auth,&request,h).await.unwrap();assert_eq!(borrowed.execute_authorized(&auth,&request,&op,h).await.unwrap().state(),ResultState::Uncommitted);probe.compare(&borrowed.recorder,index);assert_eq!(borrowed.recorder.events,["send","decoded"]);drop(borrowed);tx.rollback().await.unwrap();
        let mut tx=raw.begin().await.unwrap();let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();
        let h=borrowed.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=borrowed.acquire_authorized(&auth,&request,h).await.unwrap();
        let e=borrowed.execute_authorized_atomic(&auth,&request,&op,ExpectedRows::exactly(1).unwrap(),h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Unsupported);assert_eq!(e.disposition(),AtomicDisposition::NotStarted);assert!(borrowed.recorder.events.is_empty());
        let h=borrowed.authorization_candidate(&auth,&request,&op,MultiRowUpdate,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=borrowed.acquire_authorized(&auth,&request,h).await.unwrap();drop(borrowed);tx.rollback().await.unwrap();
        let mut tx=raw.begin().await.unwrap();let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();assert!(borrowed.execute_authorized(&auth,&request,&op,h).await.is_err());assert!(borrowed.recorder.calls.is_empty());drop(borrowed);tx.rollback().await.unwrap();
        let index=probe.calls();let h=s.authorization_candidate(&auth,&request,&delete,MultiRowDelete,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();
        let e=s.execute_authorized_atomic(&auth,&request,&delete,ExpectedRows::exactly(2).unwrap(),h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::ExpectationMismatch);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);probe.compare(&s.recorder,index);s.recorder=Recorder::default();
        assert_eq!(sqlx::query("SELECT COUNT(*) FROM quent_dispatch_test.items WHERE tenant=11").fetch_one(&mut *observer).await.unwrap().get::<i64,_>(0),1);
        let index=probe.calls();let h=s.authorization_candidate(&auth,&request,&delete,MultiRowDelete,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();assert_eq!(s.execute_authorized(&auth,&request,&delete,h).await.unwrap().state(),ResultState::Observed);probe.compare(&s.recorder,index);
        s.recorder=Recorder::default();
        sqlx::query("INSERT INTO quent_dispatch_test.items(id,tenant,txt) VALUES(1,11,'before')").execute(&mut *observer).await.unwrap();
        let index=probe.calls();let h=s.authorization_candidate(&auth,&request,&delete,MultiRowDelete,MAX_TTL).unwrap();#[cfg(feature="review")] let _=h.review();let h=s.acquire_authorized(&auth,&request,h).await.unwrap();assert_eq!(s.execute_authorized_atomic(&auth,&request,&delete,ExpectedRows::exactly(1).unwrap(),h).await.unwrap().state(),ResultState::Committed);probe.compare(&s.recorder,index);
        assert_eq!(sqlx::query("SELECT COUNT(*) FROM quent_dispatch_test.items WHERE tenant=11").fetch_one(&mut *observer).await.unwrap().get::<i64,_>(0),0);
        println!("QT-AI-09 PR2 {:?}: request/session-bound provider positive; independently counted provider refusals and 0send; inspected SQL allocation/ordered args/all metadata equal; direct/owned/borrowed gates; expiry after begin rollback",d);
    }};
}

macro_rules! suite {
    ($name:ident, $feature:literal, $session:ident, $borrowed:ident, $connection:ty, $options:ty, $dialect:expr, $dsn:literal, $create:literal, $seed:literal, $ssl:expr) => {
        #[cfg(feature=$feature)]
        #[test]
        fn $name() {
            use sqlx::{ConnectOptions, Connection, Row};
            assert_eq!(
                std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),
                Ok("quent_dispatch_test")
            );
            let options: $options = std::env::var($dsn).unwrap().parse().unwrap();
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            #[cfg(feature = "output-tracing")]
            let capture = output_capture::Capture::default();
            #[cfg(feature = "output-tracing")]
            let subscriber = tracing::subscriber::set_default(capture.clone());
            runtime.block_on(Box::pin(async {
                let d = $dialect;
                let c = context(d, 11, "subject", vec![]);
                let mut observer =
                    <$connection>::connect_with(&options.clone().disable_statement_logging())
                        .await
                        .unwrap();
                sqlx::query($create).execute(&mut observer).await.unwrap();
                let result = std::panic::AssertUnwindSafe(async {
                    authorized_cases!(
                        $session,
                        $borrowed,
                        $connection,
                        options,
                        d,
                        c,
                        observer,
                        $create,
                        $seed,
                        $ssl
                    );
                });
                use futures_util::FutureExt;
                let result = result.catch_unwind().await;
                sqlx::query("DROP TABLE IF EXISTS quent_dispatch_test.items")
                    .execute(&mut observer)
                    .await
                    .unwrap();
                if let Err(panic) = result {
                    std::panic::resume_unwind(panic);
                }
            }));
            #[cfg(feature = "output-tracing")]
            {
                drop(subscriber);
                assert!(
                    capture.0.lock().unwrap().is_empty(),
                    "no implicit provider/driver log"
                );
                let positive = output_capture::Capture::default();
                tracing::subscriber::with_default(positive.clone(), || {
                    tracing::warn!("CANARY_detector")
                });
                assert!(
                    positive
                        .0
                        .lock()
                        .unwrap()
                        .iter()
                        .any(|s| s.contains("CANARY_detector"))
                );
            }
        }
    };
}
suite!(
    live_authorized_postgres,
    "postgres",
    PostgresSession,
    BorrowedPostgresTransaction,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    "CREATE TABLE quent_dispatch_test.items (id BIGINT NOT NULL,tenant BIGINT NOT NULL,small SMALLINT NULL,normal INT NULL,big BIGINT NULL,flag BOOLEAN NULL,txt TEXT NULL,optional BIGINT NULL,PRIMARY KEY(tenant,id))",
    "INSERT INTO quent_dispatch_test.items(id,tenant,txt) VALUES(1,11,'before'),(2,22,'other')",
    sqlx::postgres::PgSslMode::Disable
);
suite!(
    live_authorized_mysql,
    "mysql",
    MysqlSession,
    BorrowedMysqlTransaction,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "CREATE TABLE quent_dispatch_test.items (id BIGINT NOT NULL,tenant BIGINT NOT NULL,small SMALLINT NULL,normal INT NULL,big BIGINT NULL,flag BOOLEAN NULL,txt TEXT NULL,optional BIGINT NULL,tiny TINYINT NULL,u_tiny TINYINT UNSIGNED NULL,u_small SMALLINT UNSIGNED NULL,u_normal INT UNSIGNED NULL,u_big BIGINT UNSIGNED NULL,PRIMARY KEY(tenant,id))",
    "INSERT INTO quent_dispatch_test.items(id,tenant,txt) VALUES(1,11,'before'),(2,22,'other')",
    sqlx::mysql::MySqlSslMode::Disabled
);
