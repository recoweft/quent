async fn poll_once<F: std::future::Future>(
    mut f: std::pin::Pin<&mut F>,
) -> std::task::Poll<F::Output> {
    std::future::poll_fn(|cx| std::task::Poll::Ready(f.as_mut().poll(cx))).await
}
use super::*;
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
        {
            let mut cs = fields(d);
            if variant == 3 {
                cs.reverse();
            }
            cs
        },
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

// These tests fail if explicitly selected without dedicated target opt-in. Normal
// DB-free CI commands explicitly exclude live_; required live CI selects them.
macro_rules! live_suite {
    ($name:ident, $feature:literal, $session:ident, $connection:ty, $options:ty, $dialect:expr, $dsn:literal, $create:literal, $seed:literal) => {
        #[cfg(feature=$feature)]
        #[test]
        fn $name() {
            use sqlx::{Connection, ConnectOptions, Row};
            assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(), Ok("quent_dispatch_test"), "dedicated disposable DB opt-in required");
            let dsn = std::env::var($dsn).expect("explicit dedicated DB DSN required");
            let options: $options = dsn.parse().expect("invalid dedicated test DSN");
            let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            // Setup is deliberately separate raw SQL, never counted as dispatch.
            runtime.block_on(async {
                let mut setup = <$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
                sqlx::query($create).execute(&mut setup).await.unwrap();
                sqlx::query($seed).execute(&mut setup).await.unwrap();
            });
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(||runtime.block_on(async {
                let d = $dialect; let c = context(d,11,"subject",vec![]);
                let mut session = $session::connect(options.clone(),c.clone()).await.unwrap();
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
                assert!(std::error::Error::source(&e).is_none());
            })));
            // Cleanup executes even after an assertion panic. Container wrapper also
            // removes its dedicated servers on command failure.
            runtime.block_on(async { let mut setup=<$connection>::connect_with(&options.disable_statement_logging()).await.unwrap(); sqlx::query("DROP TABLE IF EXISTS quent_dispatch_test.items").execute(&mut setup).await.unwrap(); });
            if let Err(panic) = result { std::panic::resume_unwind(panic); }
        }
    }
}
live_suite!(
    live_postgres,
    "postgres",
    PostgresSession,
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
        rows: vec![vec![Value::Text(
            "CANARY_SQL_identifier_reason_value".into(),
        )]],
        count: None,
        affected: None,
    };
    assert_eq!(format!("{r:?}"), "ExecutionResult([REDACTED])");
    for e in [
        ExecutionError::Connect,
        ExecutionError::ReadFailed,
        ExecutionError::Decode,
        ExecutionError::OutcomeUnknown,
    ] {
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
        assert!(recheck(&h, &identity, &c, &c, &op).is_ok());
        let gigantic = select(d)
            .filter(Predicate::And(vec![filter(1); 1025]))
            .freeze();
        assert_eq!(
            recheck(&h, &identity, &c, &c, &gigantic),
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
                recheck(&write_h, &identity, &c, &c, &different),
                Err(ExecutionError::OperationMismatch)
            );
        }
    }
}
