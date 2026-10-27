use super::*;
#[test]
fn observation_budget_bounds_and_closed_public_register() {
    assert!(ObservationBudget::new(4097, 1, Duration::from_secs(1), 2).is_err());
    for entries in [0, 129] {
        assert!(ObservationBudget::new(1, entries, Duration::from_secs(1), 2).is_err());
    }
    for duration in [Duration::ZERO, Duration::from_secs(301)] {
        assert!(ObservationBudget::new(1, 1, duration, 2).is_err());
    }
    for repeat in [0, 1, 129] {
        assert!(ObservationBudget::new(1, 1, Duration::from_secs(1), repeat).is_err());
    }
    let c = context(Dialect::Postgres, 11, "CANARY_observation_subject", vec![]);
    let collector = RequestCollector::new(
        &c,
        ObservationBudget::new(0, 1, Duration::from_secs(1), 2).unwrap(),
    );
    let evidence = collector.finish();
    let view = evidence.public_view();
    check_public(&view);
    for e in [
        ExecutionError::DispatchBudget,
        ExecutionError::ObservationLimit,
        ExecutionError::ObservationClosed,
    ] {
        assert!(std::error::Error::source(&e).is_none());
        assert!(!format!("{e:?} {e}").contains("CANARY"));
    }
    assert!(!format!("{evidence:?}").contains("CANARY"));
    // Artificial lifecycle control for a pending/forgotten attempt, no DB evidence.
    let r = RequestCollector::new(
        &c,
        ObservationBudget::new(1, 2, Duration::from_secs(1), 2).unwrap(),
    );
    let link = crate::execution::observation::attach(&r.shared, &c).unwrap();
    let second_link = crate::execution::observation::attach(&r.shared, &c).unwrap();
    let attempt =
        crate::execution::observation::Attempt::start(&Some(link), ObservedTransaction::None)
            .unwrap()
            .unwrap();
    assert!(r.public_view().is_incomplete());
    assert!(matches!(
        crate::execution::observation::Attempt::start(
            &Some(second_link),
            ObservedTransaction::None
        ),
        Err(ExecutionError::ObservationClosed)
    ));
    assert!(matches!(
        crate::execution::observation::attach(&r.shared, &c),
        Err(ExecutionError::ObservationClosed)
    ));
    drop(attempt);
    assert_eq!(
        r.public_view().entries()[0].status,
        ObservationStatus::Interrupted
    );
}
pub(super) fn check_public(view: &ObservationView) {
    let text = view.to_json();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    // Detection positive control independent of the producer's keys/encoder.
    let detect = |s: &str| {
        [
            "CANARY",
            "quent_dispatch_test",
            "items",
            "subject",
            "SELECT ",
            "tenant",
            "hash",
            "sql",
            "target",
            "path",
            "column",
        ]
        .iter()
        .any(|n| s.contains(n))
    };
    assert!(detect("CANARY_secret SELECT tenant FROM items"));
    assert!(!detect(&text));
    let top = v.as_object().unwrap();
    let mut keys: Vec<_> = top.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "budget_refused",
            "capacity_exceeded",
            "closed",
            "entries",
            "incomplete",
            "kind",
            "submissions",
            "version"
        ]
    );
    assert_eq!(v["kind"], "quent_observation_view");
    assert_eq!(v["version"], 1);
    assert!(v["submissions"].as_u64().unwrap() <= 4096);
    for e in v["entries"].as_array().unwrap() {
        let mut keys: Vec<_> = e.as_object().unwrap().keys().map(String::as_str).collect();
        keys.sort_unstable();
        assert_eq!(
            keys,
            [
                "begin_acks",
                "begin_attempts",
                "commit_acks",
                "commit_attempts",
                "consumption",
                "decoded_bytes",
                "decoded_rows",
                "duration_micros",
                "duration_precision",
                "n_plus_one_suspected",
                "repeat_count",
                "rollback_acks",
                "rollback_attempts",
                "status",
                "submitted",
                "transaction"
            ]
        );
        assert!(
            ["Pending", "Succeeded", "Failed", "Refused", "Interrupted"]
                .contains(&e["status"].as_str().unwrap())
        );
        assert!(
            ["NotObserved", "Partial", "Complete"].contains(&e["consumption"].as_str().unwrap())
        );
        assert!(
            [
                "None",
                "BorrowedUnconfirmed",
                "NotStarted",
                "Unknown",
                "Committed",
                "RolledBack"
            ]
            .contains(&e["transaction"].as_str().unwrap())
        );
        assert!(["Exact", "CeilingExceeded"].contains(&e["duration_precision"].as_str().unwrap()));
        for k in [
            "begin_acks",
            "begin_attempts",
            "commit_acks",
            "commit_attempts",
            "rollback_acks",
            "rollback_attempts",
        ] {
            assert!(e[k].as_u64().unwrap() <= 1);
        }
        assert!(e["duration_micros"].as_u64().unwrap() <= 300_000_000);
        assert!(e["decoded_rows"].as_u64().unwrap() <= 256);
        assert!(e["decoded_bytes"].as_u64().unwrap() <= 1_048_576);
        assert!(e["repeat_count"].as_u64().unwrap() <= 128);
        for k in ["submitted", "n_plus_one_suspected"] {
            assert!(e[k].is_boolean());
        }
    }
    assert!(!detect(&format!("{view:?} {view}")));
}
macro_rules! observation_cases {
 ($session:ident,$borrowed:ident,$connection:ty,$options:ident,$d:ident,$c:ident) => {{
    let d=$d;let c=&$c;let options=&$options;
    let limits=|n,cap|ObservationBudget::new(n,cap,Duration::from_secs(300),2).unwrap();
    let op=||select(d).filter(filter(1)).limit(10).freeze();
    let mut observer=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();
    if d==Dialect::Mysql {assert_eq!(sqlx::query("SELECT ENGINE FROM information_schema.tables WHERE TABLE_SCHEMA='quent_dispatch_test' AND TABLE_NAME='items'").fetch_one(&mut observer).await.unwrap().get::<String,_>(0),"InnoDB");}
    let data_before=sqlx::query("SELECT id,tenant,txt FROM quent_dispatch_test.items ORDER BY tenant,id").fetch_all(&mut observer).await.unwrap().into_iter().map(|r|(r.get::<i64,_>(0),r.get::<i64,_>(1),r.get::<String,_>(2))).collect::<Vec<_>>();
    let catalog_sql=if d==Dialect::Postgres {"SELECT column_name,data_type,is_nullable FROM information_schema.columns WHERE table_schema='quent_dispatch_test' AND table_name='items' ORDER BY ordinal_position"}else{"SELECT COLUMN_NAME,DATA_TYPE,IS_NULLABLE FROM information_schema.COLUMNS WHERE TABLE_SCHEMA='quent_dispatch_test' AND TABLE_NAME='items' ORDER BY ORDINAL_POSITION"};
    let catalog_before=sqlx::query(sqlx::AssertSqlSafe(catalog_sql.to_owned())).fetch_all(&mut observer).await.unwrap().into_iter().map(|r|(r.get::<String,_>(0),r.get::<String,_>(1),r.get::<String,_>(2))).collect::<Vec<_>>();
    // Actual public scopes: zero, just below, at limit and rejected next send.
    for n in [0,1,2] {
        let mut s=$session::connect(options.clone(),c.clone()).await.unwrap();
        let mut request=RequestCollector::new(c,limits(n,8));
        {
            let mut scoped=s.observe(&mut request).unwrap();
            for _ in 0..n { assert_eq!(scoped.execute(c,op()).await.unwrap().sensitive_rows().len(),1); }
            assert_eq!(scoped.execute(c,op()).await.unwrap_err(),ExecutionError::DispatchBudget);
        }
        assert_eq!(s.recorder.calls.len(),usize::from(n));
        let v=request.public_view();assert_eq!(v.submissions(),n);assert!(v.is_closed());
        assert_eq!(v.entries().last().unwrap().status,ObservationStatus::Refused);assert!(!v.entries().last().unwrap().submitted);
        check_public(&v);
        assert_eq!(s.observe(&mut request).unwrap_err(),ExecutionError::ObservationClosed);
    }
    // Independently new request starts at zero, not a refund/reset of the old one.
    let mut s=$session::connect(options.clone(),c.clone()).await.unwrap();
    let mut request=RequestCollector::new(c,limits(8,8));
    {
        let mut scoped=s.observe(&mut request).unwrap();
        let first=op();let handle=scoped.validate(c,&first,MAX_TTL).unwrap();
        let f=scoped.execute_validated(c,&first,handle);drop(f); // no poll/no entry/no submission
        scoped.execute(c,first).await.unwrap();
        scoped.execute(c,select(d).filter(filter(99)).limit(1).freeze()).await.unwrap(); // different values, same exact shape
        scoped.execute(c,Draft::select(table(),vec![Projection::new(key(),None)]).filter(filter(1)).freeze()).await.unwrap(); // different projection
    }
    let v=request.public_view();assert_eq!(v.submissions(),3);assert_eq!(v.entries().len(),3);
    assert_eq!(v.entries().iter().map(|e|e.repeat_count).collect::<Vec<_>>(),[1,2,1]);
    assert!(v.entries()[1].n_plus_one_suspected);assert!(!v.entries()[2].n_plus_one_suspected);
    assert!(v.entries().iter().all(|e|e.consumption==Consumption::Complete));check_public(&v);
    // Exact snapshot mismatches on scope attachment: target, tenant and subject.
    for wrong in [context_variant(d,11,"subject",vec![],1),context(d,22,"subject",vec![]),context(d,11,"CANARY_other_subject",vec![])] {
        let mut bad=RequestCollector::new(&wrong,limits(4,4));
        assert_eq!(s.observe(&mut bad).unwrap_err(),ExecutionError::ContextMismatch);assert_eq!(bad.public_view().submissions(),0);
    }
    let mut independent=RequestCollector::new(c,limits(1,1));
    s.observe(&mut independent).unwrap().execute(c,op()).await.unwrap();assert_eq!(independent.public_view().entries()[0].repeat_count,1);
    // Capacity overflow is explicit and refuses instead of losing an observation.
    let mut capacity=RequestCollector::new(c,limits(4,1));
    {let mut scoped=s.observe(&mut capacity).unwrap();scoped.execute(c,op()).await.unwrap();assert_eq!(scoped.execute(c,op()).await.unwrap_err(),ExecutionError::ObservationLimit);}
    assert_eq!(capacity.public_view().submissions(),1);assert!(capacity.public_view().is_incomplete());check_public(&capacity.public_view());
    // Real monotonic measurement ceiling is observation-only: a completed read
    // stays successful, then subsequent request use is closed (no server deadline).
    let mut duration=RequestCollector::new(c,ObservationBudget::new(2,2,Duration::from_nanos(1),2).unwrap());
    s.observe(&mut duration).unwrap().execute(c,op()).await.unwrap();
    assert_eq!(duration.public_view().entries()[0].status,ObservationStatus::Succeeded);
    assert_eq!(duration.public_view().entries()[0].duration_precision,DurationPrecision::CeilingExceeded);
    assert!(duration.public_view().is_closed());assert!(duration.public_view().is_incomplete());
    // Exclusive expiry exactly at final boundary: existing private clock seam,
    // not a claim of scheduling/network time control; charge must remain zero.
    let operation=op();let h=s.validate(c,&operation,MAX_TTL).unwrap();s.final_now=Some(h.expires);
    let mut expired=RequestCollector::new(c,limits(2,2));
    assert_eq!(s.observe(&mut expired).unwrap().execute_validated(c,&operation,h).await.unwrap_err(),ExecutionError::Expired);
    assert_eq!(expired.public_view().submissions(),0);assert_eq!(expired.public_view().entries()[0].status,ObservationStatus::Refused);
    // Native SQL/args remain identical with collection selected; collector I/O0.
    let mut baseline=$session::connect(options.clone(),c.clone()).await.unwrap();baseline.execute(c,op()).await.unwrap();
    let mut enabled=$session::connect(options.clone(),c.clone()).await.unwrap();
    let mut exact=RequestCollector::new(c,limits(2,2));enabled.observe(&mut exact).unwrap().execute(c,op()).await.unwrap();
    assert_eq!(baseline.recorder.calls,enabled.recorder.calls);assert_eq!(baseline.recorder.events,enabled.recorder.events);
    #[cfg(feature="operation-spec")]
    {
        let mapping=spec_mapping(c,d);let compiled=read_spec(d).compile(&mapping,c).unwrap();
        let mut request=RequestCollector::new(c,limits(1,2));
        assert_eq!(enabled.observe(&mut request).unwrap().execute_compiled_read(c,&mapping,compiled).await.unwrap().sensitive_rows().len(),1);
        assert_eq!(request.public_view().submissions(),1);assert_eq!(request.public_view().entries()[0].consumption,Consumption::Complete);
    }
    // Forgetting the public scope must retain enforcement; finishing seals
    // private evidence, so the original session cannot append to a finished request.
    let mut sealed=RequestCollector::new(c,limits(2,2));
    let scoped=enabled.observe(&mut sealed).unwrap();std::mem::forget(scoped);
    let evidence=sealed.finish();assert!(evidence.public_view().is_closed());
    let sent=enabled.recorder.calls.len();assert_eq!(enabled.execute(c,op()).await.unwrap_err(),ExecutionError::ObservationClosed);assert_eq!(enabled.recorder.calls.len(),sent);
    // Owned Tx success, expectation rollback, pre-business budget refusal after begin.
    let before=sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<String,_>(0);
    for (budget,expected,tx) in [(1,1,ObservedTransaction::Committed),(1,0,ObservedTransaction::RolledBack),(0,1,ObservedTransaction::RolledBack)] {
        let mut s=$session::connect(options.clone(),c.clone()).await.unwrap();let mut request=RequestCollector::new(c,limits(budget,2));
        let e=ExpectedRows::exactly(expected).unwrap();let operation=Draft::update(table(),write(Value::Text(format!("CANARY_observation_{budget}_{expected}")),changed())).filter(filter(1)).freeze();
        {let mut scope=s.observe(&mut request).unwrap();let h=scope.validate_atomic(c,&operation,e,MAX_TTL).unwrap();let r=scope.execute_atomic(c,&operation,e,h).await;
        if expected==1 && budget==1 {assert_eq!(r.unwrap().state(),ResultState::Committed);}else{let failure=r.unwrap_err();assert_eq!(failure.disposition(),AtomicDisposition::RolledBack);assert_eq!(failure.cause(),if budget==0{ExecutionError::DispatchBudget}else{ExecutionError::ExpectationMismatch});}}
        let v=request.public_view();assert_eq!(v.submissions(),budget);let entry=&v.entries()[0];assert_eq!(entry.transaction,tx);assert_eq!((entry.begin_attempts,entry.begin_acks),(1,1));assert_eq!(entry.commit_acks,u8::from(tx==ObservedTransaction::Committed));assert_eq!(entry.rollback_acks,u8::from(tx==ObservedTransaction::RolledBack));
        assert_eq!(s.recorder.calls.len(),usize::from(budget));check_public(&v);
        let value=sqlx::query("SELECT txt FROM quent_dispatch_test.items WHERE tenant=11 AND id=1").fetch_one(&mut observer).await.unwrap().get::<String,_>(0);
        assert_eq!(value,"CANARY_observation_1_1"); // distinguish the rejected write's different value; no refund
    }
    sqlx::query(if d==Dialect::Postgres {"UPDATE quent_dispatch_test.items SET txt=$1 WHERE tenant=11 AND id=1"}else{"UPDATE quent_dispatch_test.items SET txt=? WHERE tenant=11 AND id=1"}).bind(&before).execute(&mut observer).await.unwrap();
    // Borrowed Tx remains unconfirmed, and caller rollback is outside observation.
    let mut raw=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut tx=raw.begin().await.unwrap();
    let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();let mut request=RequestCollector::new(c,limits(1,2));
    borrowed.observe(&mut request).unwrap().execute(c,op()).await.unwrap();drop(borrowed);tx.rollback().await.unwrap();
    assert_eq!(request.public_view().entries()[0].transaction,ObservedTransaction::BorrowedUnconfirmed);
    assert_eq!(request.public_view().entries()[0].rollback_attempts,0);
    // A retained borrowed attachment must close after an unsupported execution,
    // even through a deliberately forgotten scope; caller Tx is still untouched.
    let unsupported_write=Draft::update(table(),write(Value::Text("CANARY_unsupported".into()),changed())).filter(filter(1)).freeze();
    let namespace=CommandNamespace::new(b"observation".to_vec()).unwrap();
    let ledger=CommandLedger::application_supplied(Table::new(vec![id("quent_dispatch_test"),id("observation_ledger_never_installed")],None).unwrap()).unwrap();
    let h=baseline.validate_command(c,&unsupported_write,&namespace,CommandId::new([19;16]),&ledger,MAX_TTL).unwrap();
    let mut raw=<$connection>::connect_with(&options.clone().disable_statement_logging()).await.unwrap();let mut tx=raw.begin().await.unwrap();
    let mut borrowed=$borrowed::new(&mut tx,c.clone()).unwrap();let mut request=RequestCollector::new(c,limits(2,2));
    std::mem::forget(borrowed.observe(&mut request).unwrap());
    let failure=borrowed.execute_command(c,&unsupported_write,h).await.unwrap_err();assert_eq!(failure.cause(),ExecutionError::Unsupported);assert_eq!(failure.disposition(),AtomicDisposition::NotStarted);
    assert!(request.public_view().is_closed());assert!(request.public_view().is_incomplete());assert_eq!(request.public_view().submissions(),0);assert!(borrowed.recorder.calls.is_empty());assert!(borrowed.recorder.events.is_empty());
    assert_eq!(borrowed.execute(c,op()).await.unwrap_err(),ExecutionError::ObservationClosed);
    drop(borrowed);tx.rollback().await.unwrap();
    // Paired same-session baseline/collector measurements. All reads fully consumed,
    // all updates exact matched1 and Tx committed. No setup I/O inside measured span.
    let mut bench=$session::connect(options.clone(),c.clone()).await.unwrap();
    let bench_write=Draft::update(table(),write(Value::Text(before.clone()),changed())).filter(filter(1)).freeze();
    let expected_rows=bench.execute(c,op()).await.unwrap().sensitive_rows().to_vec();
    let one=ExpectedRows::exactly(1).unwrap();let mut measurements: [Vec<u128>;2]=[Vec::new(),Vec::new()];
    for round in 0..13 {
        for enabled in if round%2==0 {[false,true]}else{[true,false]} {
            let count_before=bench.recorder.calls.len();let mut request=RequestCollector::new(c,limits(32,32));
            let start=Instant::now();
            if enabled {
                let mut scope=bench.observe(&mut request).unwrap();
                for _ in 0..10 {let r=scope.execute(c,op()).await.unwrap();assert_eq!(r.sensitive_rows(),expected_rows);let h=scope.validate_atomic(c,&bench_write,one,MAX_TTL).unwrap();assert_eq!(scope.execute_atomic(c,&bench_write,one,h).await.unwrap().state(),ResultState::Committed);}
            } else {
                for _ in 0..10 {let r=bench.execute(c,op()).await.unwrap();assert_eq!(r.sensitive_rows(),expected_rows);let h=bench.validate_atomic(c,&bench_write,one,MAX_TTL).unwrap();assert_eq!(bench.execute_atomic(c,&bench_write,one,h).await.unwrap().state(),ResultState::Committed);}
            }
            let ns=start.elapsed().as_nanos();assert_eq!(bench.recorder.calls.len()-count_before,20);
            if enabled {assert_eq!(request.public_view().submissions(),20);assert!(request.public_view().entries().iter().all(|e|e.status==ObservationStatus::Succeeded));}
            if round>=4 {measurements[usize::from(enabled)].push(ns);}
            bench.recorder=Recorder::default(); // retain the original finite per-recorder cap; each round asserted20

        }
    }
    for (i,raw) in measurements.iter().enumerate() {let mut sorted=raw.clone();sorted.sort_unstable();let median=sorted[sorted.len()/2];let mut deviations=raw.iter().map(|v|v.abs_diff(median)).collect::<Vec<_>>();deviations.sort_unstable();eprintln!("C21 observation benchmark {:?} enabled={} warmup_pairs=4 repeats=9 reads=10 owned_commits=10 raw_ns={:?} median_ns={} min_ns={} max_ns={} mad_ns={}",d,i==1,raw,median,sorted[0],sorted[8],deviations[4]);}
    let data_after=sqlx::query("SELECT id,tenant,txt FROM quent_dispatch_test.items ORDER BY tenant,id").fetch_all(&mut observer).await.unwrap().into_iter().map(|r|(r.get::<i64,_>(0),r.get::<i64,_>(1),r.get::<String,_>(2))).collect::<Vec<_>>();assert_eq!(data_before,data_after);
    let catalog_after=sqlx::query(sqlx::AssertSqlSafe(catalog_sql.to_owned())).fetch_all(&mut observer).await.unwrap().into_iter().map(|r|(r.get::<String,_>(0),r.get::<String,_>(1),r.get::<String,_>(2))).collect::<Vec<_>>();assert_eq!(catalog_before,catalog_after);
    eprintln!("C21 observation {:?}: business counts0/1/2 limits; capacity1 then next0; exact SQL/native args and events baseline identical; provider/aux/EXPLAIN0; Tx begin/ACK/commit or rollback measured separately; independent data/catalog unchanged",d);
 }};
}
