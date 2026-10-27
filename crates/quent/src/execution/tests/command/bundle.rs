use super::*;
fn bt(name: &str) -> Table {
    Table::new(vec![id("quent_dispatch_test"), id(name)], None).unwrap()
}
fn bc(table: &str, name: &str) -> Column {
    bt(table).column(
        id(name),
        match name {
            "flag" => SqlType::Bool,
            "note" | "record_id" | "command_id" | "namespace" | "subject" | "payload" => {
                SqlType::Text
            }
            _ => SqlType::I64,
        },
        matches!(name, "note" | "payload"),
    )
}
fn context(d: Dialect, variant: u8, policies: Vec<PolicyDeclaration>) -> DatabaseContext {
    let b = DatabaseBinding::application_supplied(
        DatabaseIdentity::new(if variant == 4 {
            "other_target"
        } else {
            "bundle_target"
        })
        .unwrap(),
        TargetAddress::new("bundle_route").unwrap(),
        id("quent_dispatch_test"),
        d,
    );
    let g = Generation::new(1).unwrap();
    let tenant = bc("bundle_items", "tenant");
    let mut facts = vec![];
    for table in ["bundle_items", "bundle_audit", "bundle_outbox"] {
        if variant == 1 && table == "bundle_audit" {
            continue;
        }
        let names: Vec<&str> = if table == "bundle_items" {
            vec!["tenant", "id", "amount", "flag", "note"]
        } else {
            vec![
                "tenant",
                "record_id",
                "command_id",
                "namespace",
                "subject",
                "payload",
                "recorded_at",
                "sequence",
            ]
        };
        let mut columns: Vec<_> = names.into_iter().map(|n| bc(table, n)).collect();
        if variant == 5 && table != "bundle_items" {
            for i in 0..80 {
                columns.push(bt(table).column(id(&format!("extra_{i}")), SqlType::Text, false));
            }
        }
        facts.push(
            ScopeFacts::application_asserted(
                bt(table),
                d,
                columns,
                vec![Key::Primary(vec![
                    bc(table, "tenant"),
                    bc(
                        table,
                        if table == "bundle_items" {
                            "id"
                        } else {
                            "record_id"
                        },
                    ),
                ])],
                if variant == 2 && table == "bundle_audit" {
                    TableContext::Unsupported
                } else {
                    TableContext::OrdinaryBaseTableWithoutEffects
                },
            )
            .unwrap(),
        );
    }
    let mut declarations = vec![PolicyDeclaration::Tenant(tenant.clone())];
    declarations.extend(policies);
    DatabaseContext::new(
        Settings::new(
            b.clone(),
            SettingsIdentity::new("settings").unwrap(),
            g,
            PolicySet::new(
                b.clone(),
                PolicyIdentity::new("policy").unwrap(),
                g,
                declarations,
            )
            .unwrap(),
        )
        .unwrap(),
        ApplicationContext::application_supplied(
            b.clone(),
            Some(
                SchemaSnapshot::application_supplied(
                    b.clone(),
                    SchemaIdentity::new("schema").unwrap(),
                    g,
                    facts,
                )
                .unwrap(),
            ),
            Some(
                SubjectIdentity::new(if variant == 3 {
                    "other_subject"
                } else {
                    "CANARY_bundle_subject"
                })
                .unwrap(),
            ),
            Some(TenantSnapshot::application_supplied(b, tenant, Value::I64(11)).unwrap()),
        )
        .unwrap(),
    )
    .unwrap()
}
fn business(n: i64) -> Operation {
    Draft::insert(
        bt("bundle_items"),
        vec![
            (bc("bundle_items", "id"), Input::Value(Value::I64(n))),
            (bc("bundle_items", "amount"), Input::Value(Value::I64(0))),
            (bc("bundle_items", "flag"), Input::Value(Value::Bool(false))),
            (
                bc("bundle_items", "note"),
                Input::Value(Value::Null(SqlType::Text)),
            ),
        ],
    )
    .freeze()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
// Variants change durable inputs, never ambient random/time. IDs and contents
// are determined before validate/send, including retry/lookup reconstruction.
fn companion(table: &str, n: u8, variant: u8) -> CommandCompanion {
    let col = |name| bc(table, name);
    let mut data = vec![
        (
            col("record_id"),
            Input::Value(Value::Text(format!(
                "CANARY_record_{n}_{}",
                u8::from(variant == 2)
            ))),
        ),
        (
            col("command_id"),
            Input::Value(Value::Text(if variant == 6 {
                "wrong_id".into()
            } else {
                hex(&[n; 16])
            })),
        ),
        (col("namespace"), Input::Value(Value::Text(hex(&ns().0)))),
        (
            col("subject"),
            Input::Value(Value::Text(if variant == 13 {
                "other_subject".into()
            } else {
                "CANARY_bundle_subject".into()
            })),
        ),
        (
            col("payload"),
            Input::Value(if variant == 1 {
                Value::Text("CANARY_changed_payload".into())
            } else if n.is_multiple_of(2) {
                Value::Text("CANARY_bundle_payload".into())
            } else {
                Value::Null(SqlType::Text)
            }),
        ),
        (col("recorded_at"), Input::Value(Value::I64(1793060000))),
        (
            col("sequence"),
            Input::Value(Value::I64(if variant == 3 { 1 } else { 0 })),
        ),
    ];
    match variant {
        4 => data.push((col("tenant"), Input::Value(Value::I64(22)))),
        5 => data.push((col("tenant"), Input::Value(Value::Null(SqlType::I64)))),
        7 => data.push((col("tenant"), Input::Value(Value::I64(11)))),
        8 => data.reverse(),
        9 => {
            data.pop();
        }
        10 => data.push((col("tenant"), Input::Value(Value::I32(11)))),
        20 => {
            for i in 0..80 {
                data.push((
                    bt(table).column(id(&format!("extra_{i}")), SqlType::Text, false),
                    Input::Value(Value::Text("x".repeat(4096))),
                ));
            }
        }
        14 => {
            data.push((col("tenant"), Input::Value(Value::I64(11))));
            data.push((col("tenant"), Input::Value(Value::I64(11))));
        }
        15 => {
            data.retain(|(c, _)| c != &col("payload"));
        }
        16 => {
            data.retain(|(c, _)| c != &col("payload"));
            data.push((col("payload"), Input::Value(Value::Text("x".repeat(4097)))));
        }
        _ => (),
    }
    CommandCompanion::application_supplied(
        Draft::insert(bt(table), data).freeze(),
        if variant == 11 {
            bt(table).column(id("tenant"), SqlType::I64, true)
        } else if variant == 17 {
            bc("bundle_items", "tenant")
        } else if variant == 18 {
            bt(table).column(id("tenant"), SqlType::I32, false)
        } else if variant == 19 {
            bt(table).column(id("missing_tenant"), SqlType::I64, false)
        } else {
            col("tenant")
        },
        col("record_id"),
        col("command_id"),
        col("namespace"),
        if variant == 12 {
            col("namespace")
        } else {
            col("subject")
        },
    )
}
fn bundle(n: u8, a: u8, o: u8) -> CommandBundle {
    CommandBundle::new(
        business(i64::from(n)),
        companion("bundle_audit", n, a),
        companion("bundle_outbox", n, o),
    )
}
#[test]
fn bundle_pure_mapping_canonical_and_output() {
    use std::error::Error;
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, 0, vec![]);
        let b = bundle(1, 0, 0);
        let p = c
            .prepare()
            .prepare_command_bundle(&b, &ns().0, &[1; 16])
            .unwrap();
        assert!(
            p.sensitive_records()
                .iter()
                .all(|p| p.sensitive_source().context().same_snapshot(&c))
        );
        let input = p.canonical_input(&ledger("bundle_ledger").0).unwrap();
        assert_eq!(&input.sensitive_bytes()[..4], b"QCC2");
        inspect_input(input.sensitive_bytes(), input.sensitive_bytes()).unwrap();
        for a in [1, 2, 3, 7, 8] {
            let other = bundle(1, a, 0);
            let q = c
                .prepare()
                .prepare_command_bundle(&other, &ns().0, &[1; 16])
                .unwrap()
                .canonical_input(&ledger("bundle_ledger").0)
                .unwrap();
            assert_ne!(input.sensitive_bytes(), q.sensitive_bytes());
            assert_eq!(
                inspect_input(q.sensitive_bytes(), input.sensitive_bytes()).unwrap_err(),
                ExecutionError::IdentityConflict
            );
        }
        for v in [4, 5, 6, 9, 10, 11, 12, 14, 15, 16, 17, 18, 19] {
            assert!(
                c.prepare()
                    .prepare_command_bundle(&bundle(1, v, 0), &ns().0, &[1; 16])
                    .is_err()
            );
        }
        let oversized = bundle(9, 20, 20);
        let large = context(d, 5, vec![])
            .prepare()
            .prepare_command_bundle(&oversized, &ns().0, &[9; 16])
            .unwrap();
        assert!(matches!(
            large.canonical_input(&ledger("bundle_ledger").0),
            Err(PolicyError::Budget)
        ));
        for v in [1, 2] {
            assert!(
                context(d, v, vec![])
                    .prepare()
                    .prepare_command_bundle(&b, &ns().0, &[1; 16])
                    .is_err()
            );
        }
        for policy in [
            PolicyDeclaration::Readonly(bc("bundle_audit", "payload")),
            PolicyDeclaration::Generated(bc("bundle_audit", "payload")),
            PolicyDeclaration::Pii(bc("bundle_audit", "payload")),
            PolicyDeclaration::RequiredFilter(bc("bundle_audit", "payload")),
            PolicyDeclaration::Readonly(bc("bundle_audit", "tenant")),
        ] {
            assert!(
                context(d, 0, vec![policy])
                    .prepare()
                    .prepare_command_bundle(&b, &ns().0, &[1; 16])
                    .is_err()
            );
        }
        // Mapping proof never changes normal context construction/preparation.
        let plain = companion("bundle_audit", 1, 7);
        let (original, _) = p.into_parts();
        assert!(original.same_bounded_input(&b).unwrap());
        // Access private input through a fresh pure preparation only in this module.
        let p = c
            .prepare()
            .prepare_command_bundle(&b, &ns().0, &[1; 16])
            .unwrap();
        let op = p.sensitive_records()[1].sensitive_source();
        assert!(
            c.prepare()
                .prepare_command_operation(op.original_operation())
                .is_err()
        );
        assert!(
            c.prepare()
                .prepare_operation(op.original_operation())
                .unwrap()
                .inspect()
                .risk()
                == Risk::High
        );
        for output in [
            format!("{b:?}"),
            format!("{plain:?}"),
            format!("{p:?}"),
            format!("{input:?}"),
        ] {
            assert!(!output.contains("CANARY"));
        }
        let r = saved_result(
            &[
                b"QCR1".as_slice(),
                &[if d == Dialect::Postgres { 1 } else { 2 }],
                &1u64.to_be_bytes(),
            ]
            .concat(),
            d,
        )
        .unwrap();
        let encoded = command_result_bytes(
            &r,
            Some(&[
                saved_result(&result_bytes(&r, d).unwrap(), d).unwrap(),
                saved_result(&result_bytes(&r, d).unwrap(), d).unwrap(),
            ]),
            d,
        )
        .unwrap();
        assert_eq!(encoded.len(), 29);
        assert!(saved_command_result(&encoded, d, true).is_ok());
        assert!(saved_result(&encoded, d).is_err());
        for i in [5, 13, 21] {
            let mut bad = encoded.clone();
            bad[i..i + 8].copy_from_slice(&2u64.to_be_bytes());
            assert_eq!(
                saved_command_result(&bad, d, true).unwrap_err(),
                ExecutionError::LedgerInvalid
            );
        }
        for n in [0, 2] {
            let r = ExecutionResult {
                rows: vec![],
                count: None,
                affected: Some(if d == Dialect::Postgres {
                    AffectedRows::PostgresMatched(n)
                } else {
                    AffectedRows::MysqlFoundRows(n)
                }),
                state: ResultState::Uncommitted,
            };
            assert_eq!(
                result_bytes(&r, d).unwrap_err(),
                ExecutionError::ExpectationMismatch
            );
        }
        assert!(ExecutionError::LedgerInvalid.source().is_none());
    }
}
macro_rules! suite {
($test:ident,$feature:literal,$session:ident,$borrowed:ident,$connection:ty,$options:ty,$dialect:expr,$dsn:literal,$ddl:literal,$ssl:expr)=>{
#[cfg(feature=$feature)] #[test] fn $test(){
    use sqlx::{Connection,ConnectOptions,Row};
    assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),Ok("quent_dispatch_test"));
    let options:$options=std::env::var($dsn).unwrap().parse().unwrap();
    let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    #[cfg(feature="output-tracing")] let capture=output_capture::Capture::default();
    #[cfg(feature="output-tracing")] let subscriber=tracing::subscriber::set_default(capture.clone());
    runtime.block_on(Box::pin(async {
        let d=$dialect;let c=context(d,0,vec![]);let namespace=ns();let l=ledger("bundle_ledger");
        let mut observer=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
        for statement in include_str!($ddl).split(';') {if let Some(start)=statement.find("CREATE TABLE"){sqlx::query(sqlx::AssertSqlSafe(statement[start..].to_owned())).execute(&mut observer).await.unwrap();}}
        let catalog=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_attribute a JOIN pg_class t ON t.oid=a.attrelid JOIN pg_namespace n ON n.oid=t.relnamespace LEFT JOIN pg_attrdef x ON x.adrelid=a.attrelid AND x.adnum=a.attnum WHERE n.nspname='quent_dispatch_test' AND t.relname IN ('bundle_items','bundle_ledger','bundle_audit','bundle_outbox') AND a.attnum>0 AND NOT a.attisdropped AND (a.attidentity<>'' OR a.attgenerated<>'' OR x.oid IS NOT NULL OR t.relrowsecurity)"}else{"SELECT COUNT(*) FROM information_schema.columns WHERE TABLE_SCHEMA='quent_dispatch_test' AND TABLE_NAME IN ('bundle_items','bundle_ledger','bundle_audit','bundle_outbox') AND (EXTRA<>'' OR COLUMN_DEFAULT IS NOT NULL)"};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(catalog.to_owned())).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);
        if d==Dialect::Mysql{assert_eq!(sqlx::query("SELECT COUNT(*) FROM information_schema.tables WHERE TABLE_SCHEMA='quent_dispatch_test' AND TABLE_NAME IN ('bundle_items','bundle_ledger','bundle_audit','bundle_outbox') AND ENGINE='InnoDB'").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),4);}
        let triggers=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_trigger g JOIN pg_class t ON t.oid=g.tgrelid JOIN pg_namespace n ON n.oid=t.relnamespace WHERE n.nspname='quent_dispatch_test' AND t.relname IN ('bundle_items','bundle_ledger','bundle_audit','bundle_outbox') AND NOT g.tgisinternal"}else{"SELECT COUNT(*) FROM information_schema.triggers WHERE TRIGGER_SCHEMA='quent_dispatch_test' AND EVENT_OBJECT_TABLE IN ('bundle_items','bundle_ledger','bundle_audit','bundle_outbox')"};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(triggers.to_owned())).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);
        let mut s=$session::connect(options.clone(),c.clone()).await.unwrap();
        let count=if d==Dialect::Postgres{"SELECT COUNT(*) FROM quent_dispatch_test.bundle_ledger WHERE identity=$1"}else{"SELECT COUNT(*) FROM quent_dispatch_test.bundle_ledger WHERE identity=?"};
        let select=if d==Dialect::Postgres{"SELECT input,result,version FROM quent_dispatch_test.bundle_ledger WHERE identity=$1"}else{"SELECT input,result,version FROM quent_dispatch_test.bundle_ledger WHERE identity=?"};
        let provider=Arc::new(super::super::authorized::Probe::default());let authorization=AuthorizationContext::register(c.clone(),"CANARY_bundle_provider".into(),1,b"CANARY_registration".to_vec(),provider.clone()).unwrap();let request=AuthorizationExecutionRequest::new();let broad=Draft::update(bt("bundle_items"),vec![(bc("bundle_items","amount"),Input::Value(Value::I64(1)))]).filter(Predicate::Compare(bc("bundle_items","tenant"),Compare::Eq,Value::I64(11))).freeze();let candidate=s.authorization_candidate(&authorization,&request,&broad,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();drop(s.acquire_authorized(&authorization,&request,candidate).await.unwrap());assert_eq!((provider.calls(),provider.responses()),(1,1));assert!(s.recorder.calls.is_empty());
        Box::pin(async {
        // Both implicit and explicit tenant, native NULL, actual record inspection
        // allocation/args/metadata and independent committed all-four-table observer.
        for n in [1u8,2] {
            let b=bundle(n,if n==1{0}else{7},if n==1{7}else{0});let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();
            let key=h.inner.key.sensitive_bytes().to_vec();let input=h.inner.input.sensitive_bytes().to_vec();
            let mut expected=vec![];for record in [&h.inner.operation,&h.inner.bundle.as_ref().unwrap().records[0],&h.inner.bundle.as_ref().unwrap().records[1]] {let source=record.prepared.sensitive_source();let final_source=source.final_source();expected.push((final_source.sql().as_ptr() as usize,final_source.sql().to_owned(),final_source.args().to_vec(),final_source.metadata().clone()));}
            assert!(!format!("{h:?}").contains("CANARY"));
            let result=s.execute_command_bundle(&c,&b,h).await.unwrap();assert_eq!(result.kind(),CommandResultKind::FirstCommit);assert_eq!(result.sensitive_companion_results().unwrap().len(),2);assert!(!format!("{result:?}{}",result.public_view().to_json()).contains("CANARY"));
            assert_eq!(s.recorder.responses,vec!["ledger_lookup","ledger_reserve","operation","operation","operation","ledger_result"]);
            assert_eq!(s.recorder.calls.len(),3);for (i,(address,sql,args,metadata)) in expected.into_iter().enumerate(){assert_eq!(s.recorder.sql_addresses[i],address);assert_eq!(s.recorder.calls[i],(sql,args));assert_eq!(s.recorder.metadata[i],metadata);}
            assert_eq!(s.recorder.events,vec!["begin","begun","ledger_lookup","ledger_reserve","command_ready","send","decoded","audit","send","decoded","outbox","send","decoded","ledger_result","commit","committed"]);
            let row=sqlx::query(sqlx::AssertSqlSafe(select.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap();assert_eq!(row.get::<Vec<u8>,_>(0),input);let saved=row.get::<Vec<u8>,_>(1);assert_eq!(saved.len(),29);assert!(saved_command_result(&saved,d,true).is_ok());assert_eq!(row.get::<i16,_>(2),2);
            for table in ["bundle_items","bundle_audit","bundle_outbox"] {assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{table}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),i64::from(n));}
            s.recorder=Recorder::default();let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();
            for lookup in [false,true] {let h=fresh.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();let r=if lookup{fresh.lookup_command_bundle(&c,&b,h).await.unwrap().unwrap()}else{fresh.execute_command_bundle(&c,&b,h).await.unwrap()};assert_eq!(r.kind(),CommandResultKind::Replayed);assert!(fresh.recorder.calls.is_empty());assert!(!fresh.recorder.events.contains(&"ledger_reserve"));}
            println!("PR3 {:?} commit/replay case={} business1 audit1 outbox1 ledger_lookup1 reserve1 result1 commitACK; fresh execute/lookup allrecord0",d,n);
        }
        }).await;
        Box::pin(async {
        // Full-input conflicts including plain/bundle in SAME identity space.
        for (a,o) in [(1,0),(2,0),(3,0),(7,0),(8,0),(0,1),(0,2),(0,3),(0,8)] {let b=bundle(1,a,o);let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();let e=s.execute_command_bundle(&c,&b,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::IdentityConflict);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();}
        let op=business(1);let h=s.validate_command(&c,&op,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert_eq!(s.execute_command(&c,&op,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        let op=business(3);let h=s.validate_command(&c,&op,&namespace,CommandId::new([3;16]),&l,MAX_TTL).unwrap();s.execute_command(&c,&op,h).await.unwrap();s.recorder=Recorder::default();let b=bundle(3,0,0);let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([3;16]),&l,MAX_TTL).unwrap();assert_eq!(s.lookup_command_bundle(&c,&b,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        }).await;
        Box::pin(async {
        // Eligibility/authority/consumed future preflight, no Tx/provider/send.
        for v in [4,5,6,9,10,11,12,14,15,16,17,18,19] {let b=bundle(9,v,0);assert!(s.validate_command_bundle(&c,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());assert!(s.recorder.events.is_empty());}
        let b=bundle(9,0,0);let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).unwrap();let different=bundle(9,1,0);let e=s.execute_command_bundle(&c,&different,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::OperationMismatch);assert_eq!(e.disposition(),AtomicDisposition::NotStarted);assert!(s.recorder.events.is_empty());
        let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).unwrap();drop(s.execute_command_bundle(&c,&b,h));assert!(s.recorder.events.is_empty());
        let b=bundle(9,0,0);let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).unwrap();let mut external=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut tx=external.begin().await.unwrap();{
            let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();assert_eq!(borrowed.validate_command_bundle(&c,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).unwrap_err(),ExecutionError::Unsupported);let e=borrowed.execute_command_bundle(&c,&b,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Unsupported);assert_eq!(e.disposition(),AtomicDisposition::NotStarted);assert!(borrowed.recorder.events.is_empty());
        }tx.rollback().await.unwrap();
        for variant in [2,3,4]{let current=context(d,variant,vec![]);let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).unwrap();let e=s.execute_command_bundle(&current,&b,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::ContextMismatch);assert_eq!(e.disposition(),AtomicDisposition::NotStarted);assert!(s.recorder.events.is_empty());}
        let large=context(d,5,vec![]);let reject=$session::connect(options.clone(),large.clone()).await.unwrap();let b=bundle(9,20,20);assert_eq!(reject.validate_command_bundle(&large,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).unwrap_err(),ExecutionError::Policy(PolicyError::Budget));assert!(reject.recorder.events.is_empty());
        }).await;
        Box::pin(async {
        // Role/table placement remains part of identity; source/current remains
        // independent of the saved mapping even for a newly authenticated subject.
        let swapped=CommandBundle::new(business(1),companion("bundle_outbox",1,0),companion("bundle_audit",1,0));let h=s.validate_command_bundle(&c,&swapped,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert_eq!(s.lookup_command_bundle(&c,&swapped,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        let b=bundle(9,0,0);let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([9;16]),&l,Duration::from_millis(1)).unwrap();tokio::time::sleep(Duration::from_millis(3)).await;let e=s.execute_command_bundle(&c,&b,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::NotStarted);assert!(s.recorder.events.is_empty());
        for protected in [PolicyDeclaration::Readonly(bc("bundle_audit","payload")),PolicyDeclaration::Generated(bc("bundle_outbox","payload")),PolicyDeclaration::Pii(bc("bundle_outbox","payload")),PolicyDeclaration::RequiredFilter(bc("bundle_audit","payload")),PolicyDeclaration::Readonly(bc("bundle_audit","tenant"))] {let current=context(d,0,vec![protected]);let reject=$session::connect(options.clone(),current.clone()).await.unwrap();assert!(reject.validate_command_bundle(&current,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());assert!(reject.recorder.events.is_empty());}
        for variant in [1,2] {let current=context(d,variant,vec![]);let reject=$session::connect(options.clone(),current.clone()).await.unwrap();assert!(reject.validate_command_bundle(&current,&b,&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());assert!(reject.recorder.events.is_empty());}
        for table in ["bundle_items","bundle_audit","bundle_outbox"]{assert!(s.validate_command_bundle(&c,&b,&namespace,CommandId::new([9;16]),&ledger(table),MAX_TTL).is_err());assert!(s.recorder.events.is_empty());}
        let duplicate=CommandBundle::new(business(9),companion("bundle_audit",9,0),companion("bundle_audit",9,0));assert!(s.validate_command_bundle(&c,&duplicate,&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());assert!(s.recorder.events.is_empty());
        let new_subject=context(d,3,vec![]);let mut fresh=$session::connect(options.clone(),new_subject.clone()).await.unwrap();let b=bundle(1,13,13);let h=fresh.validate_command_bundle(&new_subject,&b,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert_eq!(fresh.lookup_command_bundle(&new_subject,&b,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(fresh.recorder.calls.is_empty());
        // An original version0 write replays after current physical version1;
        // explicitly submitting stale version0 under a NEW ID conflicts/rolls back.
        for n in [70u8,71] {let op=Draft::update(bt("bundle_items"),vec![(bc("bundle_items","flag"),Input::Value(Value::Bool(true)))]).filter(Predicate::Compare(bc("bundle_items","id"),Compare::Eq,Value::I64(2))).optimistic_version(bc("bundle_items","amount"),Value::I64(0)).unwrap().freeze();let b=CommandBundle::new(op,companion("bundle_audit",n,0),companion("bundle_outbox",n,0));let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();if n==70{let mut expected=vec![];for r in [&h.inner.operation,&h.inner.bundle.as_ref().unwrap().records[0],&h.inner.bundle.as_ref().unwrap().records[1]]{let src=r.prepared.sensitive_source();let f=src.final_source();expected.push((f.sql().as_ptr() as usize,f.sql().to_owned(),f.args().to_vec(),f.metadata().clone()));}assert_eq!(s.execute_command_bundle(&c,&b,h).await.unwrap().kind(),CommandResultKind::FirstCommit);assert_eq!(s.recorder.calls.len(),3);for(i,(address,sql,args,metadata))in expected.into_iter().enumerate(){assert_eq!(s.recorder.sql_addresses[i],address);assert_eq!(s.recorder.calls[i],(sql,args));assert_eq!(s.recorder.metadata[i],metadata);}s.recorder=Recorder::default();let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();assert_eq!(s.execute_command_bundle(&c,&b,h).await.unwrap().kind(),CommandResultKind::Replayed);assert!(s.recorder.calls.is_empty());}else{let e=s.execute_command_bundle(&c,&b,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Conflict);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(s.recorder.calls.len(),1);}s.recorder=Recorder::default();}
        // Strict bundle saved-result decoder: all three expected counts matter.
        let b=bundle(1,0,7);let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();let key=h.inner.key.sensitive_bytes().to_vec();drop(h);let row=sqlx::query(sqlx::AssertSqlSafe(select.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap();let saved=row.get::<Vec<u8>,_>(1);
        let update=if d==Dialect::Postgres{"UPDATE quent_dispatch_test.bundle_ledger SET result=$1 WHERE identity=$2"}else{"UPDATE quent_dispatch_test.bundle_ledger SET result=? WHERE identity=?"};
        for (index,value) in [(5,0u64),(5,2),(13,0),(13,2),(21,0),(21,2)] {let mut bad=saved.clone();bad[index..index+8].copy_from_slice(&value.to_be_bytes());sqlx::query(sqlx::AssertSqlSafe(update.to_owned())).bind(bad).bind(&key).execute(&mut observer).await.unwrap();let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();let e=s.lookup_command_bundle(&c,&b,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerInvalid);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();}
        sqlx::query(sqlx::AssertSqlSafe(update.to_owned())).bind(saved).bind(&key).execute(&mut observer).await.unwrap();let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert_eq!(s.lookup_command_bundle(&c,&b,h).await.unwrap().unwrap().kind(),CommandResultKind::Replayed);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        }).await;
        Box::pin(async {
        // Fixed failures: business duplicate, audit duplicate, outbox duplicate.
        // Audit/outbox seed statements are setup, never production sends.
        for (n,stage) in [(1u8,1usize),(10,2),(11,3)] {
            if stage>1 {let table=if stage==2{"bundle_audit"}else{"bundle_outbox"};sqlx::query(sqlx::AssertSqlSafe(format!("INSERT INTO quent_dispatch_test.{table}(tenant,record_id,command_id,namespace,subject,payload,recorded_at,sequence) VALUES(11,'CANARY_record_{n}_0','setup','setup','setup',NULL,0,0)"))).execute(&mut observer).await.unwrap();}
            let b=if stage==1{CommandBundle::new(business(1),companion("bundle_audit",15,0),companion("bundle_outbox",15,0))}else{bundle(n,0,0)};
            let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([if stage==1{15}else{n};16]),&l,MAX_TTL);
            let h=h.unwrap();let key=h.inner.key.sensitive_bytes().to_vec();let e=s.execute_command_bundle(&c,&b,h).await.unwrap_err();assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(s.recorder.calls.len(),stage);assert!(!s.recorder.events.contains(&"ledger_result"));assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);
            assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.bundle_items WHERE id={n}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),i64::from(stage==1));
            for (role,table) in [(2,"bundle_audit"),(3,"bundle_outbox")] {let record=if stage==1{15}else{n};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{table} WHERE record_id='CANARY_record_{record}_0'"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),i64::from(stage==role));}s.recorder=Recorder::default();
        }
        }).await;
        Box::pin(async {
        // Ledger result save CHECK failure after ALL three successful writes.
        let fail_ddl=if d==Dialect::Postgres{"CREATE TABLE quent_dispatch_test.bundle_ledger_fail(identity BYTEA PRIMARY KEY,input BYTEA NOT NULL,result BYTEA NULL CHECK(result IS NULL),version SMALLINT NOT NULL)"}else{"CREATE TABLE quent_dispatch_test.bundle_ledger_fail(identity VARBINARY(2048) PRIMARY KEY,input LONGBLOB NOT NULL,result VARBINARY(29) NULL CHECK(result IS NULL),version SMALLINT NOT NULL) ENGINE=InnoDB"};
        sqlx::query(sqlx::AssertSqlSafe(fail_ddl.to_owned())).execute(&mut observer).await.unwrap();
        let b=bundle(20,0,0);let h=s.validate_command_bundle(&c,&b,&namespace,CommandId::new([20;16]),&ledger("bundle_ledger_fail"),MAX_TTL).unwrap();let e=s.execute_command_bundle(&c,&b,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerFailed);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(s.recorder.calls.len(),3);assert!(s.recorder.events.contains(&"ledger_result"));assert!(!s.recorder.events.contains(&"commit"));
        for table in ["bundle_items","bundle_audit","bundle_outbox","bundle_ledger_fail"] {let filter=if table=="bundle_items"{" WHERE id=20"}else if table=="bundle_ledger_fail"{""}else{" WHERE record_id='CANARY_record_20_0'"};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{table}{filter}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);}
        sqlx::query("DROP TABLE quent_dispatch_test.bundle_ledger_fail").execute(&mut observer).await.unwrap();s.recorder=Recorder::default();
        }).await;
        Box::pin(async {
        for (n,fault,committed) in [(90u8,AtomicFault::Begin,false),(91,AtomicFault::Commit,false),(92,AtomicFault::CommitReplyLost,true)] {
            let b=bundle(n,0,0);let mut faulted=$session::connect(options.clone(),c.clone()).await.unwrap();faulted.atomic_fault=Some(fault);let h=faulted.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();let e=faulted.execute_command_bundle(&c,&b,h).await.unwrap_err();assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown);assert!(!faulted.is_available());assert_eq!(faulted.recorder.calls.len(),if fault==AtomicFault::Begin{0}else{3});drop(faulted);
            let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let h=fresh.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();assert_eq!(fresh.lookup_command_bundle(&c,&b,h).await.unwrap().is_some(),committed);assert!(fresh.recorder.calls.is_empty());
            for table in ["bundle_items","bundle_audit","bundle_outbox"]{let filter=if table=="bundle_items"{format!("id={n}")}else{format!("record_id='CANARY_record_{n}_0'")};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{table} WHERE {filter}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),i64::from(committed));}
        }
        }).await;
        Box::pin(async {
        // Observe actual server INSERT waits before cancel/timeout. Uncommitted
        // setup rows are released only after the command future is consumed.
        for (index,table) in ["bundle_items","bundle_audit","bundle_outbox"].into_iter().enumerate() {
            for mode in 0..3u8 {
                let n=30+index as u8*3+mode;let b=bundle(n,0,0);
                let mut locker=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut lock=locker.begin().await.unwrap();
                let seed=if table=="bundle_items"{format!("INSERT INTO quent_dispatch_test.bundle_items(tenant,id,amount,flag,note) VALUES(11,{n},0,FALSE,NULL)")}else{format!("INSERT INTO quent_dispatch_test.{table}(tenant,record_id,command_id,namespace,subject,payload,recorded_at,sequence) VALUES(11,'CANARY_record_{n}_0','setup','setup','setup',NULL,0,0)")};sqlx::query(sqlx::AssertSqlSafe(seed)).execute(&mut *lock).await.unwrap();
                let mut canceled=$session::connect(options.clone(),c.clone()).await.unwrap();let connection_id=canceled.test_connection_id().await;
                let h=canceled.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,if mode==2{Duration::from_millis(250)}else{MAX_TTL}).unwrap();let key=h.inner.key.sensitive_bytes().to_vec();let trace=canceled.recorder.live.clone();let mut f=Box::pin(canceled.execute_command_bundle(&c,&b,h));
                let waiting=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user AND wait_event_type='Lock'"}else{"SELECT COUNT(*) FROM performance_schema.data_lock_waits w JOIN performance_schema.threads t ON t.THREAD_ID=w.REQUESTING_THREAD_ID WHERE t.PROCESSLIST_ID=?"};
                tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(f.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(connection_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
                assert_eq!(trace.lock().unwrap().iter().filter(|e|**e=="send").count(),index+1);
                if mode==1{assert!(tokio::time::timeout(Duration::from_millis(1),f.as_mut()).await.is_err());}
                if mode==2{tokio::time::sleep(Duration::from_millis(270)).await;lock.rollback().await.unwrap();let e=f.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);}else{drop(f);assert!(!canceled.is_available());drop(canceled);lock.rollback().await.unwrap();}
                assert!(!trace.lock().unwrap().contains(&"committed"));assert_eq!(trace.lock().unwrap().contains(&"rolled_back"),mode==2);
                let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let h=fresh.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();assert!(fresh.lookup_command_bundle(&c,&b,h).await.unwrap().is_none());assert!(fresh.recorder.calls.is_empty());
                assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);
                for t in ["bundle_items","bundle_audit","bundle_outbox"] {let filter=if t=="bundle_items"{format!("id={n}")}else{format!("record_id='CANARY_record_{n}_0'")};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{t} WHERE {filter}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);}
                println!("PR3 {:?} actual INSERT wait table_role={} mode={} submissions={} commit0 rollbackACK={} observer_allfour0 fresh_lookup_none",d,index,mode,index+1,mode==2);
            }
        }
        }).await;
        Box::pin(async {
        // Finite test-only protocol awaits distinguish cancellation before result
        // save from after save/before commit; not physical communication faults.
        for after in [false,true] {for mode in 0..3u8 {
            let n=100+u8::from(after)*3+mode;let b=bundle(n,0,0);let mut paused=$session::connect(options.clone(),c.clone()).await.unwrap();let barrier=Arc::new(tokio::sync::Notify::new());paused.command_barrier=Some((after,barrier.clone()));let trace=paused.recorder.live.clone();let h=paused.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,if mode==2{Duration::from_millis(150)}else{MAX_TTL}).unwrap();let mut f=Box::pin(paused.execute_command_bundle(&c,&b,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(f.as_mut()).await.is_pending());if trace.lock().unwrap().contains(&if after{"saved_wait"}else{"result_wait"}){break;}tokio::task::yield_now().await;}}).await.unwrap();assert_eq!(trace.lock().unwrap().iter().filter(|e|**e=="send").count(),3);assert_eq!(trace.lock().unwrap().contains(&"ledger_result"),after);
            if mode==1{assert!(tokio::time::timeout(Duration::from_millis(1),f.as_mut()).await.is_err());}
            if mode==2{tokio::time::sleep(Duration::from_millis(170)).await;barrier.notify_one();let e=f.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);}else{drop(f);assert!(!paused.is_available());drop(paused);}
            assert!(!trace.lock().unwrap().contains(&"committed"));assert_eq!(trace.lock().unwrap().contains(&"rolled_back"),mode==2);let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let h=fresh.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();assert!(fresh.lookup_command_bundle(&c,&b,h).await.unwrap().is_none());assert!(fresh.recorder.calls.is_empty());
            for table in ["bundle_items","bundle_audit","bundle_outbox"]{let filter=if table=="bundle_items"{format!("id={n}")}else{format!("record_id='CANARY_record_{n}_0'")};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{table} WHERE {filter}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);}
            println!("PR3 {:?} result-await seam case={} after_save={} mode={} record_sends3 result_send={} commit0 rollbackACK={}",d,n,after,mode,after,mode==2);
        }}
        }).await;
        Box::pin(async {
        // Server-observed reservation races retain PR2 option A, including
        // different-input followers already started before an ACK rollback.
        let waiting=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user AND wait_event_type='Lock'"}else{"SELECT COUNT(*) FROM performance_schema.data_lock_waits w JOIN performance_schema.threads t ON t.THREAD_ID=w.REQUESTING_THREAD_ID WHERE t.PROCESSLIST_ID=?"};
        for rollback in [false,true] {for different in [false,true] {
            let n=80+u8::from(rollback)*2+u8::from(different);let b=bundle(n,0,0);let other=bundle(n,u8::from(different),0);
            let mut lead=$session::connect(options.clone(),c.clone()).await.unwrap();let barrier=Arc::new(tokio::sync::Notify::new());lead.final_barrier=Some(barrier.clone());let h=lead.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,if rollback{Duration::from_millis(300)}else{MAX_TTL}).unwrap();let key=h.inner.key.sensitive_bytes().to_vec();let trace=lead.recorder.live.clone();let mut first=Box::pin(lead.execute_command_bundle(&c,&b,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(first.as_mut()).await.is_pending());if trace.lock().unwrap().contains(&"command_ready"){break;}tokio::task::yield_now().await;}}).await.unwrap();
            let mut follow=$session::connect(options.clone(),c.clone()).await.unwrap();let connection_id=follow.test_connection_id().await;let h=follow.validate_command_bundle(&c,&other,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();let mut second=Box::pin(follow.execute_command_bundle(&c,&other,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(second.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(connection_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
            let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let h=fresh.validate_command_bundle(&c,&other,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();assert!(fresh.lookup_command_bundle(&c,&other,h).await.unwrap().is_none());assert!(fresh.recorder.calls.is_empty());
            if rollback{tokio::time::sleep(Duration::from_millis(320)).await;}barrier.notify_one();let first=first.await;
            if rollback {let e=first.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(second.await.unwrap().kind(),CommandResultKind::FirstCommit);assert!(lead.recorder.calls.is_empty());assert_eq!(follow.recorder.calls.len(),3);}else{assert_eq!(first.unwrap().kind(),CommandResultKind::FirstCommit);let e=second.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerFailed);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(lead.recorder.calls.len(),3);assert!(follow.recorder.calls.is_empty());}
            let h=fresh.validate_command_bundle(&c,&other,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();let r=fresh.lookup_command_bundle(&c,&other,h).await;if !rollback && different{assert_eq!(r.unwrap_err().cause(),ExecutionError::IdentityConflict);}else{assert_eq!(r.unwrap().unwrap().kind(),CommandResultKind::Replayed);}assert!(fresh.recorder.calls.is_empty());assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),1);
            for table in ["bundle_items","bundle_audit","bundle_outbox"] {let filter=if table=="bundle_items"{format!("id={n}")}else{format!("record_id='CANARY_record_{n}_0'")};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{table} WHERE {filter}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),1);}
            println!("PR3 {:?} reservation race={} rollback={} different={} leader_allrecord={} follower_allrecord={} observer_allfour1 fresh_allrecord0",d,n,rollback,different,if rollback{0}else{3},if rollback{3}else{0});
        }}
        for different in [false,true] {
            let n=84+u8::from(different);let b=bundle(n,0,0);let other=bundle(n,u8::from(different),0);
            let mut locker=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut lock=locker.begin().await.unwrap();sqlx::query(sqlx::AssertSqlSafe(format!("INSERT INTO quent_dispatch_test.bundle_audit(tenant,record_id,command_id,namespace,subject,payload,recorded_at,sequence) VALUES(11,'CANARY_record_{n}_0','setup','setup','setup',NULL,0,0)"))).execute(&mut *lock).await.unwrap();
            let mut lead=$session::connect(options.clone(),c.clone()).await.unwrap();let lead_id=lead.test_connection_id().await;let h=lead.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,Duration::from_millis(500)).unwrap();let key=h.inner.key.sensitive_bytes().to_vec();let mut first=Box::pin(lead.execute_command_bundle(&c,&b,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(first.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(lead_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
            let mut follow=$session::connect(options.clone(),c.clone()).await.unwrap();let follow_id=follow.test_connection_id().await;let h=follow.validate_command_bundle(&c,&other,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();let mut second=Box::pin(follow.execute_command_bundle(&c,&other,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(second.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(follow_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
            tokio::time::sleep(Duration::from_millis(520)).await;lock.rollback().await.unwrap();let e=first.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(second.await.unwrap().kind(),CommandResultKind::FirstCommit);assert_eq!(lead.recorder.calls.len(),2);assert_eq!(follow.recorder.calls.len(),3);assert!(!lead.recorder.events.contains(&"outbox"));assert!(follow.recorder.events.contains(&"committed"));assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),1);
            let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let h=fresh.validate_command_bundle(&c,&other,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();assert_eq!(fresh.lookup_command_bundle(&c,&other,h).await.unwrap().unwrap().kind(),CommandResultKind::Replayed);assert!(fresh.recorder.calls.is_empty());for table in ["bundle_items","bundle_audit","bundle_outbox"]{let filter=if table=="bundle_items"{format!("id={n}")}else{format!("record_id='CANARY_record_{n}_0'")};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{table} WHERE {filter}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),1);}
            println!("PR3 {:?} after-audit-wait race={} different={} leader business1/audit1/outbox0 rollbackACK follower business1/audit1/outbox1 commitACK retained_ledger1",d,n,different);
        }
        }).await;
        Box::pin(async {
        // Real COMMIT protocol reply-loss fixture, including before-send and ACK controls.
        for (i,mode) in [super::super::command_network::Mode::BeforeCommit,super::super::command_network::Mode::LoseReply,super::super::command_network::Mode::Pass].into_iter().enumerate() {
            use std::sync::atomic::Ordering;
            let n=120+i as u8;let b=bundle(n,0,0);let target=std::net::SocketAddr::new(options.get_host().parse().unwrap(),options.get_port());let proxy=super::super::command_network::Proxy::new(target,d==Dialect::Postgres,mode);
            let mut network=$session::connect(options.clone().host("127.0.0.1").port(proxy.address.port()).ssl_mode($ssl),c.clone()).await.unwrap();
            let h=network.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();let key=h.inner.key.sensitive_bytes().to_vec();let input=h.inner.input.sensitive_bytes().to_vec();let r=tokio::time::timeout(Duration::from_secs(10),network.execute_command_bundle(&c,&b,h)).await.unwrap();
            if i==2{assert_eq!(r.unwrap().kind(),CommandResultKind::FirstCommit);}else{let e=r.unwrap_err();assert_eq!(e.cause(),ExecutionError::OutcomeUnknown);assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown);assert!(!network.is_available());}
            assert_eq!(network.recorder.calls.len(),3);assert_eq!(proxy.sent.load(Ordering::SeqCst),i!=0);assert_eq!(proxy.reply.load(Ordering::SeqCst),i!=0);drop(network);drop(proxy);
            let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let h=fresh.validate_command_bundle(&c,&b,&namespace,CommandId::new([n;16]),&l,MAX_TTL).unwrap();let r=fresh.lookup_command_bundle(&c,&b,h).await.unwrap();assert!(fresh.recorder.calls.is_empty());assert_eq!(r.is_some(),i!=0);
            assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),i64::from(i!=0));
            for table in ["bundle_items","bundle_audit","bundle_outbox"] {let filter=if table=="bundle_items"{format!("id={n}")}else{format!("record_id='CANARY_record_{n}_0'")};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(format!("SELECT COUNT(*) FROM quent_dispatch_test.{table} WHERE {filter}"))).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),i64::from(i!=0));}
            if i!=0{let row=sqlx::query(sqlx::AssertSqlSafe(select.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap();assert_eq!(row.get::<Vec<u8>,_>(0),input);assert!(saved_command_result(&row.get::<Vec<u8>,_>(1),d,true).is_ok());}
            println!("PR3 {:?} network case={} business1 audit1 outbox1 forwarded={} real_server_ack={} client_ack={} fresh_lookup_allrecord0 observer_allfour={}",d,n,i!=0,i!=0,i==2,i!=0);
        }
        }).await;
        assert_eq!((provider.calls(),provider.responses()),(1,1));
        for table in ["bundle_outbox","bundle_audit","bundle_ledger","bundle_items"]{sqlx::query(sqlx::AssertSqlSafe(format!("DROP TABLE quent_dispatch_test.{table}"))).execute(&mut observer).await.unwrap();}
    }));
    #[cfg(feature="output-tracing")] {drop(subscriber);assert!(capture.0.lock().unwrap().is_empty());let positive=output_capture::Capture::default();tracing::subscriber::with_default(positive.clone(),||tracing::warn!("CANARY_bundle_detector"));assert!(positive.0.lock().unwrap().iter().any(|s|s.contains("CANARY_bundle_detector")));}
}}
}
suite!(
    live_bundle_postgres,
    "postgres",
    PostgresSession,
    BorrowedPostgresTransaction,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    "../../../../../../fixtures/command-bundle/postgres.sql",
    sqlx::postgres::PgSslMode::Disable
);
suite!(
    live_bundle_mysql,
    "mysql",
    MysqlSession,
    BorrowedMysqlTransaction,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "../../../../../../fixtures/command-bundle/mysql.sql",
    sqlx::mysql::MySqlSslMode::Disabled
);
