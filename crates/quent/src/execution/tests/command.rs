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
($test:ident,$feature:literal,$session:ident,$connection:ty,$options:ty,$dialect:expr,$dsn:literal,$ddl:literal) => {
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
    "../../../../../fixtures/command-ledger/postgres.sql"
);
command_suite!(
    live_command_mysql,
    "mysql",
    MysqlSession,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "../../../../../fixtures/command-ledger/mysql.sql"
);
