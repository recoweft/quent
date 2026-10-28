//! QT-AI-14: one tenant request, reviewed authorization, completion and observer.
//! The registered adapter is an application assertion used as a test control;
//! it does not authenticate a user, physical route or remote authorization service.
use super::*;

fn companion_table(name: &str) -> Table {
    Table::new(vec![id("quent_dispatch_test"), id(name)], None).unwrap()
}
fn companion_column(table: &str, name: &str) -> Column {
    companion_table(table).column(
        id(name),
        if name == "tenant" {
            SqlType::I64
        } else {
            SqlType::Text
        },
        name == "payload",
    )
}
fn acceptance_context(d: Dialect, subject: &str) -> DatabaseContext {
    acceptance_context_variant(d, 11, subject, 0)
}
fn acceptance_context_variant(
    d: Dialect,
    current_tenant: i64,
    subject: &str,
    variant: u8,
) -> DatabaseContext {
    let base = context_schema(
        d,
        current_tenant,
        subject,
        if variant == 11 {
            vec![PolicyDeclaration::Readonly(changed())]
        } else {
            vec![]
        },
        variant,
        vec![key(), tenant(), changed()],
    );
    let mut facts = base.sensitive_source().schema().unwrap().2.to_vec();
    for table in ["acceptance_audit", "acceptance_outbox"] {
        facts.push(
            ScopeFacts::application_asserted(
                companion_table(table),
                d,
                [
                    "tenant",
                    "record_id",
                    "command_id",
                    "namespace",
                    "subject",
                    "payload",
                ]
                .into_iter()
                .map(|n| companion_column(table, n))
                .collect(),
                vec![Key::Primary(vec![
                    companion_column(table, "tenant"),
                    companion_column(table, "record_id"),
                ])],
                TableContext::OrdinaryBaseTableWithoutEffects,
            )
            .unwrap(),
        );
    }
    let source = base.sensitive_source();
    let (database, address, catalog, _) = source.target();
    let binding = DatabaseBinding::application_supplied(
        DatabaseIdentity::new(database).unwrap(),
        TargetAddress::new(address).unwrap(),
        catalog.clone(),
        d,
    );
    let (schema_id, schema_generation, _) = source.schema().unwrap();
    base.with_application(
        ApplicationContext::application_supplied(
            binding.clone(),
            Some(
                SchemaSnapshot::application_supplied(
                    binding.clone(),
                    SchemaIdentity::new(schema_id).unwrap(),
                    Generation::new(schema_generation).unwrap(),
                    facts,
                )
                .unwrap(),
            ),
            Some(SubjectIdentity::new(subject).unwrap()),
            Some(
                TenantSnapshot::application_supplied(binding, tenant(), Value::I64(current_tenant))
                    .unwrap(),
            ),
        )
        .unwrap(),
    )
    .unwrap()
}
fn acceptance_bundle(n: u8, payload: &str) -> CommandBundle {
    let companion = |table: &str| {
        let column = |name| companion_column(table, name);
        let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
        CommandCompanion::application_supplied(
            Draft::insert(
                companion_table(table),
                vec![
                    (
                        column("record_id"),
                        Input::Value(Value::Text("one-record".into())),
                    ),
                    (
                        column("command_id"),
                        Input::Value(Value::Text(hex(&[n; 16]))),
                    ),
                    (
                        column("namespace"),
                        Input::Value(Value::Text(hex(b"tenant-acceptance"))),
                    ),
                    (
                        column("subject"),
                        Input::Value(Value::Text("subject".into())),
                    ),
                    (column("payload"), Input::Value(Value::Text(payload.into()))),
                ],
            )
            .freeze(),
            column("tenant"),
            column("record_id"),
            column("command_id"),
            column("namespace"),
            column("subject"),
        )
    };
    CommandBundle::new(
        Draft::update(
            table(),
            write(Value::Text("CANARY_bundle_saved".into()), changed()),
        )
        .filter(filter(1))
        .freeze(),
        companion("acceptance_audit"),
        companion("acceptance_outbox"),
    )
}

