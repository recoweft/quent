use super::*;
fn ct() -> Table {
    Table::new(vec![id("quent_dispatch_test"), id("command_items")], None).unwrap()
}
fn cc(n: &str) -> Column {
    ct().column(
        id(n),
        match n {
            "flag" => SqlType::Bool,
            "note" => SqlType::Text,
            _ => SqlType::I64,
        },
        n == "note",
    )
}
fn command_context(
    d: Dialect,
    tenant: i64,
    variant: u8,
    extras: Vec<PolicyDeclaration>,
) -> DatabaseContext {
    let b = DatabaseBinding::application_supplied(
        DatabaseIdentity::new(if variant == 1 {
            "other_target"
        } else {
            "command_target"
        })
        .unwrap(),
        TargetAddress::new("command_route").unwrap(),
        id("quent_dispatch_test"),
        d,
    );
    let mut columns = ["tenant", "id", "amount", "flag", "note"].map(cc).to_vec();
    if variant == 2 {
        columns.reverse();
    }
    let f = ScopeFacts::application_asserted(
        ct(),
        d,
        columns,
        vec![Key::Primary(vec![cc("tenant"), cc("id")])],
        if variant == 3 {
            TableContext::Unsupported
        } else {
            TableContext::OrdinaryBaseTableWithoutEffects
        },
    )
    .unwrap();
    let g = Generation::new(1).unwrap();
    let mut policies = vec![PolicyDeclaration::Tenant(cc("tenant"))];
    policies.extend(extras);
    DatabaseContext::new(
        Settings::new(
            b.clone(),
            SettingsIdentity::new("settings").unwrap(),
            g,
            PolicySet::new(
                b.clone(),
                PolicyIdentity::new("policies").unwrap(),
                g,
                policies,
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
                    vec![f],
                )
                .unwrap(),
            ),
            if variant == 5 {
                None
            } else {
                Some(
                    SubjectIdentity::new(if variant == 4 {
                        "CANARY_other_subject"
                    } else {
                        "CANARY_command_subject"
                    })
                    .unwrap(),
                )
            },
            Some(
                TenantSnapshot::application_supplied(b, cc("tenant"), Value::I64(tenant)).unwrap(),
            ),
        )
        .unwrap(),
    )
    .unwrap()
}
// Dedicated native-width / deliberately false key fixtures share the production
// context constructors. No capability bypass or alternative business executor.
fn extra_context(d: Dialect, table: Table, columns: Vec<Column>) -> DatabaseContext {
    let b = DatabaseBinding::application_supplied(
        DatabaseIdentity::new("command_target").unwrap(),
        TargetAddress::new("command_route").unwrap(),
        id("quent_dispatch_test"),
        d,
    );
    let g = Generation::new(1).unwrap();
    let tenant = columns[0].clone();
    let key = columns[1].clone();
    let facts = ScopeFacts::application_asserted(
        table,
        d,
        columns,
        vec![Key::Primary(vec![tenant.clone(), key])],
        TableContext::OrdinaryBaseTableWithoutEffects,
    )
    .unwrap();
    DatabaseContext::new(
        Settings::new(
            b.clone(),
            SettingsIdentity::new("settings").unwrap(),
            g,
            PolicySet::new(
                b.clone(),
                PolicyIdentity::new("policies").unwrap(),
                g,
                vec![PolicyDeclaration::Tenant(tenant.clone())],
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
                    vec![facts],
                )
                .unwrap(),
            ),
            Some(SubjectIdentity::new("CANARY_command_subject").unwrap()),
            Some(TenantSnapshot::application_supplied(b, tenant, Value::I64(11)).unwrap()),
        )
        .unwrap(),
    )
    .unwrap()
}
fn ci(n: i64, note: Value, explicit: bool) -> Operation {
    let mut pairs = vec![
        (cc("id"), Input::Value(Value::I64(n))),
        (cc("amount"), Input::Value(Value::I64(0))),
        (cc("flag"), Input::Value(Value::Bool(false))),
        (cc("note"), Input::Value(note)),
    ];
    if explicit {
        pairs.push((cc("tenant"), Input::Value(Value::I64(11))));
    }
    Draft::insert(ct(), pairs).freeze()
}
fn cu(n: i64, amount: i64) -> Operation {
    Draft::update(ct(), vec![(cc("amount"), Input::Value(Value::I64(amount)))])
        .filter(Predicate::Compare(cc("id"), Compare::Eq, Value::I64(n)))
        .freeze()
}
fn ledger(name: &str) -> CommandLedger {
    CommandLedger::application_supplied(
        Table::new(vec![id("quent_dispatch_test"), id(name)], None).unwrap(),
    )
    .unwrap()
}
fn ns() -> CommandNamespace {
    CommandNamespace::new(b"CANARY_command_namespace".to_vec()).unwrap()
}
#[test]
fn command_storage_decode_and_fixed_output() {
    use std::error::Error;
    let c = command_context(Dialect::Postgres, 11, 0, vec![]);
    let p = c
        .prepare()
        .prepare_command_operation(&ci(1, Value::Null(SqlType::Text), false))
        .unwrap();
    let canonical = p
        .sensitive_source()
        .canonical_command_input(&ledger("command_ledger").0)
        .unwrap();
    let bytes = canonical.sensitive_bytes();
    assert_eq!(inspect_input(bytes, bytes), Ok(()));
    for bad in [
        b"QCC2".to_vec(),
        bytes[..bytes.len() - 1].to_vec(),
        [bytes, &[255, 0, 0, 0, 0]].concat(),
        [bytes, &[9, 0, 0, 0, 1, 2]].concat(),
        vec![0; 1_048_577],
    ] {
        assert_eq!(
            inspect_input(&bad, bytes),
            Err(ExecutionError::LedgerInvalid)
        );
    }
    for tail in [
        vec![1, 0, 0, 0, 2, 0, 0],
        vec![10, 0, 0, 0, 1, 255],
        vec![12, 0, 0, 0, 1, 255],
    ] {
        let mut bad = b"QCC1".to_vec();
        bad.extend(tail);
        assert_eq!(
            inspect_input(&bad, bytes),
            Err(ExecutionError::LedgerInvalid)
        );
    }
    let mut deep = vec![];
    for _ in 0..34 {
        let mut next = vec![128];
        next.extend_from_slice(&(deep.len() as u32).to_be_bytes());
        next.extend(deep);
        deep = next;
    }
    let mut bad = b"QCC1".to_vec();
    bad.extend(deep);
    assert_eq!(
        inspect_input(&bad, bytes),
        Err(ExecutionError::LedgerInvalid)
    );
    let mut many = b"QCC1".to_vec();
    for _ in 0..131073 {
        many.extend_from_slice(&[128, 0, 0, 0, 0]);
    }
    assert_eq!(
        inspect_input(&many, bytes),
        Err(ExecutionError::LedgerInvalid)
    );
    let mut extra = bytes.to_vec();
    extra.extend_from_slice(&[8, 0, 0, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(
        inspect_input(&extra, bytes),
        Err(ExecutionError::IdentityConflict)
    );
    let r = ExecutionResult {
        rows: vec![],
        count: None,
        affected: Some(AffectedRows::PostgresMatched(1)),
        state: ResultState::Uncommitted,
    };
    let saved = result_bytes(&r, Dialect::Postgres).unwrap();
    assert_eq!(
        saved_result(&saved, Dialect::Postgres)
            .unwrap()
            .affected_rows(),
        Some(AffectedRows::PostgresMatched(1))
    );
    for bad in [
        saved[..12].to_vec(),
        [saved.as_slice(), &[0]].concat(),
        b"QCR2\x01\0\0\0\0\0\0\0\x01".to_vec(),
    ] {
        assert_eq!(
            saved_result(&bad, Dialect::Postgres).unwrap_err(),
            ExecutionError::LedgerInvalid
        );
    }
    assert_eq!(
        saved_result(&saved, Dialect::Mysql).unwrap_err(),
        ExecutionError::LedgerInvalid
    );
    for e in [
        ExecutionError::IdentityConflict,
        ExecutionError::LedgerInvalid,
        ExecutionError::LedgerFailed,
        ExecutionError::ReservationUnresolved,
    ] {
        assert!(e.source().is_none());
        assert!(!format!("{e:?}{e}{}", e.public_view()).contains("CANARY"));
    }
    assert_eq!(
        format!(
            "{:?}{:?}{:?}",
            ns(),
            CommandId::new([0; 16]),
            ledger("command_ledger")
        ),
        "CommandNamespace([REDACTED])CommandId([REDACTED])CommandLedger([REDACTED])"
    );
}

macro_rules! borrowed_command_cases {
    (PostgresSession,$d:ident,$tx:ident,$c:ident,$op:ident,$ns:ident,$l:ident,$h:ident) => {{
        let mut b = BorrowedPostgresTransaction::new(&mut $tx, $c.clone()).unwrap();
        assert_eq!(
            b.validate_command(&$c, &$op, &$ns, CommandId::new([51; 16]), &$l, MAX_TTL)
                .unwrap_err(),
            ExecutionError::Unsupported
        );
        assert_eq!(
            b.execute_command(&$c, &$op, $h).await.unwrap_err(),
            AtomicFailure {
                cause: ExecutionError::Unsupported,
                disposition: AtomicDisposition::NotStarted
            }
        );
        assert!(b.recorder.events.is_empty());
    }};
    (MysqlSession,$d:ident,$tx:ident,$c:ident,$op:ident,$ns:ident,$l:ident,$h:ident) => {{
        let mut b = BorrowedMysqlTransaction::new(&mut $tx, $c.clone()).unwrap();
        assert_eq!(
            b.validate_command(&$c, &$op, &$ns, CommandId::new([51; 16]), &$l, MAX_TTL)
                .unwrap_err(),
            ExecutionError::Unsupported
        );
        assert_eq!(
            b.execute_command(&$c, &$op, $h).await.unwrap_err(),
            AtomicFailure {
                cause: ExecutionError::Unsupported,
                disposition: AtomicDisposition::NotStarted
            }
        );
        assert!(b.recorder.events.is_empty());
    }};
}

macro_rules! command_suite {
($test:ident,$feature:literal,$session:ident,$connection:ty,$options:ty,$dialect:expr,$dsn:literal,$ddl:literal,$ssl:expr) => {
#[cfg(feature=$feature)]
#[test]
fn $test(){
    use sqlx::{Connection,ConnectOptions,Row};
    assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),Ok("quent_dispatch_test"));
    let options:$options=std::env::var($dsn).unwrap().parse().unwrap();
    let runtime=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    #[cfg(feature="output-tracing")] let capture=output_capture::Capture::default();
    #[cfg(feature="output-tracing")] let subscriber=tracing::subscriber::set_default(capture.clone());
    runtime.block_on(Box::pin(async {
        let d=$dialect;let c=command_context(d,11,0,vec![]);let namespace=ns();let l=ledger("command_ledger");
        let mut observer=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
        for statement in include_str!($ddl).split(';') {if let Some(start)=statement.find("CREATE TABLE") {sqlx::query(sqlx::AssertSqlSafe(statement[start..].to_owned())).execute(&mut observer).await.unwrap();}}
        let ddl_check=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_attribute a JOIN pg_class t ON t.oid=a.attrelid JOIN pg_namespace n ON n.oid=t.relnamespace LEFT JOIN pg_attrdef x ON x.adrelid=a.attrelid AND x.adnum=a.attnum WHERE n.nspname='quent_dispatch_test' AND t.relname IN ('command_items','command_ledger') AND a.attnum>0 AND NOT a.attisdropped AND (a.attidentity<>'' OR a.attgenerated<>'' OR x.oid IS NOT NULL OR t.relrowsecurity)"}else{"SELECT COUNT(*) FROM information_schema.columns WHERE TABLE_SCHEMA='quent_dispatch_test' AND TABLE_NAME IN ('command_items','command_ledger') AND (EXTRA<>'' OR COLUMN_DEFAULT IS NOT NULL)"};
        assert_eq!(sqlx::query(sqlx::AssertSqlSafe(ddl_check.to_owned())).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);
        if d==Dialect::Mysql{assert_eq!(sqlx::query("SELECT COUNT(*) FROM information_schema.tables WHERE TABLE_SCHEMA='quent_dispatch_test' AND TABLE_NAME IN ('command_items','command_ledger') AND ENGINE='InnoDB'").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),2);}
        let triggers=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_trigger g JOIN pg_class t ON t.oid=g.tgrelid JOIN pg_namespace n ON n.oid=t.relnamespace WHERE n.nspname='quent_dispatch_test' AND t.relname IN ('command_items','command_ledger') AND NOT g.tgisinternal"}else{"SELECT COUNT(*) FROM information_schema.triggers WHERE TRIGGER_SCHEMA='quent_dispatch_test' AND EVENT_OBJECT_TABLE IN ('command_items','command_ledger')"};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(triggers.to_owned())).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);
        let mut s=$session::connect(options.clone(),c.clone()).await.unwrap();
        let provider=Arc::new(super::authorized::Probe::default());
        let authorization=AuthorizationContext::register(c.clone(),"CANARY_command_provider".into(),1,b"CANARY_credential".to_vec(),provider.clone()).unwrap();
        let request=AuthorizationExecutionRequest::new();
        let broad=Draft::update(ct(),vec![(cc("amount"),Input::Value(Value::I64(1)))]).filter(Predicate::Compare(cc("tenant"),Compare::Eq,Value::I64(11))).freeze();
        let candidate=s.authorization_candidate(&authorization,&request,&broad,AuthorizationScope::MultiRowUpdate,MAX_TTL).unwrap();
        drop(s.acquire_authorized(&authorization,&request,candidate).await.unwrap());
        assert_eq!((provider.calls(),provider.responses()),(1,1));assert!(s.recorder.calls.is_empty());
        let mut executions=0usize;let mut replays=0usize;let mut rollbacks=0usize;
        // First explicit/all-column and implicit tenant INSERT, plus Low UPDATE.
        for (index,op) in [ci(1,Value::Null(SqlType::Text),true),ci(2,Value::Text(String::new()),false),cu(1,7)].into_iter().enumerate(){
            let durable=CommandId::new([index as u8;16]);
            let h=s.validate_command(&c,&op,&namespace,durable.clone(),&l,MAX_TTL).unwrap();
            let expected_sql=h.operation.prepared.sensitive_source().final_source().sql().as_ptr() as usize;
            let expected_args=h.operation.prepared.sensitive_source().final_source().args().to_vec();
            let expected_metadata=h.operation.prepared.sensitive_source().final_source().metadata().clone();
            let key=h.key.sensitive_bytes().to_vec();let input=h.input.sensitive_bytes().to_vec();
            assert_eq!(format!("{h:?}"),"ValidatedCommand([REDACTED])");
            let r=s.execute_command(&c,&op,h).await.unwrap();executions+=1;
            assert_eq!(format!("{r:?}"),"CommandResult([REDACTED])");assert_eq!(r.kind(),CommandResultKind::FirstCommit);assert_eq!(r.sensitive_result().state(),ResultState::Committed);
            assert_eq!(s.recorder.calls.len(),1);assert_eq!(s.recorder.sql_addresses,[expected_sql]);assert_eq!(s.recorder.calls[0].1,expected_args);assert_eq!(s.recorder.metadata,[expected_metadata]);
            assert_eq!(s.recorder.events,["begin","begun","ledger_lookup","ledger_reserve","command_ready","send","decoded","ledger_result","commit","committed"]);
            s.recorder=Recorder::default();
            let query=if d==Dialect::Postgres{"SELECT input,result FROM quent_dispatch_test.command_ledger WHERE identity=$1"}else{"SELECT input,result FROM quent_dispatch_test.command_ledger WHERE identity=?"};
            let row=sqlx::query(sqlx::AssertSqlSafe(query.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap();
            assert_eq!(row.get::<Vec<u8>,_>(0),input);assert_eq!(row.get::<Vec<u8>,_>(1).len(),13);
            for lookup in [false,true]{
                let mut new=$session::connect(options.clone(),c.clone()).await.unwrap();let h=new.validate_command(&c,&op,&namespace,durable.clone(),&l,MAX_TTL).unwrap();
                let r=if lookup{new.lookup_command(&c,&op,h).await.unwrap().unwrap()}else{new.execute_command(&c,&op,h).await.unwrap()};replays+=1;
                assert_eq!(r.kind(),CommandResultKind::Replayed);assert_eq!(r.sensitive_result().affected_rows(),Some(if d==Dialect::Postgres{AffectedRows::PostgresMatched(1)}else{AffectedRows::MysqlFoundRows(1)}));
                assert!(new.recorder.calls.is_empty());assert_eq!(new.recorder.events,["begin","begun","ledger_lookup","commit","committed"]);
            }
            let changed=if index==2{cu(1,8)}else{ci((index+1) as i64,Value::Text("CANARY_different_input".into()),index==0)};
            let h=s.validate_command(&c,&changed,&namespace,durable.clone(),&l,MAX_TTL).unwrap();let e=s.execute_command(&c,&changed,h).await.unwrap_err();
            assert_eq!(e.cause(),ExecutionError::IdentityConflict);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);rollbacks+=1;assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        }
        let row=sqlx::query("SELECT amount,note FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap();assert_eq!(row.get::<i64,_>(0),7);assert!(row.get::<Option<String>,_>(1).is_none());
        // Namespace/tenant/target separation: successful lookup returns None, never
        // an unrelated row. No implicit business write on lookup miss.
        for variant in 0..3{
            let different=command_context(d,if variant==1{22}else{11},if variant==2{1}else{0},vec![]);
            let namespace=if variant==0{CommandNamespace::new(b"different_namespace".to_vec()).unwrap()}else{ns()};
            let mut new=$session::connect(options.clone(),different.clone()).await.unwrap();let op=ci(2,Value::Text(String::new()),false);
            let h=new.validate_command(&different,&op,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert!(new.lookup_command(&different,&op,h).await.unwrap().is_none());assert!(new.recorder.calls.is_empty());
        }
        // Omitted tenant vs explicitly equal tenant, operation kind and ordered
        // values are separate inputs even when they would write the same row.
        for changed in [ci(2,Value::Text(String::new()),true),cu(2,0),Draft::insert(ct(),vec![(cc("note"),Input::Value(Value::Text(String::new()))),(cc("flag"),Input::Value(Value::Bool(false))),(cc("amount"),Input::Value(Value::I64(0))),(cc("id"),Input::Value(Value::I64(2)))]).freeze()] {
            let h=s.validate_command(&c,&changed,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert_eq!(s.execute_command(&c,&changed,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        }
        for (amount,flag) in [(1,false),(0,true)] {
            let changed=Draft::insert(ct(),vec![(cc("id"),Input::Value(Value::I64(2))),(cc("amount"),Input::Value(Value::I64(amount))),(cc("flag"),Input::Value(Value::Bool(flag))),(cc("note"),Input::Value(Value::Text(String::new())))]).freeze();let h=s.validate_command(&c,&changed,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert_eq!(s.execute_command(&c,&changed,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        }
        let old=cu(1,8);let changed=cu(1,9);let h=s.validate_command(&c,&old,&namespace,CommandId::new([18;16]),&l,MAX_TTL).unwrap();assert_eq!(s.execute_command(&c,&changed,h).await.unwrap_err().cause(),ExecutionError::OperationMismatch);assert!(s.recorder.events.is_empty());
        let pc=command_context(d,11,0,vec![PolicyDeclaration::Readonly(cc("note"))]);let mut ps=$session::connect(options.clone(),pc.clone()).await.unwrap();let op=cu(1,7);let h=ps.validate_command(&pc,&op,&namespace,CommandId::new([2;16]),&l,MAX_TTL).unwrap();assert_eq!(ps.lookup_command(&pc,&op,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(ps.recorder.calls.is_empty());
        // Whole-context comparison catches changed contents under the same generation.
        let op=ci(2,Value::Text(String::new()),false);let different=command_context(d,11,2,vec![]);
        let h=s.validate_command(&c,&op,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();let e=s.lookup_command(&different,&op,h).await.unwrap_err();assert_eq!(e.disposition(),AtomicDisposition::NotStarted);assert_eq!(e.cause(),ExecutionError::ContextMismatch);assert!(s.recorder.events.is_empty());
        let mut new=$session::connect(options.clone(),different.clone()).await.unwrap();let h=new.validate_command(&different,&op,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert_eq!(new.lookup_command(&different,&op,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(new.recorder.calls.is_empty());
        let changed_subject=command_context(d,11,4,vec![]);let mut subject_session=$session::connect(options.clone(),changed_subject.clone()).await.unwrap();let h=subject_session.validate_command(&changed_subject,&op,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();assert_eq!(subject_session.lookup_command(&changed_subject,&op,h).await.unwrap_err().cause(),ExecutionError::IdentityConflict);assert!(subject_session.recorder.calls.is_empty());
        let absent_subject=command_context(d,11,5,vec![]);let absent=$session::connect(options.clone(),absent_subject.clone()).await.unwrap();assert_eq!(absent.validate_command(&absent_subject,&op,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap_err(),ExecutionError::MissingSubject);assert!(absent.recorder.events.is_empty());
        // Preparation refusals happen before reservation or business dispatch.
        let mut missing=vec![(cc("id"),Input::Value(Value::I64(9))),(cc("amount"),Input::Value(Value::I64(0))),(cc("flag"),Input::Value(Value::Bool(false)))];
        assert!(s.validate_command(&c,&Draft::insert(ct(),missing.clone()).freeze(),&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err()); // omitted nullable note
        missing.push((cc("note"),Input::Value(Value::Null(SqlType::Text))));missing.push((cc("tenant"),Input::Value(Value::I64(22))));assert!(s.validate_command(&c,&Draft::insert(ct(),missing).freeze(),&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());
        for bad in [Draft::insert(ct(),vec![(cc("id"),Input::Missing)]).freeze(),Draft::insert(ct(),vec![(cc("id"),Input::Value(Value::Null(SqlType::I64)))]).freeze(),Draft::insert(ct(),vec![(cc("id"),Input::Value(Value::I32(1)))]).freeze(),Draft::update(ct(),vec![(cc("amount"),Input::Value(Value::I64(9)))]).filter(Predicate::Compare(cc("tenant"),Compare::Eq,Value::I64(11))).freeze()] {assert!(s.validate_command(&c,&bad,&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());}
        assert_eq!(s.validate(&c,&op,MAX_TTL).unwrap_err(),ExecutionError::RiskRefused);assert!(s.recorder.events.is_empty());
        for policy in [PolicyDeclaration::Readonly(cc("amount")),PolicyDeclaration::Generated(cc("amount")),PolicyDeclaration::Pii(cc("note"))]{let pc=command_context(d,11,0,vec![policy]);let ps=$session::connect(options.clone(),pc.clone()).await.unwrap();assert!(ps.validate_command(&pc,&op,&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());assert!(ps.recorder.events.is_empty());}
        let unsupported=command_context(d,11,3,vec![]);let us=$session::connect(options.clone(),unsupported.clone()).await.unwrap();assert!(us.validate_command(&unsupported,&op,&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());
        let base=vec![(cc("id"),Input::Value(Value::I64(9))),(cc("amount"),Input::Value(Value::I64(0))),(cc("flag"),Input::Value(Value::Bool(false))),(cc("note"),Input::Value(Value::Null(SqlType::Text)))];
        let mut duplicate=base.clone();duplicate.push((cc("id"),Input::Value(Value::I64(9))));
        let mut extra=base.clone();extra.push((ct().column(id("extra"),SqlType::I64,false),Input::Value(Value::I64(0))));
        let mut nullability=base.clone();nullability[3].0=ct().column(id("note"),SqlType::Text,false);
        for data in [duplicate,extra,nullability]{assert!(s.validate_command(&c,&Draft::insert(ct(),data).freeze(),&namespace,CommandId::new([9;16]),&l,MAX_TTL).is_err());assert!(s.recorder.events.is_empty());}
        // Unpolled consumption and expiry/session mismatch: zero ledger and business.
        let h=s.validate_command(&c,&op,&namespace,CommandId::new([10;16]),&l,MAX_TTL).unwrap();drop(s.execute_command(&c,&op,h));assert!(s.recorder.events.is_empty());
        let h=s.validate_command(&c,&op,&namespace,CommandId::new([10;16]),&l,Duration::from_millis(1)).unwrap();tokio::time::sleep(Duration::from_millis(3)).await;assert_eq!(s.execute_command(&c,&op,h).await.unwrap_err().cause(),ExecutionError::Expired);assert!(s.recorder.events.is_empty());
        let h=s.validate_command(&c,&op,&namespace,CommandId::new([10;16]),&l,MAX_TTL).unwrap();let mut other=$session::connect(options.clone(),c.clone()).await.unwrap();assert_eq!(other.execute_command(&c,&op,h).await.unwrap_err().cause(),ExecutionError::ContextMismatch);assert!(other.recorder.events.is_empty());
        // Business constraint failure and full result expectation failure each
        // roll back the actual reservation; independent observer sees no ledger.
        for (index,op) in [(20,cu(1,-1)),(21,cu(999,7)),(22,ci(1,Value::Text("duplicate".into()),false))] {
            let h=s.validate_command(&c,&op,&namespace,CommandId::new([index;16]),&l,MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();let e=s.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.disposition(),AtomicDisposition::RolledBack);rollbacks+=1;assert_eq!(s.recorder.calls.len(),1);
            let query=if d==Dialect::Postgres{"SELECT COUNT(*) FROM quent_dispatch_test.command_ledger WHERE identity=$1"}else{"SELECT COUNT(*) FROM quent_dispatch_test.command_ledger WHERE identity=?"};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(query.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);s.recorder=Recorder::default();
        }
        // A real ledger result CHECK failure occurs AFTER a business UPDATE;
        // the independently observed business value and reservation both roll back.
        let bad_ddl=if d==Dialect::Postgres{"CREATE TABLE quent_dispatch_test.command_ledger_fail(identity BYTEA PRIMARY KEY,input BYTEA NOT NULL,result BYTEA NULL CHECK(result IS NULL),version SMALLINT NOT NULL)"}else{"CREATE TABLE quent_dispatch_test.command_ledger_fail(identity VARBINARY(2048) PRIMARY KEY,input LONGBLOB NOT NULL,result VARBINARY(13) NULL,version SMALLINT NOT NULL,CHECK(result IS NULL)) ENGINE=InnoDB"};
        sqlx::query(sqlx::AssertSqlSafe(bad_ddl.to_owned())).execute(&mut observer).await.unwrap();
        let op=cu(1,99);let h=s.validate_command(&c,&op,&namespace,CommandId::new([25;16]),&ledger("command_ledger_fail"),MAX_TTL).unwrap();let e=s.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerFailed);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);rollbacks+=1;assert_eq!(s.recorder.calls.len(),1);assert!(s.recorder.events.contains(&"ledger_result"));
        assert_eq!(sqlx::query("SELECT amount FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),7);
        assert_eq!(sqlx::query("SELECT COUNT(*) FROM quent_dispatch_test.command_ledger_fail").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);s.recorder=Recorder::default();
        // Unknown ledger protocol version in an intentionally invalid fixture
        // is rejected from real storage before treating its NULL result as missing.
        let bad=ledger("command_ledger_fail");let h=s.validate_command(&c,&op,&namespace,CommandId::new([27;16]),&bad,MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();let input=h.input.sensitive_bytes().to_vec();drop(h);
        let seed=if d==Dialect::Postgres{"INSERT INTO quent_dispatch_test.command_ledger_fail(identity,input,result,version) VALUES($1,$2,NULL,2)"}else{"INSERT INTO quent_dispatch_test.command_ledger_fail(identity,input,result,version) VALUES(?,?,NULL,2)"};sqlx::query(sqlx::AssertSqlSafe(seed.to_owned())).bind(&key).bind(&input).execute(&mut observer).await.unwrap();
        let h=s.validate_command(&c,&op,&namespace,CommandId::new([27;16]),&bad,MAX_TTL).unwrap();assert_eq!(s.lookup_command(&c,&op,h).await.unwrap_err().cause(),ExecutionError::LedgerInvalid);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        // Missing ledger is a read error, never permission to proceed to business.
        let h=s.validate_command(&c,&op,&namespace,CommandId::new([26;16]),&ledger("command_ledger_missing"),MAX_TTL).unwrap();let e=s.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerFailed);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        // Completion fault is test-seam evidence, not physical reply loss.
        let op=cu(1,8);s.atomic_fault=Some(AtomicFault::CommitReplyLost);let h=s.validate_command(&c,&op,&namespace,CommandId::new([30;16]),&l,MAX_TTL).unwrap();let e=s.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown);assert!(!s.is_available());
        let mut s=$session::connect(options.clone(),c.clone()).await.unwrap();let h=s.validate_command(&c,&op,&namespace,CommandId::new([30;16]),&l,MAX_TTL).unwrap();assert_eq!(s.lookup_command(&c,&op,h).await.unwrap().unwrap().kind(),CommandResultKind::Replayed);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        // Real server UPDATE wait is observed before drop, timeout and disconnect.
        // The ledger reservation is still uncommitted and business sends are one.
        for mode in 0..3 {
            let mut locker=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut lock=locker.begin().await.unwrap();
            sqlx::query("UPDATE quent_dispatch_test.command_items SET amount=amount WHERE tenant=11 AND id=1").execute(&mut *lock).await.unwrap();
            let mut canceled=$session::connect(options.clone(),c.clone()).await.unwrap();let connection_id=canceled.test_connection_id().await;let op=cu(1,9);
            let h=canceled.validate_command(&c,&op,&namespace,CommandId::new([40+mode;16]),&l,MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();let trace=canceled.recorder.live.clone();let mut future=Box::pin(canceled.execute_command(&c,&op,h));
            let waiting=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user AND wait_event_type='Lock' AND query LIKE 'UPDATE %'"}else{"SELECT COUNT(*) FROM performance_schema.data_lock_waits w JOIN performance_schema.threads t ON t.THREAD_ID=w.REQUESTING_THREAD_ID WHERE t.PROCESSLIST_ID=?"};
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(future.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(connection_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
            assert_eq!(trace.lock().unwrap().iter().filter(|e|**e=="send").count(),1);
            if mode==1{assert!(tokio::time::timeout(Duration::from_millis(1),future.as_mut()).await.is_err());}
            if mode==2{if d==Dialect::Postgres{sqlx::query("SELECT pg_terminate_backend(CAST($1 AS INT))").bind(connection_id).execute(&mut observer).await.unwrap();}else{sqlx::query(sqlx::AssertSqlSafe(format!("KILL CONNECTION {connection_id}"))).execute(&mut observer).await.unwrap();}let e=tokio::time::timeout(Duration::from_secs(10),future.as_mut()).await.unwrap().unwrap_err();assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown);}
            drop(future);assert!(!canceled.is_available());assert!(!trace.lock().unwrap().contains(&"committed"));assert!(!trace.lock().unwrap().contains(&"rolled_back"));drop(canceled);lock.rollback().await.unwrap();
            let query=if d==Dialect::Postgres{"SELECT COUNT(*) FROM quent_dispatch_test.command_ledger WHERE identity=$1"}else{"SELECT COUNT(*) FROM quent_dispatch_test.command_ledger WHERE identity=?"};assert_eq!(sqlx::query(sqlx::AssertSqlSafe(query.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);
            assert_eq!(sqlx::query("SELECT amount FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),8);
        }
        // Observe the real await at the existing test-only final barrier before
        // business send, let the REAL clock expire, then require acknowledged rollback.
        let op=cu(1,9);let barrier=Arc::new(tokio::sync::Notify::new());s.final_barrier=Some(barrier.clone());let h=s.validate_command(&c,&op,&namespace,CommandId::new([50;16]),&l,Duration::from_millis(100)).unwrap();let trace=s.recorder.live.clone();let mut future=Box::pin(s.execute_command(&c,&op,h));
        tokio::time::timeout(Duration::from_secs(2),async{loop{assert!(poll_once(future.as_mut()).await.is_pending());if trace.lock().unwrap().contains(&"command_ready"){break;}tokio::task::yield_now().await;}}).await.unwrap();assert!(!trace.lock().unwrap().contains(&"send"));tokio::time::sleep(Duration::from_millis(110)).await;barrier.notify_one();let e=future.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(s.recorder.calls.is_empty());s.recorder=Recorder::default();
        // Borrowed Tx cannot acquire or consume an atomic command. Completion stays external.
        let h=s.validate_command(&c,&op,&namespace,CommandId::new([51;16]),&l,MAX_TTL).unwrap();
        let mut external=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut tx=external.begin().await.unwrap();
        borrowed_command_cases!($session,d,tx,c,op,namespace,l,h);
        tx.rollback().await.unwrap();
        // Minimum real production reservation winner/loser case. The existing
        // private final barrier orders actual transactions, not a fake executor.
        let op=ci(3,Value::Null(SqlType::Text),false);
        let mut winner=$session::connect(options.clone(),c.clone()).await.unwrap();let barrier=Arc::new(tokio::sync::Notify::new());winner.final_barrier=Some(barrier.clone());let trace=winner.recorder.live.clone();
        let h=winner.validate_command(&c,&op,&namespace,CommandId::new([60;16]),&l,MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();let mut win=Box::pin(winner.execute_command(&c,&op,h));
        tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(win.as_mut()).await.is_pending());if trace.lock().unwrap().contains(&"command_ready"){break;}tokio::task::yield_now().await;}}).await.unwrap();
        let mut loser=$session::connect(options.clone(),c.clone()).await.unwrap();let connection_id=loser.test_connection_id().await;let h=loser.validate_command(&c,&op,&namespace,CommandId::new([60;16]),&l,MAX_TTL).unwrap();let loser_trace=loser.recorder.live.clone();let mut lose=Box::pin(loser.execute_command(&c,&op,h));
        let waiting=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user AND wait_event_type='Lock' AND query LIKE 'INSERT INTO %'"}else{"SELECT COUNT(*) FROM performance_schema.data_lock_waits w JOIN performance_schema.threads t ON t.THREAD_ID=w.REQUESTING_THREAD_ID WHERE t.PROCESSLIST_ID=?"};
        tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(lose.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(connection_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
        assert!(!loser_trace.lock().unwrap().contains(&"send"));
        let count_query=if d==Dialect::Postgres{"SELECT COUNT(*) FROM quent_dispatch_test.command_ledger WHERE identity=$1"}else{"SELECT COUNT(*) FROM quent_dispatch_test.command_ledger WHERE identity=?"};
        assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count_query.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0); // uncommitted reservation invisible
        barrier.notify_one();assert_eq!(win.await.unwrap().kind(),CommandResultKind::FirstCommit);executions+=1;
        let e=lose.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerFailed);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);rollbacks+=1;assert_eq!(winner.recorder.calls.len(),1);assert!(loser.recorder.calls.is_empty());
        assert_eq!(sqlx::query("SELECT COUNT(*) FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=3").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),1);
        assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count_query.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),1);
        let h=loser.validate_command(&c,&op,&namespace,CommandId::new([60;16]),&l,MAX_TTL).unwrap();assert_eq!(loser.lookup_command(&c,&op,h).await.unwrap().unwrap().kind(),CommandResultKind::Replayed);replays+=1;assert!(loser.recorder.calls.is_empty());
        // Frozen key/input emitted by the MERGED PR1 Core writer, not by this
        // revision's encoder. Seed an old saved record then replay with both APIs.
        let old_key=if d==Dialect::Postgres{include_bytes!("../../../../../fixtures/command-ledger/legacy-pr1-postgres-key.bin").as_slice()}else{include_bytes!("../../../../../fixtures/command-ledger/legacy-pr1-mysql-key.bin").as_slice()};
        let old_input=if d==Dialect::Postgres{include_bytes!("../../../../../fixtures/command-ledger/legacy-pr1-postgres-input.bin").as_slice()}else{include_bytes!("../../../../../fixtures/command-ledger/legacy-pr1-mysql-input.bin").as_slice()};
        let old_result=[b"QCR1".as_slice(),&[if d==Dialect::Postgres{1}else{2}],&1u64.to_be_bytes()].concat();
        sqlx::query("INSERT INTO quent_dispatch_test.command_items(tenant,id,amount,flag,note) VALUES(11,888,0,FALSE,NULL)").execute(&mut observer).await.unwrap();
        let insert_old=if d==Dialect::Postgres{"INSERT INTO quent_dispatch_test.command_ledger(identity,input,result,version) VALUES($1,$2,$3,1)"}else{"INSERT INTO quent_dispatch_test.command_ledger(identity,input,result,version) VALUES(?,?,?,1)"};sqlx::query(sqlx::AssertSqlSafe(insert_old.to_owned())).bind(old_key).bind(old_input).bind(&old_result).execute(&mut observer).await.unwrap();
        for explicit in [false,true] {let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let op=ci(888,Value::Null(SqlType::Text),false);let h=if explicit{fresh.validate_command_expected(&c,&op,&namespace,CommandId::new([150;16]),&l,ExpectedRows(1),MAX_TTL)}else{fresh.validate_command(&c,&op,&namespace,CommandId::new([150;16]),&l,MAX_TTL)}.unwrap();assert_eq!(h.key.sensitive_bytes(),old_key);assert_eq!(h.input.sensitive_bytes(),old_input);let r=fresh.execute_command(&c,&op,h).await.unwrap();assert_eq!(r.kind(),CommandResultKind::Replayed);assert_eq!(result_bytes(r.sensitive_result(),d).unwrap(),old_result);assert!(fresh.recorder.calls.is_empty());}
        println!("PR2 {:?} merged PR1 canonical fixtures byte-equal; old ledger replay via implicit/explicit exact1 APIs business0 each",d);
        // PR2 finite optimistic commands, exact expectations, and saved results.
        for (i, expected_version, actual_success) in [(100u8,0,true),(101,0,false),(102,1,true)] {
            let op=Draft::update(ct(),vec![(cc("note"),Input::Value(Value::Text("CANARY_version_result".into())))])
                .filter(Predicate::Compare(cc("id"),Compare::Eq,Value::I64(2)))
                .optimistic_version(cc("amount"),Value::I64(expected_version)).unwrap().freeze();
            let mut v=$session::connect(options.clone(),c.clone()).await.unwrap();
            let h=v.validate_command_expected(&c,&op,&namespace,CommandId::new([i;16]),&l,ExpectedRows(1),MAX_TTL).unwrap();
            let key=h.key.sensitive_bytes().to_vec();let input=h.input.sensitive_bytes().to_vec();
            let source=h.operation.prepared.sensitive_source();let sql=source.final_source().sql().to_owned();let address=source.final_source().sql().as_ptr() as usize;let metadata=source.final_source().metadata().clone();let args=source.final_source().args().to_vec();
            let result=v.execute_command(&c,&op,h).await;
            assert_eq!(v.recorder.calls,vec![(sql,args)]);assert_eq!(v.recorder.sql_addresses,[address]);assert_eq!(v.recorder.metadata,[metadata]);
            if actual_success {
                assert_eq!(result.unwrap().kind(),CommandResultKind::FirstCommit);
                let mut replay=$session::connect(options.clone(),c.clone()).await.unwrap();
                let h=replay.validate_command(&c,&op,&namespace,CommandId::new([i;16]),&l,MAX_TTL).unwrap();
                let r=replay.execute_command(&c,&op,h).await.unwrap();assert_eq!(r.kind(),CommandResultKind::Replayed);assert!(replay.recorder.calls.is_empty());
                let select=if d==Dialect::Postgres{"SELECT input,result FROM quent_dispatch_test.command_ledger WHERE identity=$1"}else{"SELECT input,result FROM quent_dispatch_test.command_ledger WHERE identity=?"};
                let row=sqlx::query(sqlx::AssertSqlSafe(select.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap();assert_eq!(row.get::<Vec<u8>,_>(0),input);assert_eq!(row.get::<Vec<u8>,_>(1),result_bytes(r.sensitive_result(),d).unwrap());
            } else {let e=result.unwrap_err();assert_eq!(e.cause(),ExecutionError::Conflict);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);}
            println!("PR2 {:?} version case={} business1 success={} replay business0 on success",d,i,actual_success);
        }
        assert_eq!(sqlx::query("SELECT amount FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=2").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),2);
        // Each supported native signed width reaches its maximum by a checked
        // successor, then refuses overflow before any SQL. PG i8 stays unsupported.
        for (j,ty,sql_type,expected,max) in [(0,SqlType::I8,"TINYINT",Value::I8(i8::MAX-1),Value::I8(i8::MAX)),(1,SqlType::I16,"SMALLINT",Value::I16(i16::MAX-1),Value::I16(i16::MAX)),(2,SqlType::I32,"INTEGER",Value::I32(i32::MAX-1),Value::I32(i32::MAX)),(3,SqlType::I64,"BIGINT",Value::I64(i64::MAX-1),Value::I64(i64::MAX))] {
            let table=Table::new(vec![id("quent_dispatch_test"),id("command_native")],None).unwrap();let tenant=table.column(id("tenant"),SqlType::I64,false);let key=table.column(id("id"),SqlType::I64,false);let version=table.column(id("version"),ty,false);let note=table.column(id("note"),SqlType::Text,false);
            let native_context=extra_context(d,table.clone(),vec![tenant.clone(),key.clone(),version.clone(),note.clone()]);
            let insert=Draft::insert(table.clone(),vec![(key.clone(),Input::Value(Value::I64(1))),(version.clone(),Input::Value(expected.clone())),(note.clone(),Input::Value(Value::Text("CANARY_native_version".into())))]).freeze();
            let mut native=$session::connect(options.clone(),native_context.clone()).await.unwrap();
            if d==Dialect::Postgres && ty==SqlType::I8 {assert_eq!(native.validate_command(&native_context,&insert,&namespace,CommandId::new([130;16]),&l,MAX_TTL).unwrap_err(),ExecutionError::Policy(PolicyError::InvalidPlan(quent_core::PlanError::Unsupported)));assert!(native.recorder.events.is_empty());continue;}
            let ddl=format!("CREATE TABLE quent_dispatch_test.command_native(tenant BIGINT NOT NULL,id BIGINT NOT NULL,version {sql_type} NOT NULL,note TEXT NOT NULL,PRIMARY KEY(tenant,id)){}",if d==Dialect::Mysql{" ENGINE=InnoDB"}else{""});sqlx::query(sqlx::AssertSqlSafe(ddl)).execute(&mut observer).await.unwrap();
            let h=native.validate_command(&native_context,&insert,&namespace,CommandId::new([130+j*3;16]),&l,MAX_TTL).unwrap();assert_eq!(native.execute_command(&native_context,&insert,h).await.unwrap().kind(),CommandResultKind::FirstCommit);native.recorder=Recorder::default();
            let update=||Draft::update(table.clone(),vec![(note.clone(),Input::Value(Value::Text("CANARY_native_next".into())))]).filter(Predicate::Compare(key.clone(),Compare::Eq,Value::I64(1)));
            assert!(update().optimistic_version(version.clone(),max.clone()).is_err());assert!(native.recorder.events.is_empty());
            let op=update().optimistic_version(version.clone(),expected.clone()).unwrap().freeze();let h=native.validate_command(&native_context,&op,&namespace,CommandId::new([131+j*3;16]),&l,MAX_TTL).unwrap();assert_eq!(native.execute_command(&native_context,&op,h).await.unwrap().kind(),CommandResultKind::FirstCommit);assert_eq!(native.recorder.calls.len(),1);
            let mut fresh=$session::connect(options.clone(),native_context.clone()).await.unwrap();let h=fresh.validate_command(&native_context,&op,&namespace,CommandId::new([131+j*3;16]),&l,MAX_TTL).unwrap();assert_eq!(fresh.execute_command(&native_context,&op,h).await.unwrap().kind(),CommandResultKind::Replayed);assert!(fresh.recorder.calls.is_empty());
            let row=sqlx::query("SELECT version FROM quent_dispatch_test.command_native WHERE tenant=11 AND id=1").persistent(false).fetch_one(&mut observer).await.unwrap();let actual=match ty{SqlType::I8=>Value::I8(row.get(0)),SqlType::I16=>Value::I16(row.get(0)),SqlType::I32=>Value::I32(row.get(0)),_=>Value::I64(row.get(0))};assert_eq!(actual,max);
            sqlx::query("DROP TABLE quent_dispatch_test.command_native").execute(&mut observer).await.unwrap();
            println!("PR2 {:?} native version width case={} INSERT1/version1/replay0business/max observed/overflow construction refusal",d,j);
        }
        // Deliberately inaccurate asserted key: multiple real matches must not
        // become a successful saved result even though the ordinary gate proved
        // Single from supplied facts. This does not support missing physical keys.
        let table=Table::new(vec![id("quent_dispatch_test"),id("command_bad_key")],None).unwrap();let tenant=table.column(id("tenant"),SqlType::I64,false);let key=table.column(id("id"),SqlType::I64,false);let amount=table.column(id("amount"),SqlType::I64,false);
        let bad_context=extra_context(d,table.clone(),vec![tenant,key.clone(),amount.clone()]);let mut bad=$session::connect(options.clone(),bad_context.clone()).await.unwrap();
        let ddl=format!("CREATE TABLE quent_dispatch_test.command_bad_key(tenant BIGINT NOT NULL,id BIGINT NOT NULL,amount BIGINT NOT NULL){}",if d==Dialect::Mysql{" ENGINE=InnoDB"}else{""});sqlx::query(sqlx::AssertSqlSafe(ddl)).execute(&mut observer).await.unwrap();sqlx::query("INSERT INTO quent_dispatch_test.command_bad_key VALUES(11,1,0),(11,1,0)").execute(&mut observer).await.unwrap();
        for version in [false,true] {let mut draft=Draft::update(table.clone(),vec![]).filter(Predicate::Compare(key.clone(),Compare::Eq,Value::I64(1)));if version {draft=draft.optimistic_version(amount.clone(),Value::I64(0)).unwrap();}else{draft=Draft::update(table.clone(),vec![(amount.clone(),Input::Value(Value::I64(1)))]).filter(Predicate::Compare(key.clone(),Compare::Eq,Value::I64(1)));}
            let op=draft.freeze();let h=bad.validate_command(&bad_context,&op,&namespace,CommandId::new([145+u8::from(version);16]),&l,MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();let e=bad.execute_command(&bad_context,&op,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::ExpectationMismatch);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(bad.recorder.calls.len(),1);assert!(!bad.recorder.events.contains(&"ledger_result"));assert_eq!(sqlx::query("SELECT COUNT(*) FROM quent_dispatch_test.command_bad_key WHERE amount<>0").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count_query.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);bad.recorder=Recorder::default();}
        sqlx::query("DROP TABLE quent_dispatch_test.command_bad_key").execute(&mut observer).await.unwrap();
        let mut zero=$session::connect(options.clone(),c.clone()).await.unwrap();
        for expected in [0,2,256] {for op in [ci(900,Value::Null(SqlType::Text),false),cu(999,1),Draft::update(ct(),vec![(cc("note"),Input::Value(Value::Text("x".into())))]).filter(Predicate::Compare(cc("id"),Compare::Eq,Value::I64(2))).optimistic_version(cc("amount"),Value::I64(2)).unwrap().freeze()] {
            // Includes an already committed identity; unsupported expectation is
            // rejected without reading/disclosing whether that identity exists.
            assert_eq!(zero.validate_command_expected(&c,&op,&namespace,CommandId::new([100;16]),&l,ExpectedRows::exactly(expected).unwrap(),MAX_TTL).unwrap_err(),ExecutionError::Unsupported);
            assert!(zero.recorder.events.is_empty());assert!(zero.recorder.calls.is_empty());
        }}
        for (index,op,cause) in [(103,cu(999,1),ExecutionError::ExpectationMismatch),(104,Draft::update(ct(),vec![(cc("note"),Input::Value(Value::Text("x".into())))]).filter(Predicate::Compare(cc("id"),Compare::Eq,Value::I64(999))).optimistic_version(cc("amount"),Value::I64(0)).unwrap().freeze(),ExecutionError::Conflict)] {
            let h=zero.validate_command_expected(&c,&op,&namespace,CommandId::new([index;16]),&l,ExpectedRows(1),MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();
            let e=zero.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.cause(),cause);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(zero.recorder.calls.len(),1);assert!(!zero.recorder.events.contains(&"ledger_result"));assert!(!zero.recorder.events.contains(&"commit"));
            assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count_query.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),0);zero.recorder=Recorder::default();
        }
        let different_version=Draft::update(ct(),vec![(cc("note"),Input::Value(Value::Text("CANARY_version_result".into())))])
            .filter(Predicate::Compare(cc("id"),Compare::Eq,Value::I64(2))).optimistic_version(cc("amount"),Value::I64(1)).unwrap().freeze();
        let original_version=Draft::update(ct(),vec![(cc("note"),Input::Value(Value::Text("CANARY_version_result".into())))])
            .filter(Predicate::Compare(cc("id"),Compare::Eq,Value::I64(2))).optimistic_version(cc("amount"),Value::I64(0)).unwrap().freeze();
        let h=zero.validate_command(&c,&original_version,&namespace,CommandId::new([100;16]),&l,MAX_TTL).unwrap();let e=zero.execute_command(&c,&different_version,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::OperationMismatch);assert_eq!(e.disposition(),AtomicDisposition::NotStarted);assert!(zero.recorder.events.is_empty());
        let h=zero.validate_command(&c,&different_version,&namespace,CommandId::new([100;16]),&l,MAX_TTL).unwrap();let e=zero.execute_command(&c,&different_version,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::IdentityConflict);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(zero.recorder.calls.is_empty());zero.recorder=Recorder::default();
        println!("PR2 {:?} unsupported expectations0/2/256 x INSERT/UPDATE/version: activity0; missing/version0: rollback/no reservation; same ID different version: IdentityConflict business0",d);
        // Native changed-row control versus unchanged production command match.
        let target=std::net::SocketAddr::new(options.get_host().parse().unwrap(),options.get_port());
        let proxy=super::command_network::Proxy::new(target,d==Dialect::Postgres,super::command_network::Mode::ChangedRows);
        let plain=options.clone().host("127.0.0.1").port(proxy.address.port()).ssl_mode($ssl).disable_statement_logging();
        let mut native=<$connection>::connect_with(&plain).await.unwrap();
        let changed=sqlx::query("UPDATE quent_dispatch_test.command_items SET amount=amount WHERE tenant=11 AND id=2").execute(&mut native).await.unwrap().rows_affected();
        assert_eq!(changed,if d==Dialect::Mysql{0}else{1});drop(native);drop(proxy);
        let op=cu(2,2);let h=zero.validate_command(&c,&op,&namespace,CommandId::new([105;16]),&l,MAX_TTL).unwrap();assert_eq!(zero.execute_command(&c,&op,h).await.unwrap().sensitive_result().affected_rows(),Some(if d==Dialect::Postgres{AffectedRows::PostgresMatched(1)}else{AffectedRows::MysqlFoundRows(1)}));
        println!("PR2 {:?} unchanged native control={} production matched1 (same observer value2)",d,changed);
        // Invalid version assignments keep the same policy/native/key gate.
        for column in [cc("tenant"),cc("note"),cc("flag")] {
            let value=match column.sql_type(){SqlType::Text=>Value::Text("bad".into()),SqlType::Bool=>Value::Bool(false),_=>Value::I64(11)};
            let draft=Draft::update(ct(),vec![(cc("note"),Input::Value(Value::Text("x".into())))]).filter(Predicate::Compare(cc("id"),Compare::Eq,Value::I64(2))).optimistic_version(column,value);
            if let Ok(draft)=draft {assert!(zero.validate_command(&c,&draft.freeze(),&namespace,CommandId::new([106;16]),&l,MAX_TTL).is_err());}
        }
        for policy in [PolicyDeclaration::Readonly(cc("amount")),PolicyDeclaration::Generated(cc("amount")),PolicyDeclaration::Pii(cc("amount"))] {
            let protected=command_context(d,11,0,vec![policy]);let session=$session::connect(options.clone(),protected.clone()).await.unwrap();
            let op=Draft::update(ct(),vec![(cc("note"),Input::Value(Value::Text("x".into())))]).filter(Predicate::Compare(cc("id"),Compare::Eq,Value::I64(2))).optimistic_version(cc("amount"),Value::I64(2)).unwrap().freeze();assert!(session.validate_command(&protected,&op,&namespace,CommandId::new([106;16]),&l,MAX_TTL).is_err());assert!(session.recorder.events.is_empty());
        }
        // Four actual two-session reservation interleavings. Server wait is the
        // scheduling evidence; a clock wait is used only to expire the leader.
        for rollback in [false,true] {for different in [false,true] {
            let index=110+u8::from(rollback)*2+u8::from(different);let op=cu(2,200+i64::from(index));let other=if different{cu(2,400+i64::from(index))}else{op.clone()};
            let mut lead=$session::connect(options.clone(),c.clone()).await.unwrap();let barrier=Arc::new(tokio::sync::Notify::new());lead.final_barrier=Some(barrier.clone());let trace=lead.recorder.live.clone();
            let h=lead.validate_command(&c,&op,&namespace,CommandId::new([index;16]),&l,if rollback{Duration::from_secs(1)}else{MAX_TTL}).unwrap();let key=h.key.sensitive_bytes().to_vec();let mut first=Box::pin(lead.execute_command(&c,&op,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(first.as_mut()).await.is_pending());if trace.lock().unwrap().contains(&"command_ready"){break;}tokio::task::yield_now().await;}}).await.unwrap();
            let mut follow=$session::connect(options.clone(),c.clone()).await.unwrap();let connection_id=follow.test_connection_id().await;let h=follow.validate_command(&c,&other,&namespace,CommandId::new([index;16]),&l,MAX_TTL).unwrap();let mut second=Box::pin(follow.execute_command(&c,&other,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(second.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(connection_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
            let mut lookup=$session::connect(options.clone(),c.clone()).await.unwrap();let h=lookup.validate_command(&c,&op,&namespace,CommandId::new([index;16]),&l,MAX_TTL).unwrap();assert!(lookup.lookup_command(&c,&op,h).await.unwrap().is_none());assert!(lookup.recorder.calls.is_empty()); // no retry authorization
            if rollback{tokio::time::sleep(Duration::from_millis(1010)).await;}
            barrier.notify_one();let first_result=first.await;
            if rollback {let e=first_result.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(second.await.unwrap().kind(),CommandResultKind::FirstCommit);assert!(lead.recorder.calls.is_empty());assert_eq!(follow.recorder.calls.len(),1);}
            else {assert_eq!(first_result.unwrap().kind(),CommandResultKind::FirstCommit);let e=second.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerFailed);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(lead.recorder.calls.len(),1);assert!(follow.recorder.calls.is_empty());}
            // Explicit reconciliation on a NEW connection/snapshot after ACK.
            let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let h=fresh.validate_command(&c,&other,&namespace,CommandId::new([index;16]),&l,MAX_TTL).unwrap();let result=fresh.lookup_command(&c,&other,h).await;
            if !rollback && different {assert_eq!(result.unwrap_err().cause(),ExecutionError::IdentityConflict);}else{assert_eq!(result.unwrap().unwrap().kind(),CommandResultKind::Replayed);}assert!(fresh.recorder.calls.is_empty());
            assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count_query.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),1);
            println!("PR2 {:?} race case={} server wait observed; leader rollback={} different={} durable record1; reconciliation business0",d,index,rollback,different);
        }}
        // Both attempts actually send business after the first one's UPDATE
        // waited on a real row lock, expired, completed, and ACK-rolled back.
        // This is the explicit counterexample to total business-send exactly-one.
        for different in [false,true] {
            let index=114+u8::from(different);let op=cu(2,700+i64::from(index));let other=if different{cu(2,800+i64::from(index))}else{op.clone()};
            let mut locker=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut lock=locker.begin().await.unwrap();sqlx::query("UPDATE quent_dispatch_test.command_items SET amount=amount WHERE tenant=11 AND id=2").execute(&mut *lock).await.unwrap();
            let mut lead=$session::connect(options.clone(),c.clone()).await.unwrap();let lead_id=lead.test_connection_id().await;let h=lead.validate_command(&c,&op,&namespace,CommandId::new([index;16]),&l,Duration::from_secs(1)).unwrap();let key=h.key.sensitive_bytes().to_vec();let mut first=Box::pin(lead.execute_command(&c,&op,h));
            let row_wait=if d==Dialect::Postgres{"SELECT COUNT(*) FROM pg_stat_activity WHERE pid=$1 AND usename=current_user AND wait_event_type='Lock' AND query LIKE 'UPDATE %'"}else{waiting};
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(first.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(row_wait.to_owned())).bind(lead_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
            let mut follow=$session::connect(options.clone(),c.clone()).await.unwrap();let follow_id=follow.test_connection_id().await;let h=follow.validate_command(&c,&other,&namespace,CommandId::new([index;16]),&l,MAX_TTL).unwrap();let mut second=Box::pin(follow.execute_command(&c,&other,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(second.as_mut()).await.is_pending());if sqlx::query(sqlx::AssertSqlSafe(waiting.to_owned())).bind(follow_id).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0)>0{break;}tokio::task::yield_now().await;}}).await.unwrap();
            tokio::time::sleep(Duration::from_millis(1010)).await;lock.rollback().await.unwrap();let e=first.await.unwrap_err();assert_eq!(e.cause(),ExecutionError::Expired);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert_eq!(second.await.unwrap().kind(),CommandResultKind::FirstCommit);
            assert_eq!(lead.recorder.calls.len(),1);assert_eq!(follow.recorder.calls.len(),1);assert!(lead.recorder.events.contains(&"rolled_back"));assert!(!lead.recorder.events.contains(&"ledger_result"));assert!(follow.recorder.events.contains(&"committed"));
            assert_eq!(sqlx::query("SELECT amount FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=2").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),if different{800+i64::from(index)}else{700+i64::from(index)});
            assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count_query.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),1);
            println!("PR2 {:?} business-wait race case={} both server waits observed; different={} lead business1/ACKrollback, follow business1/ACKcommit, durable ledger1",d,index,different);
        }

        // Dedicated real transport fixture. The driver sends native COMMIT;
        // its real acknowledgement is either forwarded, suppressed, or COMMIT
        // itself is stopped before forwarding. No completion fault is injected.
        let before_network=sqlx::query("SELECT amount FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=2").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0);
        for (i,mode) in [super::command_network::Mode::BeforeCommit,super::command_network::Mode::LoseReply,super::command_network::Mode::Pass].into_iter().enumerate() {
            use std::sync::atomic::Ordering;
            let target=std::net::SocketAddr::new(options.get_host().parse().unwrap(),options.get_port());
            let proxy=super::command_network::Proxy::new(target,d==Dialect::Postgres,mode);
            let plain=options.clone().host("127.0.0.1").port(proxy.address.port()).ssl_mode($ssl);
            let mut network=$session::connect(plain,c.clone()).await.unwrap();let op=cu(2,600+i as i64);
            let h=network.validate_command(&c,&op,&namespace,CommandId::new([120+i as u8;16]),&l,MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();let network_input=h.input.sensitive_bytes().to_vec();
            let result=tokio::time::timeout(Duration::from_secs(10),network.execute_command(&c,&op,h)).await.unwrap();
            if i==2{assert_eq!(result.unwrap().kind(),CommandResultKind::FirstCommit);}else{let e=result.unwrap_err();assert_eq!(e.cause(),ExecutionError::OutcomeUnknown);assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown);assert!(!network.is_available());}
            assert_eq!(network.recorder.calls.len(),1);assert_eq!(proxy.sent.load(Ordering::SeqCst),i!=0);assert_eq!(proxy.reply.load(Ordering::SeqCst),i!=0);drop(network);drop(proxy);
            let mut fresh=$session::connect(options.clone(),c.clone()).await.unwrap();let h=fresh.validate_command(&c,&op,&namespace,CommandId::new([120+i as u8;16]),&l,MAX_TTL).unwrap();let result=fresh.lookup_command(&c,&op,h).await.unwrap();
            if i==0{assert!(result.is_none());assert_eq!(sqlx::query("SELECT amount FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=2").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),before_network);}else{assert_eq!(result.unwrap().kind(),CommandResultKind::Replayed);assert_eq!(sqlx::query("SELECT amount FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=2").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),600+i as i64);}
            if i!=0 {let select=if d==Dialect::Postgres{"SELECT input,result,version FROM quent_dispatch_test.command_ledger WHERE identity=$1"}else{"SELECT input,result,version FROM quent_dispatch_test.command_ledger WHERE identity=?"};let row=sqlx::query(sqlx::AssertSqlSafe(select.to_owned())).bind(&key).fetch_one(&mut observer).await.unwrap();assert_eq!(row.get::<Vec<u8>,_>(0),network_input);assert_eq!(row.get::<Vec<u8>,_>(1),[b"QCR1".as_slice(),&[if d==Dialect::Postgres{1}else{2}],&1u64.to_be_bytes()].concat());assert_eq!(row.get::<i16,_>(2),1);}
            assert!(fresh.recorder.calls.is_empty());assert_eq!(sqlx::query(sqlx::AssertSqlSafe(count_query.to_owned())).bind(key).fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),i64::from(i!=0));
            println!("PR2 {:?} network case={} business1 commit_forwarded={} real_server_ack={} client_ack={} fresh_lookup_business0",d,120+i,i!=0,i!=0,i==2);
        }

        // Invalid installation/storage fixture has no CHECK/length protection:
        // exercise the production bounded read + strict decoder, never a mock.
        let corrupt=ledger("command_ledger_corrupt");
        let ddl=if d==Dialect::Postgres{"CREATE TABLE quent_dispatch_test.command_ledger_corrupt(identity BYTEA PRIMARY KEY,input BYTEA NULL,result BYTEA NULL,version SMALLINT NULL)"}else{"CREATE TABLE quent_dispatch_test.command_ledger_corrupt(identity VARBINARY(2048) PRIMARY KEY,input LONGBLOB NULL,result LONGBLOB NULL,version SMALLINT NULL) ENGINE=InnoDB"};
        sqlx::query(sqlx::AssertSqlSafe(ddl.to_owned())).execute(&mut observer).await.unwrap();
        let op=cu(2,777);let mut decoder=$session::connect(options.clone(),c.clone()).await.unwrap();
        let h=decoder.validate_command(&c,&op,&namespace,CommandId::new([125;16]),&corrupt,MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();let input=h.input.sensitive_bytes().to_vec();drop(h);
        let good_result=[b"QCR1".as_slice(),&[if d==Dialect::Postgres{1}else{2}],&1u64.to_be_bytes()].concat();
        let insert=if d==Dialect::Postgres{"INSERT INTO quent_dispatch_test.command_ledger_corrupt(identity,input,result,version) VALUES($1,$2,$3,1)"}else{"INSERT INTO quent_dispatch_test.command_ledger_corrupt(identity,input,result,version) VALUES(?,?,?,1)"};
        sqlx::query(sqlx::AssertSqlSafe(insert.to_owned())).bind(&key).bind(&input).bind(&good_result).execute(&mut observer).await.unwrap();
        let mut cases:Vec<(Option<Vec<u8>>,Option<Vec<u8>>,Option<i16>,ExecutionError)>=vec![];
        for bad in [b"QCC9".to_vec(),input[..input.len()-1].to_vec(),[input.as_slice(),&[255,0,0,0,0]].concat(),[input.as_slice(),&[1,0,0,0,2,0,0]].concat(),[input.as_slice(),&[9,0,0,0,1,2]].concat(),[input.as_slice(),&[10,0,0,0,1,255]].concat(),vec![0;1_048_577]] {cases.push((Some(bad),Some(good_result.clone()),Some(1),ExecutionError::LedgerInvalid));}
        // Structurally valid extra/duplicate semantic content is unequal to
        // EVERY byte of the current canonical record; no partial/hash compare.
        for bad in [[input.as_slice(),&[9,0,0,0,1,0]].concat(),[input.as_slice(),&input[4..]].concat()] {cases.push((Some(bad),Some(good_result.clone()),Some(1),ExecutionError::IdentityConflict));}
        for bad in [b"QCR9".to_vec(),good_result[..12].to_vec(),[good_result.as_slice(),&[0]].concat(),[b"QCR1".as_slice(),&[if d==Dialect::Postgres{2}else{1}],&1u64.to_be_bytes()].concat(),[b"QCR1".as_slice(),&[if d==Dialect::Postgres{1}else{2}],&2u64.to_be_bytes()].concat(),[b"QCR1".as_slice(),&[if d==Dialect::Postgres{1}else{2}],&0u64.to_be_bytes()].concat(),[b"QCR2".as_slice(),&[if d==Dialect::Postgres{1}else{2}],&1u64.to_be_bytes()].concat(),[b"QCR2".as_slice(),&[if d==Dialect::Postgres{1}else{2}],&0u64.to_be_bytes()].concat(),vec![0;10000]] {cases.push((Some(input.clone()),Some(bad),Some(1),ExecutionError::LedgerInvalid));}
        cases.extend([(None,Some(good_result.clone()),Some(1),ExecutionError::LedgerInvalid),(Some(input.clone()),None,Some(1),ExecutionError::ReservationUnresolved),(Some(input.clone()),Some(good_result.clone()),Some(2),ExecutionError::LedgerInvalid),(Some(input.clone()),Some(good_result.clone()),None,ExecutionError::LedgerInvalid)]);
        let update=if d==Dialect::Postgres{"UPDATE quent_dispatch_test.command_ledger_corrupt SET input=$1,result=$2,version=$3 WHERE identity=$4"}else{"UPDATE quent_dispatch_test.command_ledger_corrupt SET input=?,result=?,version=? WHERE identity=?"};
        let count=cases.len();
        for (input,result,version,cause) in cases {
            sqlx::query(sqlx::AssertSqlSafe(update.to_owned())).bind(input).bind(result).bind(version).bind(&key).execute(&mut observer).await.unwrap();
            let h=decoder.validate_command(&c,&op,&namespace,CommandId::new([125;16]),&corrupt,MAX_TTL).unwrap();let e=decoder.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.cause(),cause);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(decoder.recorder.calls.is_empty());assert_eq!(decoder.recorder.events,["begin","begun","ledger_lookup","rollback","rolled_back"]);decoder.recorder=Recorder::default();
        }
        sqlx::query(sqlx::AssertSqlSafe(update.to_owned())).bind(&input).bind(&good_result).bind(1i16).bind(&key).execute(&mut observer).await.unwrap();
        let h=decoder.validate_command(&c,&op,&namespace,CommandId::new([125;16]),&corrupt,MAX_TTL).unwrap();assert_eq!(decoder.execute_command(&c,&op,h).await.unwrap().kind(),CommandResultKind::Replayed);assert!(decoder.recorder.calls.is_empty());
        println!("PR2 {:?} storage counterexamples={} each business0/reserve0/ACKrollback; valid restored control Replayed business0",d,count);
        // Wrong installation: two rows with the same binary identity. LIMIT2
        // must reject before any individual matching record can authorize replay.
        let drop_key=if d==Dialect::Postgres{"ALTER TABLE quent_dispatch_test.command_ledger_corrupt DROP CONSTRAINT command_ledger_corrupt_pkey"}else{"ALTER TABLE quent_dispatch_test.command_ledger_corrupt DROP PRIMARY KEY"};sqlx::query(sqlx::AssertSqlSafe(drop_key.to_owned())).execute(&mut observer).await.unwrap();
        sqlx::query(sqlx::AssertSqlSafe(insert.to_owned())).bind(&key).bind(&input).bind(&good_result).execute(&mut observer).await.unwrap();
        let h=decoder.validate_command(&c,&op,&namespace,CommandId::new([125;16]),&corrupt,MAX_TTL).unwrap();let e=decoder.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerInvalid);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(decoder.recorder.calls.is_empty());
        sqlx::query("DROP TABLE quent_dispatch_test.command_ledger_corrupt").execute(&mut observer).await.unwrap();
        // INTEGER1 is decodable as i16 by SQLx MySQL's broad compatibility;
        // native ledger width checking must still refuse this valid payload.
        let ddl=if d==Dialect::Postgres{"CREATE TABLE quent_dispatch_test.command_ledger_corrupt(identity BYTEA PRIMARY KEY,input BYTEA,result BYTEA,version INTEGER)"}else{"CREATE TABLE quent_dispatch_test.command_ledger_corrupt(identity VARBINARY(2048) PRIMARY KEY,input LONGBLOB,result LONGBLOB,version INTEGER) ENGINE=InnoDB"};sqlx::query(sqlx::AssertSqlSafe(ddl.to_owned())).execute(&mut observer).await.unwrap();
        sqlx::query(sqlx::AssertSqlSafe(insert.to_owned())).bind(&key).bind(&input).bind(&good_result).execute(&mut observer).await.unwrap();
        let mut decoder=$session::connect(options.clone(),c.clone()).await.unwrap();
        let row=sqlx::query("SELECT version FROM quent_dispatch_test.command_ledger_corrupt").fetch_one(&mut observer).await.unwrap();if d==Dialect::Mysql{assert_eq!(row.try_get::<i16,_>(0).unwrap(),1);} // detecting control
        let h=decoder.validate_command(&c,&op,&namespace,CommandId::new([125;16]),&corrupt,MAX_TTL).unwrap();let e=decoder.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.cause(),ExecutionError::LedgerInvalid);assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(decoder.recorder.calls.is_empty());
        sqlx::query("DROP TABLE quent_dispatch_test.command_ledger_corrupt").execute(&mut observer).await.unwrap();
        println!("PR2 {:?} additional duplicate-key + wrong-native-width ledger cases: business0/ACKrollback; native decode control distinguished",d);


        // Actual pending future before business dispatch, with a test-only await
        // seam. Timeout/drop consumes the carrier; no acknowledged rollback claim.
        for timeout in [false,true] {
            let op=cu(1,9);let mut canceled=$session::connect(options.clone(),c.clone()).await.unwrap();canceled.final_barrier=Some(Arc::new(tokio::sync::Notify::new()));let trace=canceled.recorder.live.clone();let h=canceled.validate_command(&c,&op,&namespace,CommandId::new([70+u8::from(timeout);16]),&l,MAX_TTL).unwrap();let mut future=Box::pin(canceled.execute_command(&c,&op,h));
            tokio::time::timeout(Duration::from_secs(10),async{loop{assert!(poll_once(future.as_mut()).await.is_pending());if trace.lock().unwrap().contains(&"command_ready"){break;}tokio::task::yield_now().await;}}).await.unwrap();if timeout{assert!(tokio::time::timeout(Duration::from_millis(1),future.as_mut()).await.is_err());}drop(future);assert!(!canceled.is_available());assert!(!trace.lock().unwrap().contains(&"send"));assert!(!trace.lock().unwrap().contains(&"rolled_back"));drop(canceled);
        }
        // Begin/commit/rollback failures use existing TEST-ONLY completion seams.
        for (index,fault) in [AtomicFault::Begin,AtomicFault::Commit,AtomicFault::Rollback,AtomicFault::RollbackReplyLost].into_iter().enumerate(){let mut failed=$session::connect(options.clone(),c.clone()).await.unwrap();let op=if index<2{cu(1,9)}else{cu(999,9)};let h=failed.validate_command(&c,&op,&namespace,CommandId::new([80+index as u8;16]),&l,MAX_TTL).unwrap();failed.atomic_fault=Some(fault);let e=failed.execute_command(&c,&op,h).await.unwrap_err();assert_eq!(e.disposition(),AtomicDisposition::OutcomeUnknown);assert!(!failed.is_available());if index==0{assert!(failed.recorder.calls.is_empty());}drop(failed);}
        // Corrupt/unknown/pending ledger rows are never interpreted as missing.
        let op=ci(2,Value::Text(String::new()),false);let h=s.validate_command(&c,&op,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();let key=h.key.sensitive_bytes().to_vec();let input=h.input.sensitive_bytes().to_vec();drop(h);
        for mode in 0..3 {
            let (sql,value)=match mode {0=>(if d==Dialect::Postgres{"UPDATE quent_dispatch_test.command_ledger SET input=$1 WHERE identity=$2"}else{"UPDATE quent_dispatch_test.command_ledger SET input=? WHERE identity=?"},b"QCC9".to_vec()),1=>(if d==Dialect::Postgres{"UPDATE quent_dispatch_test.command_ledger SET result=$1 WHERE identity=$2"}else{"UPDATE quent_dispatch_test.command_ledger SET result=? WHERE identity=?"},vec![0;13]),_=>(if d==Dialect::Postgres{"UPDATE quent_dispatch_test.command_ledger SET input=$1,result=NULL WHERE identity=$2"}else{"UPDATE quent_dispatch_test.command_ledger SET input=?,result=NULL WHERE identity=?"},input.clone())};
            sqlx::query(sqlx::AssertSqlSafe(sql.to_owned())).bind(value).bind(&key).execute(&mut observer).await.unwrap();let h=s.validate_command(&c,&op,&namespace,CommandId::new([1;16]),&l,MAX_TTL).unwrap();let e=s.execute_command(&c,&op,h).await.unwrap_err();assert!(matches!(e.cause(),ExecutionError::LedgerInvalid|ExecutionError::ReservationUnresolved));assert_eq!(e.disposition(),AtomicDisposition::RolledBack);assert!(s.recorder.calls.is_empty());rollbacks+=1;s.recorder=Recorder::default();
            let restore=if d==Dialect::Postgres{"UPDATE quent_dispatch_test.command_ledger SET input=$1 WHERE identity=$2"}else{"UPDATE quent_dispatch_test.command_ledger SET input=? WHERE identity=?"};sqlx::query(sqlx::AssertSqlSafe(restore.to_owned())).bind(&input).bind(&key).execute(&mut observer).await.unwrap();
        }
        assert_eq!(sqlx::query("SELECT amount FROM quent_dispatch_test.command_items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<i64,_>(0),8);
        assert_eq!((provider.calls(),provider.responses()),(1,1));
        sqlx::query("DROP TABLE quent_dispatch_test.command_ledger_fail").execute(&mut observer).await.unwrap();
        println!("QT-AI-10 PR1 {:?}: first commits={}, new-session replay/lookup={} business0 each, confirmed rollback={} + same-generation conflict; independent business/ledger observer; provider calls/responses=0 (no provider entry)",d,executions,replays,rollbacks);
        sqlx::query("DROP TABLE quent_dispatch_test.command_ledger").execute(&mut observer).await.unwrap();sqlx::query("DROP TABLE quent_dispatch_test.command_items").execute(&mut observer).await.unwrap();
    }));
    #[cfg(feature="output-tracing")] {drop(subscriber);assert!(capture.0.lock().unwrap().is_empty(),"implicit command/driver tracing");let positive=output_capture::Capture::default();tracing::subscriber::with_default(positive.clone(),||tracing::warn!("CANARY_command_detector"));assert!(positive.0.lock().unwrap().iter().any(|s|s.contains("CANARY_command_detector")));}
}
};}
command_suite!(
    live_command_postgres,
    "postgres",
    PostgresSession,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    "../../../../../fixtures/command-ledger/postgres.sql",
    sqlx::postgres::PgSslMode::Disable
);
command_suite!(
    live_command_mysql,
    "mysql",
    MysqlSession,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "../../../../../fixtures/command-ledger/mysql.sql",
    sqlx::mysql::MySqlSslMode::Disabled
);