macro_rules! tenant_acceptance {
    ($name:ident, $feature:literal, $session:ident, $borrowed:ident, $connection:ty, $options:ty, $d:expr, $dsn:literal, $ssl:expr) => {
        #[cfg(feature = $feature)]
        #[test]
        fn $name() {
            use crate::authorization::{AdapterResponse, AuthorizationDecision, AuthorizationRequest, TrustedAuthorizationAdapter};
            use sqlx::{ConnectOptions, Connection, Row};
            use std::sync::{Mutex, atomic::{AtomicUsize, Ordering}};
            // Explicit provider observes the same already-reviewed retained record.
            type ReviewedRecord = (usize, String, Vec<Value>, Metadata);
            #[derive(Default)]
            struct ApplicationReview {
                calls: AtomicUsize,
                reviewed: Mutex<Vec<ReviewedRecord>>,
                response: Mutex<Option<AdapterResponse>>,
            }
            impl TrustedAuthorizationAdapter for Arc<ApplicationReview> {
                fn authorize<'a>(&'a self, request: AuthorizationRequest<'a>) -> std::pin::Pin<Box<dyn std::future::Future<Output=AuthorizationDecision> + Send + 'a>> {
                    self.calls.fetch_add(1, Ordering::SeqCst);
                    assert!(request.is_runtime_bound());
                    let inspection = request.sensitive_preparation().inspect();
                    let review = crate::review::inspection(&inspection);
                    assert_eq!(review.state(), crate::review::State::Refused);
                    for output in [review.to_json(), review.to_pretty(), format!("{review:?}")] {
                        for secret in ["CANARY", "items", "subject", "UPDATE ", "quent_dispatch_test"] {
                            assert!(!output.contains(secret));
                        }
                    }
                    let source = inspection.sensitive_source();
                    self.reviewed.lock().unwrap().push((source.sql().as_ptr() as usize, source.sql().to_owned(), source.args().to_vec(), source.metadata().clone()));
                    // Test control only. A real application must independently
                    // authenticate and authorize before returning this decision.
                    let response = self.response.lock().unwrap().take().unwrap_or(AdapterResponse::AllowCandidate);
                    Box::pin(async move { request.respond(response) })
                }
            }
            assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(), Ok("quent_dispatch_test"));
            let options: $options = std::env::var($dsn).unwrap().parse().unwrap();
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            runtime.block_on(async {
                let d = $d;
                // This test deliberately owns items only for its duration. Setup
                // and independent observer traffic are outside business counts.
                let mut observer = <$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                sqlx::query("DROP TABLE IF EXISTS quent_dispatch_test.items").execute(&mut observer).await.unwrap();
                sqlx::query("CREATE TABLE quent_dispatch_test.items(id BIGINT NOT NULL,tenant BIGINT NOT NULL,txt TEXT NULL,PRIMARY KEY(tenant,id))").execute(&mut observer).await.unwrap();
                sqlx::query("INSERT INTO quent_dispatch_test.items VALUES(1,11,'initial'),(2,22,'other')").execute(&mut observer).await.unwrap();
                let c = acceptance_context(d,"subject");
                let provider = Arc::new(ApplicationReview::default());
                let auth = AuthorizationContext::register(c.clone(), "application-review-control".into(), 1, vec![], provider.clone()).unwrap();
                let request = AuthorizationExecutionRequest::new();
                let mut session = $session::connect(options.clone(), c.clone()).await.unwrap();
                let op = Draft::update(table(),write(Value::Text("CANARY_tenant_change".into()),changed())).filter(eq(tenant(),Value::I64(11))).freeze();
                assert_eq!(provider.calls.load(Ordering::SeqCst),0);
                assert!(session.recorder.calls.is_empty());
                assert!(session.recorder.events.is_empty());
                assert_eq!(session.execute(&c,op.clone()).await.unwrap_err(),ExecutionError::RiskRefused);
                // Plan/review success and application assertions are not permits.
                assert!(session.recorder.calls.is_empty());
                assert!(session.recorder.events.is_empty());
                // Same original operation, separately supplied changed current:
                // successful acquisition is consumed, then refused before begin.
                let candidate = session.authorization_candidate(&auth,&request,&op,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
                let prior_calls = provider.calls.load(Ordering::SeqCst);
                let review = candidate.review();
                assert_eq!(review.state(),crate::review::State::Refused);
                for _ in 0..3 { assert_eq!(candidate.review().to_json(),review.to_json()); }
                assert_eq!(provider.calls.load(Ordering::SeqCst),prior_calls);

                let permit = session.acquire_authorized(&auth,&request,candidate).await.unwrap();
                let changed_current = acceptance_context(d,"other_subject");
                let changed_auth = auth.with_database(changed_current).unwrap();
                let failure = session.execute_authorized_atomic(&changed_auth,&request,&op,ExpectedRows::exactly(1).unwrap(),permit).await.unwrap_err();
                assert_eq!(failure.disposition(),AtomicDisposition::NotStarted);
                assert!(session.recorder.calls.is_empty());
                assert!(session.recorder.events.is_empty());
                // Reacquire explicitly: committed result follows full consumption
                // and exact row expectation; a fresh observer sees tenant isolation.
                let candidate = session.authorization_candidate(&auth,&request,&op,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
                let prior_calls = provider.calls.load(Ordering::SeqCst);
                let review = candidate.review();
                assert_eq!(review.state(),crate::review::State::Refused);
                for _ in 0..3 { assert_eq!(candidate.review().to_json(),review.to_json()); }
                assert_eq!(provider.calls.load(Ordering::SeqCst),prior_calls);

                // Artificial observation seam only; the application above used
                // the public output-only review, never a private getter.
                let source = candidate.candidate.test_review_record().sensitive_source();
                let final_record = source.final_source();
                let before_review = (final_record.sql().as_ptr() as usize,final_record.sql().to_owned(),final_record.args().to_vec(),final_record.metadata().clone());
                assert_eq!(candidate.review().to_json(),review.to_json());
                assert!(session.recorder.calls.is_empty());
                assert!(session.recorder.events.is_empty());
                let permit = session.acquire_authorized(&auth,&request,candidate).await.unwrap();
                let result = session.execute_authorized_atomic(&auth,&request,&op,ExpectedRows::exactly(1).unwrap(),permit).await.unwrap();
                assert_eq!(result.state(),ResultState::Committed);
                assert!(result.sensitive_rows().is_empty());
                assert_eq!(session.recorder.events,["begin","begun","send","decoded","expectation","commit","committed"]);
                {
                let records = provider.reviewed.lock().unwrap();
                assert_eq!(records[1],before_review);
                let (address,sql,args,metadata) = &records[1];
                assert_eq!(session.recorder.sql_addresses,[*address]);
                assert_eq!(session.recorder.calls,[(sql.clone(),args.clone())]);
                assert_eq!(session.recorder.metadata.as_slice(),std::slice::from_ref(metadata));
                }
                let rows = sqlx::query("SELECT tenant,id,txt FROM quent_dispatch_test.items ORDER BY tenant,id").fetch_all(&mut observer).await.unwrap();
                assert_eq!(rows.len(),2);
                assert_eq!(rows[0].get::<i64,_>(0),11);
                assert_eq!(rows[0].get::<String,_>(2),"CANARY_tenant_change");
                assert_eq!(rows[1].get::<i64,_>(0),22);
                assert_eq!(rows[1].get::<String,_>(2),"other");
                // Same application request: an incorrect expected count rolls
                // back the following attempted change, preserving saved state.
                let rollback = Draft::update(table(),write(Value::Text("CANARY_rollback".into()),changed())).filter(eq(tenant(),Value::I64(11))).freeze();
                let candidate = session.authorization_candidate(&auth,&request,&rollback,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
                let permit = session.acquire_authorized(&auth,&request,candidate).await.unwrap();
                let failure = session.execute_authorized_atomic(&auth,&request,&rollback,ExpectedRows::exactly(2).unwrap(),permit).await.unwrap_err();
                assert_eq!(failure.cause(),ExecutionError::ExpectationMismatch);
                assert_eq!(failure.disposition(),AtomicDisposition::RolledBack);
                assert_eq!(session.recorder.calls.len(),2);
                assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<String,_>(0),"CANARY_tenant_change");
                // Borrowed completion is caller-owned. Read using ordinary CRUD
                // through the same gate, then explicitly roll back at the caller.
                let mut connection = <$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                let mut tx = connection.begin().await.unwrap();
                let mut borrowed = $borrowed::new(&mut tx,c.clone()).unwrap();
                let read = Draft::select(table(),vec![Projection::new(changed(),None)]).filter(filter(1)).limit(1).freeze();
                let result = borrowed.execute(&c,read).await.unwrap();
                assert_eq!(result.state(),ResultState::Uncommitted);
                assert_eq!(result.sensitive_rows(),[vec![Value::Text("CANARY_tenant_change".into())]]);
                assert_eq!(borrowed.recorder.calls.len(),1);
                assert_eq!(borrowed.recorder.events,["send","row_decoded","decoded"]);
                drop(borrowed);
                tx.rollback().await.unwrap();
                // Continue the SAME tenant/context and business table with a
                // durable command. IDs/inputs are fixed before first submission.
                let ledger_table = Table::new(vec![id("quent_dispatch_test"),id("acceptance_ledger")],None).unwrap();
                let ledger = CommandLedger::application_supplied(ledger_table).unwrap();
                let ddl = if d == Dialect::Postgres {
                    "CREATE TABLE quent_dispatch_test.acceptance_ledger(identity BYTEA PRIMARY KEY,input BYTEA NOT NULL,result BYTEA NULL,version SMALLINT NOT NULL)"
                } else {
                    "CREATE TABLE quent_dispatch_test.acceptance_ledger(identity VARBINARY(2048) PRIMARY KEY,input LONGBLOB NOT NULL,result VARBINARY(29) NULL,version SMALLINT NOT NULL) ENGINE=InnoDB"
                };
                sqlx::query(ddl).execute(&mut observer).await.unwrap();
                let namespace = CommandNamespace::new(b"tenant-acceptance".to_vec()).unwrap();
                let command_id = CommandId::new([14;16]);
                let command = Draft::update(table(),write(Value::Text("CANARY_saved_command".into()),changed())).filter(filter(1)).freeze();
                let before = session.recorder.calls.len();
                let handle = session.validate_command(&c,&command,&namespace,command_id.clone(),&ledger,MAX_TTL).unwrap();
                assert_eq!(session.recorder.calls.len(),before);
                let saved = session.execute_command(&c,&command,handle).await.unwrap();
                assert_eq!(saved.kind(),CommandResultKind::FirstCommit);
                assert_eq!(saved.sensitive_result().state(),ResultState::Committed);
                assert_eq!(session.recorder.calls.len(),before+1);
                let handle = session.validate_command(&c,&command,&namespace,command_id.clone(),&ledger,MAX_TTL).unwrap();
                let replay = session.execute_command(&c,&command,handle).await.unwrap();
                assert_eq!(replay.kind(),CommandResultKind::Replayed);
                assert_eq!(replay.sensitive_result().affected_rows(),saved.sensitive_result().affected_rows());
                assert_eq!(session.recorder.calls.len(),before+1);
                let conflicting = Draft::update(table(),write(Value::Text("CANARY_conflict".into()),changed())).filter(filter(1)).freeze();
                let handle = session.validate_command(&c,&conflicting,&namespace,command_id.clone(),&ledger,MAX_TTL).unwrap();
                assert_eq!(session.execute_command(&c,&conflicting,handle).await.unwrap_err().cause(),ExecutionError::IdentityConflict);
                assert_eq!(session.recorder.calls.len(),before+1);
                assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<String,_>(0),"CANARY_saved_command");
                let stored = sqlx::query("SELECT input,result,version FROM quent_dispatch_test.acceptance_ledger").fetch_all(&mut observer).await.unwrap();
                assert_eq!(stored.len(),1);
                assert_eq!(&stored[0].get::<Vec<u8>,_>(0)[..4],b"QCC1");
                assert_eq!(&stored[0].get::<Vec<u8>,_>(1)[..4],b"QCR1");
                assert_eq!(stored[0].get::<i16,_>(2),1);
                // Artificial storage corruption is labelled separately from
                // transport loss. Refusal cannot reinterpret corruption as missing.
                sqlx::query("UPDATE quent_dispatch_test.acceptance_ledger SET version=9").execute(&mut observer).await.unwrap();
                let handle = session.validate_command(&c,&command,&namespace,command_id,&ledger,MAX_TTL).unwrap();
                assert_eq!(session.lookup_command(&c,&command,handle).await.unwrap_err().cause(),ExecutionError::LedgerInvalid);
                assert_eq!(session.recorder.calls.len(),before+1);
                assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<String,_>(0),"CANARY_saved_command");
                // Same app current/request/business table, fixed audit/outbox
                // role mapping. Replaying a bundle must never regenerate effects.
                for name in ["acceptance_audit","acceptance_outbox"] {
                    let ddl=format!("CREATE TABLE quent_dispatch_test.{name}(tenant BIGINT NOT NULL,record_id VARCHAR(128) NOT NULL,command_id VARCHAR(32) NOT NULL,namespace VARCHAR(256) NOT NULL,subject VARCHAR(128) NOT NULL,payload TEXT NULL,PRIMARY KEY(tenant,record_id)){}",if d==Dialect::Mysql {" ENGINE=InnoDB"}else{""});
                    sqlx::query(sqlx::AssertSqlSafe(ddl)).execute(&mut observer).await.unwrap();
                }
                if d==Dialect::Mysql {
                    assert_eq!(sqlx::query("SELECT COUNT(*) FROM information_schema.tables WHERE TABLE_SCHEMA='quent_dispatch_test' AND TABLE_NAME IN ('items','acceptance_ledger','acceptance_audit','acceptance_outbox') AND ENGINE='InnoDB'").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),4);
                }
                // Restore the deliberately corrupted plain record; bundle uses
                // a different explicitly durable ID in the same ledger namespace.
                sqlx::query("UPDATE quent_dispatch_test.acceptance_ledger SET version=1").execute(&mut observer).await.unwrap();
                let bundle=acceptance_bundle(15,"CANARY_companion");
                let start=session.recorder.calls.len();
                let handle=session.validate_command_bundle(&c,&bundle,&namespace,CommandId::new([15;16]),&ledger,MAX_TTL).unwrap();
                let result=session.execute_command_bundle(&c,&bundle,handle).await.unwrap();
                assert_eq!(result.kind(),CommandResultKind::FirstCommit);
                assert_eq!(result.sensitive_result().state(),ResultState::Committed);
                for result in result.sensitive_companion_results().unwrap() { assert_eq!(result.state(),ResultState::Committed);assert!(result.sensitive_rows().is_empty()); }
                assert_eq!(session.recorder.calls.len(),start+3);
                let handle=session.validate_command_bundle(&c,&bundle,&namespace,CommandId::new([15;16]),&ledger,MAX_TTL).unwrap();
                assert_eq!(session.execute_command_bundle(&c,&bundle,handle).await.unwrap().kind(),CommandResultKind::Replayed);
                assert_eq!(session.recorder.calls.len(),start+3);
                let changed_payload=acceptance_bundle(15,"CANARY_changed_payload");
                let handle=session.validate_command_bundle(&c,&changed_payload,&namespace,CommandId::new([15;16]),&ledger,MAX_TTL).unwrap();
                assert_eq!(session.execute_command_bundle(&c,&changed_payload,handle).await.unwrap_err().cause(),ExecutionError::IdentityConflict);
                assert_eq!(session.recorder.calls.len(),start+3);
                let duplicate=acceptance_bundle(16,"CANARY_duplicate_record");
                let handle=session.validate_command_bundle(&c,&duplicate,&namespace,CommandId::new([16;16]),&ledger,MAX_TTL).unwrap();
                assert_eq!(session.execute_command_bundle(&c,&duplicate,handle).await.unwrap_err().disposition(),AtomicDisposition::RolledBack);
                assert_eq!(session.recorder.calls.len(),start+5); // business1, rejected audit1, outbox0
                assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<String,_>(0),"CANARY_bundle_saved");
                assert_eq!(sqlx::query("SELECT COUNT(*) FROM quent_dispatch_test.acceptance_ledger").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),2);
                for name in ["acceptance_audit","acceptance_outbox"] {
                    let rows=sqlx::query(sqlx::AssertSqlSafe(format!("SELECT tenant,record_id,payload FROM quent_dispatch_test.{name}"))).fetch_all(&mut observer).await.unwrap();
                    assert_eq!(rows.len(),1);assert_eq!(rows[0].get::<i64,_>(0),11);assert_eq!(rows[0].get::<String,_>(1),"one-record");assert_eq!(rows[0].get::<String,_>(2),"CANARY_companion");
                    sqlx::query(sqlx::AssertSqlSafe(format!("DROP TABLE quent_dispatch_test.{name}"))).execute(&mut observer).await.unwrap();
                }
                println!("QT-AI-14 {d:?} same-flow bundle: first business/audit/outbox=1/1/1; replay/conflict=0/0/0; duplicate companion=1/1/0 rollbackACK; observer ledger2/audit1/outbox1 and prior saved business");
                // Dedicated plaintext fixture preserves the same app context
                // and table while dropping a real native COMMIT response. This
                // is bounded protocol fault evidence, not general TLS loss.
                for (i,mode) in [command_network::Mode::BeforeCommit,command_network::Mode::LoseReply,command_network::Mode::Pass].into_iter().enumerate() {
                    let value=if i==0 {"CANARY_not_committed"}else if i==1 {"CANARY_replyloss"}else{"CANARY_ack"};
                    let operation=Draft::update(table(),write(Value::Text(value.into()),changed())).filter(filter(1)).freeze();
                    let target=std::net::SocketAddr::new(options.get_host().parse().unwrap(),options.get_port());
                    let proxy=command_network::Proxy::new(target,d==Dialect::Postgres,mode);
                    let mut network=$session::connect(options.clone().host("127.0.0.1").port(proxy.address.port()).ssl_mode($ssl),c.clone()).await.unwrap();
                    let durable=CommandId::new([17+i as u8;16]);
                    let handle=network.validate_command(&c,&operation,&namespace,durable.clone(),&ledger,MAX_TTL).unwrap();
                    let outcome=tokio::time::timeout(Duration::from_secs(10),network.execute_command(&c,&operation,handle)).await.unwrap();
                    if i==2 { assert_eq!(outcome.unwrap().kind(),CommandResultKind::FirstCommit); } else {
                        let error=outcome.unwrap_err();assert_eq!(error.cause(),ExecutionError::OutcomeUnknown);assert_eq!(error.disposition(),AtomicDisposition::OutcomeUnknown);assert!(!network.is_available());
                    }
                    assert_eq!(network.recorder.calls.len(),1);
                    assert_eq!(proxy.sent.load(Ordering::SeqCst),i!=0);assert_eq!(proxy.reply.load(Ordering::SeqCst),i!=0);
                    drop(network);drop(proxy);
                    let mut reconciliation=$session::connect(options.clone(),c.clone()).await.unwrap();
                    let handle=reconciliation.validate_command(&c,&operation,&namespace,durable,&ledger,MAX_TTL).unwrap();
                    let stored=reconciliation.lookup_command(&c,&operation,handle).await.unwrap();
                    assert_eq!(stored.is_some(),i!=0);if let Some(result)=stored {assert_eq!(result.kind(),CommandResultKind::Replayed);}
                    assert!(reconciliation.recorder.calls.is_empty());
                    let expected=if i==0 {"CANARY_bundle_saved"}else{value};
                    assert_eq!(sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<String,_>(0),expected);
                    println!("QT-AI-14 {d:?} same-table command protocol case={i}: business1 forwarded_commit={} serverACK={} clientACK={} reconciliation_business0 stored={} observer_expected",i!=0,i!=0,i==2,i!=0);
                }
                sqlx::query("DROP TABLE quent_dispatch_test.acceptance_ledger").execute(&mut observer).await.unwrap();
                assert_eq!(provider.calls.load(Ordering::SeqCst),3);
                let business_before = session.recorder.calls.len();
                let tx_before = session.recorder.events.len();
                // Review never resets TTL, even if an expired candidate can
                // still produce a historical output-only review.
                let expired = session.authorization_candidate(&auth,&request,&op,AuthorizationScope::MultiRowUpdate,Duration::from_millis(1)).unwrap();
                let old_review = expired.review().to_json();
                tokio::time::sleep(Duration::from_millis(3)).await;
                assert_eq!(expired.review().to_json(),old_review);
                assert_eq!(session.acquire_authorized(&auth,&request,expired).await.unwrap_err(),ExecutionError::Authorization(AuthorizationError::Expired));
                assert_eq!(provider.calls.load(Ordering::SeqCst),3);
                for (response,error) in [(AdapterResponse::Deny,AuthorizationError::Denied),(AdapterResponse::Error,AuthorizationError::ProviderError),(AdapterResponse::Cancelled,AuthorizationError::Cancelled)] {
                    *provider.response.lock().unwrap()=Some(response);
                    let candidate=session.authorization_candidate(&auth,&request,&op,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
                    assert_eq!(candidate.review().state(),crate::review::State::Refused);
                    assert_eq!(session.acquire_authorized(&auth,&request,candidate).await.unwrap_err(),ExecutionError::Authorization(error));
                }
                let other_request=AuthorizationExecutionRequest::new();
                let other_session=$session::connect(options.clone(),c.clone()).await.unwrap();
                for wrong_session in [false,true] {
                    let candidate=session.authorization_candidate(&auth,&request,&op,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
                    let _ = candidate.review();
                    let error=if wrong_session { other_session.acquire_authorized(&auth,&request,candidate).await.unwrap_err() } else { session.acquire_authorized(&auth,&other_request,candidate).await.unwrap_err() };
                    assert_eq!(error,ExecutionError::Authorization(AuthorizationError::Mismatch));
                }
                assert_eq!(provider.calls.load(Ordering::SeqCst),6);
                // Reviewed candidates still compare independently supplied
                // complete contents, including same-generation changes.
                Box::pin(async {
                for variant in [0u8,1,2,4,5,6,7,8,9,10,11] {
                    let changed=acceptance_context_variant(d,if variant==10 {22}else{11},if variant==0 {"other_subject"}else{"subject"},variant);
                    let current=if matches!(variant,1|2) {AuthorizationContext::register(changed,"application-review-control".into(),1,vec![],provider.clone()).unwrap()}else{auth.with_database(changed).unwrap()};
                    let candidate=session.authorization_candidate(&auth,&request,&op,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
                    let _=candidate.review();
                    let permit=session.acquire_authorized(&auth,&request,candidate).await.unwrap();
                    assert!(session.execute_authorized(&current,&request,&op,permit).await.is_err());
                    assert_eq!(session.recorder.calls.len(),business_before);
                    assert_eq!(session.recorder.events.len(),tx_before);
                }
                let candidate=session.authorization_candidate(&auth,&request,&op,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
                let _=candidate.review();let permit=session.acquire_authorized(&auth,&request,candidate).await.unwrap();
                assert!(session.execute_authorized(&auth,&other_request,&op,permit).await.is_err());
                let candidate=session.authorization_candidate(&auth,&request,&op,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
                let _=candidate.review();let permit=session.acquire_authorized(&auth,&request,candidate).await.unwrap();
                let different=Draft::update(table(),write(Value::Text("CANARY_substitute".into()),changed())).filter(eq(tenant(),Value::I64(11))).freeze();
                assert!(session.execute_authorized(&auth,&request,&different,permit).await.is_err());
                }).await;
                assert_eq!(provider.calls.load(Ordering::SeqCst),19);
                assert_eq!(session.recorder.calls.len(),business_before);
                assert_eq!(session.recorder.events.len(),tx_before);
                let final_rows=sqlx::query("SELECT tenant,id,txt FROM quent_dispatch_test.items ORDER BY tenant,id").fetch_all(&mut observer).await.unwrap();
                assert_eq!(final_rows.len(),2);assert_eq!(final_rows[0].get::<i64,_>(0),11);assert_eq!(final_rows[0].get::<String,_>(2),"CANARY_ack");assert_eq!(final_rows[1].get::<i64,_>(0),22);assert_eq!(final_rows[1].get::<String,_>(2),"other");
                sqlx::query("DROP TABLE quent_dispatch_test.items").execute(&mut observer).await.unwrap();
                println!("QT-AI-14 {d:?} QT-C01/C03/C08/C11/C12/C14/C15/C16/C17/C18: provider=19 (3 baseline acquisitions,3 rejection controls,13 mismatched dispatch acquisitions) authorized_business=2 plain_command_business=1 command_first_commit=1 replay=1 conflict=1 artificial_ledger_invalid=1 owned_commits=1 owned_rollbacks=1 borrowed_reads=1 caller_rollback=1; changed-current business=0 Tx=0; independent observer full tenant rows=2");
            });
        }
    }
}
tenant_acceptance!(
    live_acceptance_postgres,
    "postgres",
    PostgresSession,
    BorrowedPostgresTransaction,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    sqlx::postgres::PgSslMode::Disable
);
tenant_acceptance!(
    live_acceptance_mysql,
    "mysql",
    MysqlSession,
    BorrowedMysqlTransaction,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    sqlx::mysql::MySqlSslMode::Disabled
);
