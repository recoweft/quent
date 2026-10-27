use super::super::{
    preflight::*,
    tests::{budget, context_at},
};
use super::*;
use quent_core::{Ident, SqlType, Value, migration::*};
use std::{
    cell::Cell,
    num::NonZeroU64,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};
fn nz(v: u64) -> NonZeroU64 {
    NonZeroU64::new(v).unwrap()
}
fn plan(d: Dialect, ns: &str, address: &str, tail: &str) -> MigrationPlan {
    let table = |after: bool| {
        let mut cols = vec![
            MigrationColumn::new(
                Ident::new("a").unwrap(),
                Observed::Known(SqlType::I64),
                Observed::Known(false),
                MigrationDefault::Literal(Value::I64(7)),
            )
            .unwrap(),
        ];
        if after {
            for n in ["tail_one", tail] {
                cols.push(
                    MigrationColumn::new(
                        Ident::new(n).unwrap(),
                        Observed::Known(SqlType::I32),
                        Observed::Known(true),
                        MigrationDefault::Absent,
                    )
                    .unwrap(),
                );
            }
        }
        MigrationTable::new(
            MigrationName::new(Ident::new(ns).unwrap(), Ident::new("apply_probe").unwrap()),
            TableEffects::Ordinary,
            TableCoverage {
                columns: Knowledge::Complete,
                indexes: Knowledge::Complete,
                foreign_keys: Knowledge::Complete,
                checks: Knowledge::Complete,
            },
            cols,
            vec![],
            vec![],
            vec![],
        )
        .unwrap()
    };
    let schema = |after| {
        MigrationSchema::new(
            MigrationTarget::new(
                "probe".into(),
                address.into(),
                Ident::new("quent_dispatch_test").unwrap(),
                if d == Dialect::Postgres {
                    quent_core::Dialect::Postgres
                } else {
                    quent_core::Dialect::Mysql
                },
            )
            .unwrap(),
            SchemaAuthority::ApplicationDeclared,
            Knowledge::Complete,
            vec![table(after)],
        )
        .unwrap()
    };
    plan_migration(&schema(false), &schema(true))
}
fn request<'p>(p: &'p MigrationPlan, ns: &str, role: &str) -> PreflightRequest<'p> {
    PreflightRequest::new(
        p,
        MigrationPhase::Expand,
        Scope::new(ns, &["apply_probe"]).unwrap(),
        Visibility::new(role, nz(1)).unwrap(),
        DdlExclusion::ApplicationDeclared(nz(1)),
    )
    .unwrap()
}
struct Adapter {
    calls: Arc<AtomicUsize>,
    mode: usize,
    saved: Mutex<Option<ApprovalDecision>>,
}
impl TrustedMigrationAdapter for Adapter {
    fn authorize<'a>(
        &'a self,
        r: ApprovalRequest<'a>,
    ) -> Pin<Box<dyn Future<Output = ApprovalDecision> + Send + 'a>> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            assert_eq!(r.requirements(), &REQUIREMENTS);
            assert_eq!(r.sensitive_plan().public_view().steps(), 2);
            let mut a = *r.sensitive_assertions();
            let mut h = r.sensitive_human_instruction();
            if self.mode < 5 {
                a[self.mode].state = AttestedState::Unknown;
            }
            if self.mode == 5 {
                h.state = HumanState::Denied;
            }
            let decision = r.respond(a, h);
            if self.mode == 6 {
                let mut saved = self.saved.lock().unwrap();
                if let Some(old) = saved.take() {
                    return old;
                }
                // A stale invocation token is deliberately injected by the fixture.
                return ApprovalDecision {
                    invocation: Arc::new(()),
                    ..decision
                };
            }
            decision
        })
    }
}
fn registered(c: DatabaseContext, mode: usize) -> (ApplyContext, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    (
        ApplyContext::register(
            c,
            "CANARY_migration_adapter".into(),
            nz(1),
            b"CANARY_configuration".to_vec(),
            Adapter {
                calls: calls.clone(),
                mode,
                saved: Mutex::new(None),
            },
        )
        .unwrap(),
        calls,
    )
}
fn current<'p>(c: ApplyContext, s: SessionIdentity, r: PreflightRequest<'p>) -> Current<'p> {
    let end = Instant::now() + Duration::from_secs(30);
    Current::new(
        c,
        s,
        &ApplyRequest::new("CANARY_request".into()).unwrap(),
        r,
        [Assertion::new(AttestedState::ApplicationAttested, nz(1), end); 5],
        HumanInstruction::new(HumanState::ExplicitlyAuthorized, nz(1), end),
    )
    .unwrap()
}
struct Source<'p> {
    base: Current<'p>,
    trace: Arc<Mutex<Vec<&'static str>>>,
    change_after: Cell<Option<&'static str>>,
    calls: Cell<usize>,
}
impl<'p> CurrentSource<'p> for Source<'p> {
    fn current(&self) -> ApplyResult<Current<'p>> {
        self.calls.set(self.calls.get() + 1);
        let mut c = self.base.retained();
        if self
            .change_after
            .get()
            .is_some_and(|event| self.trace.lock().unwrap().contains(&event))
        {
            c.assertions[4].generation = nz(2);
        }
        Ok(c)
    }
}
fn record(case: &str, trace: &Mutex<Vec<&'static str>>, out: ApplyOutcome) {
    let t = trace.lock().unwrap();
    let count = |e| t.iter().filter(|v| **v == e).count();
    eprintln!(
        "C22/{case}: connect={} SELECT={}/{} observe_BEGIN={}/{} observe_ROLLBACK={}/{} provider={}/{} apply_BEGIN={}/{} DDL={}/{} rejected_response={} COMMIT={}/{} apply_ROLLBACK={}/{} outcome={out:?}",
        count("connected"),
        count("catalog_select"),
        count("catalog_complete"),
        count("begin_attempt"),
        count("begun"),
        count("rollback_attempt"),
        count("rolled_back"),
        count("migration_provider_attempt"),
        count("migration_provider_ack"),
        count("apply_begin_attempt"),
        count("apply_begin_ack"),
        count("ddl_attempt"),
        count("ddl_ack"),
        count("ddl_rejected"),
        count("commit_attempt"),
        count("commit_ack"),
        count("apply_rollback_attempt"),
        count("apply_rollback_ack")
    );
}
#[test]
fn c22_application_attestation_current_and_output_contract() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let ns = if d == Dialect::Postgres {
            "public"
        } else {
            "quent_dispatch_test"
        };
        let c = context_at(
            d,
            1,
            "host:1",
            "quent_dispatch_test",
            Some("CANARY_subject"),
        );
        let (context, _) = registered(c, 99);
        let p = plan(d, ns, "host:1", "tail_two");
        let v = current(
            context,
            SessionIdentity(Arc::new(())),
            request(&p, ns, "CANARY_role"),
        );
        let deadline = Instant::now() + Duration::from_secs(10);
        assert!(v.valid(deadline).is_ok());
        for index in 0..5 {
            for state in [AttestedState::Unknown, AttestedState::Denied] {
                let mut changed = v.retained();
                changed.assertions[index].state = state;
                assert_eq!(changed.valid(deadline), Err(ApplyError::Attestation));
                assert!(!v.same(&changed));
            }
        }
        for state in [HumanState::Unknown, HumanState::Denied] {
            let mut changed = v.retained();
            changed.human.state = state;
            assert_eq!(changed.valid(deadline), Err(ApplyError::HumanDenied));
        }
        let mut expired = v.retained();
        expired.assertions[2].expires = Instant::now();
        assert_eq!(expired.valid(deadline), Err(ApplyError::Expired));
        assert_eq!(v.valid(Instant::now()), Err(ApplyError::Expired));
        let other = plan(d, ns, "host:1", "different_tail");
        let mut changed = v.retained();
        changed.request = request(&other, ns, "CANARY_role");
        assert!(!v.same(&changed));
        let mut changed = v.retained();
        changed.request.phase = MigrationPhase::Contract;
        assert!(!v.same(&changed));
        let mut changed = v.retained();
        changed.request.visibility.generation = nz(2);
        assert!(!v.same(&changed));
        let mut changed = v.retained();
        changed.request.exclusion = DdlExclusion::ApplicationDeclared(nz(2));
        assert!(!v.same(&changed));
        let mut same_label = v.retained();
        same_label.request_identity = ApplyRequest::new(v.request_id.clone()).unwrap().identity;
        assert!(
            !v.same(&same_label),
            "same textual request does not restore operation identity"
        );
        let mut changed = v.retained();
        changed.request_id = "another".into();
        assert!(!v.same(&changed));
        let mut changed = v.retained();
        changed.session = SessionIdentity(Arc::new(()));
        assert!(!v.same(&changed));
        let mut changed = v.retained();
        changed.context = changed
            .context
            .with_database(context_at(
                d,
                1,
                "host:1",
                "quent_dispatch_test",
                Some("other_subject"),
            ))
            .unwrap();
        assert!(!v.same(&changed));
        let mut changed = v.retained();
        changed.context = ApplyContext::without_adapter(v.context.database.clone());
        assert_eq!(changed.valid(deadline), Err(ApplyError::MissingAdapter));
        assert!(v.request_id.contains("CANARY"));
        assert!(v.context.database.sensitive_source().subject().is_some());
        for text in [
            format!("{v:?} {:?} {:?} {:?}", v.context, v.human, v.assertions),
            format!("{}", ApplyError::Mismatch),
        ] {
            assert!(!text.contains("CANARY"));
            assert!(!text.contains("ALTER TABLE"));
        }
        let out = ApplyOutcome::new(2);
        let json = out.to_json();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(value.as_object().unwrap().len(), 15);
        assert_eq!(value["live_coverage"], "Partial");
        assert_eq!(value["full_effects"], "UnverifiedByQuent");
        assert_eq!(value["human_trust_source"], "ApplicationAdapter");
        assert!(!json.contains("CANARY"));
        let sensitive = p.sensitive_source();
        let (before, after) = sensitive.schemas();
        let destructive = plan_migration(after, before);
        assert_eq!(destructive.public_view().status(), MigrationStatus::Refused);
        assert!(
            PreflightRequest::new(
                &destructive,
                MigrationPhase::Expand,
                Scope::new(ns, &["apply_probe"]).unwrap(),
                Visibility::new("role", nz(1)).unwrap(),
                DdlExclusion::ApplicationDeclared(nz(1))
            )
            .is_err()
        );
        use std::error::Error;
        assert!(ApplyError::Attestation.source().is_none());
    }
}
macro_rules! live_apply {
    ($name:ident,$session:ident,$conn:ty,$options:ty,$d:expr,$env:literal,$ns:literal,$role:literal,$create:literal,$pg:expr) => {
        #[test]
        fn $name() {
            use sqlx::{Connection,ConnectOptions,Row};
            assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),Ok("quent_dispatch_test"));
            let opts:$options=std::env::var($env).unwrap().parse().unwrap();
            let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            rt.block_on(async {
                let mut setup=<$conn>::connect_with(&opts.clone().disable_statement_logging()).await.unwrap();
                let mut observer=<$conn>::connect_with(&opts.clone().disable_statement_logging()).await.unwrap();
                let address=format!("{}:{}",opts.get_host(),opts.get_port());
                let db=context_at($d,1,&address,"quent_dispatch_test",Some("CANARY_subject"));
                let p=plan($d,$ns,&address,"tail_two");
                let (context,_)=registered(db.clone(),99);
                #[cfg(feature="output-tracing")] let capture=crate::output_capture::Capture::default();
                #[cfg(feature="output-tracing")] let guard=tracing::subscriber::set_default(capture.clone());
                // Each case starts with a new fixture table and dedicated product connection.
                for case in 0..38 {
                    sqlx::query("DROP TABLE IF EXISTS apply_probe").execute(&mut setup).await.unwrap();
                    sqlx::query($create).execute(&mut setup).await.unwrap();
                    sqlx::query("INSERT INTO apply_probe(a) VALUES(7)").execute(&mut setup).await.unwrap();
                    #[cfg(feature="output-tracing")] capture.0.lock().unwrap().clear();
                    eprintln!("C22/{}/fixture_case_{case}: setup_DDL=2 setup_DML=1",stringify!($name));
                    let mut session=$session::connect(opts.clone(),db.clone(),Duration::from_secs(5)).await.unwrap();
                    #[cfg(feature="output-tracing")] assert!(capture.0.lock().unwrap().is_empty());
                    let initial=current(context.clone(),session.identity(),request(&p,$ns,$role));
                    let source=Source {base:initial.retained(),trace:session.reader.trace.clone(),change_after:Cell::new(None),calls:Cell::new(0)};
                    let mut candidate=session.candidate(initial,Duration::from_secs(20)).unwrap();
                    if case==1 {source.change_after.set(Some("migration_provider_ack"));}
                    if (2..=8).contains(&case) {
                        let (cx,_)=registered(db.clone(),case-2);
                        candidate.current.context=cx;
                        // source is independently reconstructed from this registration below.
                    }
                    let mut source=source;
                    source.base=candidate.current.retained();
                    if case==17 {source.base.context=ApplyContext::without_adapter(db.clone());candidate.current=source.base.retained();}
                    if (18..23).contains(&case) {source.base.assertions[case-18].state=AttestedState::Unknown;candidate.current=source.base.retained();}
                    if case==23 {source.base.human.state=HumanState::Unknown;candidate.current=source.base.retained();}
                    let other_plan=plan($d,$ns,&address,"another_tail");
                    let other_target=plan($d,$ns,"different:1","tail_two");
                    if case==34 {source.base.context=source.base.context.with_database(context_at($d,1,&address,"quent_dispatch_test",None)).unwrap();candidate.current=source.base.retained();}
                    if case==35 {sqlx::query("CREATE INDEX apply_probe_idx ON apply_probe(a)").execute(&mut setup).await.unwrap();eprintln!("C22/fixture_index: additional_setup_DDL=1");}
                    if case==36 && !$pg {sqlx::query("ALTER TABLE apply_probe ENGINE=MyISAM").execute(&mut setup).await.unwrap();eprintln!("C22/fixture_engine: additional_setup_DDL=1");}
                    if case==9 { source.change_after.set(Some("ddl_ack")); }
                    if case==10 { source.change_after.set(Some("apply_begin_ack")); }
                    if case==11 {
                        sqlx::query("ALTER TABLE apply_probe ALTER COLUMN a SET DEFAULT 8").execute(&mut setup).await.unwrap();eprintln!("C22/fixture_default_drift: additional_setup_DDL=1");
                    }
                    #[cfg(feature="output-tracing")] capture.0.lock().unwrap().clear();
                    let prepared=session.prepare(candidate,&source,budget()).await;
                    #[cfg(feature="output-tracing")] assert!(capture.0.lock().unwrap().is_empty());
                    if (1..=8).contains(&case) || case==11 || (17..=23).contains(&case) || case==34 || case==35 || (case==36 && !$pg) {
                        assert!(prepared.is_err(),"case {case}");
                        assert!(!session.reader.trace.lock().unwrap().contains(&"ddl_attempt"));
                        record(&format!("{}/preDDL_refusal_{case}",stringify!($name)),&session.reader.trace,session.progress());
                        let count:i64=sqlx::query_scalar(if $pg {"SELECT count(*) FROM information_schema.columns WHERE table_schema='public' AND table_name='apply_probe'"}
                            else {"SELECT count(*) FROM information_schema.columns WHERE table_schema='quent_dispatch_test' AND table_name='apply_probe'"}).fetch_one(&mut observer).await.unwrap();assert_eq!(count,1);
                        let a:i64=sqlx::query_scalar("SELECT a FROM apply_probe").fetch_one(&mut observer).await.unwrap();assert_eq!(a,7);
                        eprintln!("C22/{}/preDDL_refusal_observer_{case}: columns1 data7 observer_SELECT=2",stringify!($name));
                        continue;
                    }
                    let mut handle=prepared.unwrap();
                    if case==37 {source.base.request_identity=ApplyRequest::new(source.base.request_id.clone()).unwrap().identity;}
                    if case==24 {source.base.context=source.base.context.with_database(context_at($d,1,&address,"quent_dispatch_test",Some("another_subject"))).unwrap();}
                    if case==25 {
                        let other=$session::connect(opts.clone(),db.clone(),Duration::from_secs(5)).await.unwrap();
                        source.base.session=other.identity();drop(other);
                        eprintln!("C22/other_apply_session_same_labels: additional_product_connection=1 DDL=0");
                    }
                    if case==26 {source.base.request=request(&other_plan,$ns,$role);}
                    if case==27 {source.base.request.phase=MigrationPhase::Contract;}
                    if case==28 {source.base.request.visibility.generation=nz(2);}
                    if case==29 {source.base.request.exclusion=DdlExclusion::ApplicationDeclared(nz(2));}
                    if case==30 {source.base.request_id="different_request".into();}
                    if case==31 {source.base.context=registered(db.clone(),99).0;}
                    if case==32 {source.base.human.expires=Instant::now();}
                    if case==33 {source.base.request=request(&other_target,$ns,$role);}
                    if case==12 {
                        // Deliberate fixture violation of the asserted exclusion:
                        // step one may succeed, step two gets a real server error.
                        sqlx::query("ALTER TABLE apply_probe ADD COLUMN tail_two INTEGER NULL").execute(&mut setup).await.unwrap();eprintln!("C22/fixture_second_step_conflict: additional_setup_DDL=1");
                    }
                    if case==13 { handle.candidate.deadline=Instant::now(); }
                    if case==14 {
                        let calls=session.reader.trace.lock().unwrap().len();
                        drop(session.apply(handle,&source));
                        assert_eq!(session.reader.trace.lock().unwrap().len(),calls);
                        continue;
                    }
                    if case==15 || case==16 {
                        // Real server lock blocks product ALTER. Client timeout/drop
                        // is explicitly not proof that the server did not apply it.
                        if $pg {
                            sqlx::query("BEGIN").execute(&mut observer).await.unwrap();
                            sqlx::query("LOCK TABLE apply_probe IN ACCESS EXCLUSIVE MODE").execute(&mut observer).await.unwrap();
                        } else {sqlx::raw_sql("LOCK TABLES apply_probe WRITE").execute(&mut observer).await.unwrap();}
                        handle.candidate.deadline=Instant::now()+Duration::from_millis(150);
                        if case==15 {
                            let mut f=Box::pin(session.apply(handle,&source));
                            let trace=source.trace.clone();
                            tokio::time::timeout(Duration::from_secs(2),std::future::poll_fn(|cx| {
                                assert!(f.as_mut().poll(cx).is_pending());
                                if trace.lock().unwrap().contains(&"ddl_attempt") {std::task::Poll::Ready(())}
                                else {cx.waker().wake_by_ref();std::task::Poll::Pending}
                            })).await.unwrap();drop(f);
                            assert_eq!(session.progress().completion,Completion::StepUnknown);
                            assert_eq!(session.progress().attempted,1);
                        } else {
                            #[cfg(feature="output-tracing")] capture.0.lock().unwrap().clear();
                    let out=session.apply(handle,&source).await;
                    #[cfg(feature="output-tracing")] assert!(capture.0.lock().unwrap().is_empty());
                            assert_eq!(out.attempted,1);assert_eq!(out.acknowledged,0);
                            assert!(matches!(out.completion,Completion::StepUnknown|Completion::RollbackUnknown));
                        }
                        record(&format!("{}/blocked_cancel_timeout_{case}",stringify!($name)),&session.reader.trace,session.progress());
                        drop(session);
                        sqlx::raw_sql(if $pg {"ROLLBACK"}else{"UNLOCK TABLES"}).execute(&mut observer).await.unwrap();
                        // Allow server-side connection termination before next fixture reset.
                        tokio::time::sleep(Duration::from_millis(50)).await;
                        let observed=sqlx::query(if $pg {"SELECT column_name,data_type,is_nullable,column_default,ordinal_position::text FROM information_schema.columns WHERE table_schema='public' AND table_name='apply_probe' ORDER BY ordinal_position"}
                            else {"SELECT column_name,data_type,is_nullable,column_default,CAST(ordinal_position AS CHAR) FROM information_schema.columns WHERE table_schema='quent_dispatch_test' AND table_name='apply_probe' ORDER BY ordinal_position"}).fetch_all(&mut observer).await.unwrap();
                        assert!((1..=2).contains(&observed.len()));
                        for (i,row) in observed.iter().enumerate() {
                            assert_eq!(row.get::<String,_>(0),if i==0 {"a"}else{"tail_one"});
                            assert_eq!(row.get::<String,_>(1),if i==0 {"bigint"}else if $pg {"integer"}else{"int"});
                            assert_eq!(row.get::<String,_>(2),if i==0 {"NO"}else{"YES"});
                            assert_eq!(row.get::<Option<String>,_>(3),if i==0 {Some("7".into())}else{None});
                            assert_eq!(row.get::<String,_>(4),(i+1).to_string());
                        }
                        let a:i64=sqlx::query_scalar("SELECT a FROM apply_probe").fetch_one(&mut observer).await.unwrap();assert_eq!(a,7);
                        eprintln!("C22/{}/cancel_timeout_observer_case_{case}: snapshot_columns={} native/order/null/default/data7 observer_SELECT=2 fixture_lock_controls={} snapshot_is_not_client_ACK",stringify!($name),observed.len(),if $pg {3}else{2});
                        continue;
                    }
                    #[cfg(feature="output-tracing")] capture.0.lock().unwrap().clear();
                    let out=session.apply(handle,&source).await;
                    #[cfg(feature="output-tracing")] assert!(capture.0.lock().unwrap().is_empty());
                    record(&format!("{}/apply_{case}",stringify!($name)),&session.reader.trace,out);
                    assert!(session.candidate(source.base.retained(),Duration::from_secs(1)).is_err() || case==13 || (24..=33).contains(&case) || case==37);
                    if (24..=33).contains(&case) || case==37 {
                        assert_eq!(out.attempted,0);assert_eq!(out.completion,Completion::NotSent);
                        assert_eq!(out.refusal,Some(ApplyError::Mismatch));
                        let count:i64=sqlx::query_scalar(if $pg {"SELECT count(*) FROM information_schema.columns WHERE table_schema='public' AND table_name='apply_probe'"}
                            else {"SELECT count(*) FROM information_schema.columns WHERE table_schema='quent_dispatch_test' AND table_name='apply_probe'"}).fetch_one(&mut observer).await.unwrap();assert_eq!(count,1);
                        let a:i64=sqlx::query_scalar("SELECT a FROM apply_probe").fetch_one(&mut observer).await.unwrap();assert_eq!(a,7);
                        eprintln!("C22/{}/post_seal_refusal_observer_{case}: columns1 data7 observer_SELECT=2",stringify!($name));
                        continue;
                    }
                    if case==13 {assert_eq!(out.attempted,0);assert_eq!(out.refusal,Some(ApplyError::Expired));continue;}
                    if case==10 && $pg {assert_eq!(out.acknowledged,0);assert_eq!(out.completion,Completion::RolledBack);}
                    else if case==9 {assert_eq!(out.acknowledged,1);assert_eq!(out.unsent,1);assert_eq!(out.completion,if $pg {Completion::RolledBack}else{Completion::ImplicitCommitPrefix});}
                    else if case==12 {assert_eq!(out.acknowledged,1);assert_eq!(out.rejected_step,Some(1));assert_eq!(out.unacknowledged_step,None);assert_eq!(out.completion,if $pg {Completion::RolledBack}else{Completion::StepRejected});}
                    else {assert_eq!(out.acknowledged,2);assert_eq!(out.completion,if $pg {Completion::Committed}else{Completion::ImplicitCommitPrefix});assert!(out.refusal.is_none());}
                    let observer_sql=if $pg {"SELECT column_name,data_type,is_nullable,column_default,ordinal_position::text FROM information_schema.columns WHERE table_schema='public' AND table_name='apply_probe' ORDER BY ordinal_position"}
                        else {"SELECT column_name,data_type,is_nullable,column_default,CAST(ordinal_position AS CHAR) FROM information_schema.columns WHERE table_schema='quent_dispatch_test' AND table_name='apply_probe' ORDER BY ordinal_position"};
                    let columns=sqlx::query(sqlx::AssertSqlSafe(observer_sql.to_owned())).fetch_all(&mut observer).await.unwrap();
                    let names:Vec<String>=columns.iter().map(|r|r.get(0)).collect();
                    let expected:Vec<&str>=if case==12 {if $pg {vec!["a","tail_two"]}else{vec!["a","tail_two","tail_one"]}}
                        else if (case==9 || case==10) && $pg {vec!["a"]}
                        else if case==9 {vec!["a","tail_one"]}else{vec!["a","tail_one","tail_two"]};
                    assert_eq!(names,expected);
                    for (index,r) in columns.iter().enumerate() {
                        assert_eq!(r.get::<String,_>(4),(index+1).to_string());
                        assert_eq!(r.get::<String,_>(2),if index==0 {"NO"}else{"YES"});
                        if index==0 { assert_eq!(r.get::<String,_>(1),"bigint");assert_eq!(r.get::<Option<String>,_>(3),Some("7".into())); }
                        if index>0 { assert_eq!(r.get::<Option<String>,_>(3),None);assert_eq!(r.get::<String,_>(1),if $pg {"integer"}else{"int"}); }
                    }
                    let a:i64=sqlx::query_scalar("SELECT a FROM apply_probe").fetch_one(&mut observer).await.unwrap();assert_eq!(a,7);
                    let mut observer_selects=2;
                    for tail in ["tail_one","tail_two"] {
                        if names.iter().any(|n|n==tail) {
                            let query=format!("SELECT COUNT(*) FROM apply_probe WHERE {tail} IS NOT NULL");
                            let nonnull:i64=sqlx::query_scalar(sqlx::AssertSqlSafe(query)).fetch_one(&mut observer).await.unwrap();
                            assert_eq!(nonnull,0);observer_selects+=1;
                        }
                    }
                    eprintln!("C22/{}/observer_case_{case}: columns={} native/order/null/default/data checked; observer_SELECT={observer_selects} fixture_DDL_DML separate",stringify!($name),columns.len());
                }
                #[cfg(feature="output-tracing")] {drop(guard);}
            });
        }
    };
}
#[cfg(feature = "postgres")]
live_apply!(
    live_apply_postgres,
    PostgresApply,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    "public",
    "quent_test",
    "CREATE TABLE apply_probe(a bigint NOT NULL DEFAULT 7)",
    true
);
#[cfg(feature = "mysql")]
live_apply!(
    live_apply_mysql,
    MysqlApply,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "quent_dispatch_test",
    "quent_test@%",
    "CREATE TABLE apply_probe(a bigint NOT NULL DEFAULT 7) ENGINE=InnoDB",
    false
);

#[path = "network.rs"]
mod network;
macro_rules! live_response_loss {
    ($name:ident,$session:ident,$conn:ty,$options:ty,$ssl:path,$d:expr,$env:literal,$ns:literal,$role:literal,$create:literal,$pg:expr) => {
        #[test]
        fn $name() {
            use sqlx::{Connection,ConnectOptions,Row};
            assert_eq!(std::env::var("QUENT_TEST_DISPOSABLE").as_deref(),Ok("quent_dispatch_test"));
            let opts:$options=std::env::var($env).unwrap().parse().unwrap();
            let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            rt.block_on(async {
                let mut observer=<$conn>::connect_with(&opts.clone().disable_statement_logging()).await.unwrap();
                for last in [false,true] {
                    sqlx::query("DROP TABLE IF EXISTS apply_probe").execute(&mut observer).await.unwrap();
                    sqlx::query($create).execute(&mut observer).await.unwrap();
                    sqlx::query("INSERT INTO apply_probe(a) VALUES(7)").execute(&mut observer).await.unwrap();
                    let proxy=network::Proxy::new(format!("{}:{}",opts.get_host(),opts.get_port()).parse().unwrap(),$pg,
                        if $pg&&last {None}else{Some(usize::from(last))});
                    let options=opts.clone().host("127.0.0.1").port(proxy.address.port()).ssl_mode($ssl);
                    let address=proxy.address.to_string();let db=context_at($d,1,&address,"quent_dispatch_test",Some("CANARY_subject"));
                    let p=plan($d,$ns,&address,"tail_two");let (cx,_)=registered(db.clone(),99);
                    let mut session=$session::connect(options,db,Duration::from_secs(3)).await.unwrap();
                    let c=current(cx,session.identity(),request(&p,$ns,$role));
                    let source=Source {base:c.retained(),trace:session.reader.trace.clone(),change_after:Cell::new(None),calls:Cell::new(0)};
                    let candidate=session.candidate(c,Duration::from_secs(2)).unwrap();
                    let handle=session.prepare(candidate,&source,budget()).await.unwrap();
                    let out=session.apply(handle,&source).await;
                    assert!(proxy.server_ack.load(Ordering::SeqCst),"actual server command ACK withheld");
                    if $pg&&last {assert_eq!(out.completion,Completion::CommitUnknown);assert_eq!(out.acknowledged,2);}
                    else {assert!(matches!(out.completion,Completion::StepUnknown|Completion::RollbackUnknown));assert_eq!(out.unacknowledged_step,Some(usize::from(last)));assert_eq!(out.acknowledged,usize::from(last));}
                    record(&format!("{}/actual_server_ACK_withheld_{last}",stringify!($name)),&session.reader.trace,out);
                    // Catalog observer is a separate physical connection. PostgreSQL
                    // uncommitted changes remain invisible; lost COMMIT ACK may be durable.
                    let count:i64=sqlx::query_scalar(if $pg {"SELECT count(*) FROM information_schema.columns WHERE table_schema='public' AND table_name='apply_probe'"}
                        else {"SELECT count(*) FROM information_schema.columns WHERE table_schema='quent_dispatch_test' AND table_name='apply_probe'"}).fetch_one(&mut observer).await.unwrap();
                    assert_eq!(count,if $pg&&!last {1}else if last {3}else{2});
                    let observer_sql=if $pg {"SELECT column_name,data_type,is_nullable,column_default,ordinal_position::text FROM information_schema.columns WHERE table_schema='public' AND table_name='apply_probe' ORDER BY ordinal_position"}
                        else {"SELECT column_name,data_type,is_nullable,column_default,CAST(ordinal_position AS CHAR) FROM information_schema.columns WHERE table_schema='quent_dispatch_test' AND table_name='apply_probe' ORDER BY ordinal_position"};
                    let columns=sqlx::query(sqlx::AssertSqlSafe(observer_sql.to_owned())).fetch_all(&mut observer).await.unwrap();
                    let expected=["a","tail_one","tail_two"];
                    for (i,column) in columns.iter().enumerate() {
                        assert_eq!(column.get::<String,_>(0),expected[i]);
                        assert_eq!(column.get::<String,_>(1),if i==0 {"bigint"}else if $pg {"integer"}else{"int"});
                        assert_eq!(column.get::<String,_>(2),if i==0 {"NO"}else{"YES"});
                        assert_eq!(column.get::<Option<String>,_>(3),if i==0 {Some("7".into())}else{None});
                        assert_eq!(column.get::<String,_>(4),(i+1).to_string());
                    }
                    let executes=proxy.ddl_executes.load(Ordering::SeqCst);
                    drop(session);drop(proxy);
                    let value:i64=sqlx::query_scalar("SELECT a FROM apply_probe").fetch_one(&mut observer).await.unwrap();assert_eq!(value,7);
                    eprintln!("C22/{}/real_protocol_loss last={last} actual_server_ACK=true forwarded_client_ACK=false ddl_executes={executes} observer_columns={count} observer_data7 native/order/null/default_checked observer_SELECT=3 setup_DDL=2 setup_DML=1",stringify!($name));
                }
            });
        }
    };
}
#[cfg(feature = "postgres")]
live_response_loss!(
    live_apply_response_loss_postgres,
    PostgresApply,
    sqlx::PgConnection,
    sqlx::postgres::PgConnectOptions,
    sqlx::postgres::PgSslMode::Disable,
    Dialect::Postgres,
    "QUENT_TEST_POSTGRES_DSN",
    "public",
    "quent_test",
    "CREATE TABLE apply_probe(a bigint NOT NULL DEFAULT 7)",
    true
);
#[cfg(feature = "mysql")]
live_response_loss!(
    live_apply_response_loss_mysql,
    MysqlApply,
    sqlx::MySqlConnection,
    sqlx::mysql::MySqlConnectOptions,
    sqlx::mysql::MySqlSslMode::Disabled,
    Dialect::Mysql,
    "QUENT_TEST_MYSQL_DSN",
    "quent_dispatch_test",
    "quent_test@%",
    "CREATE TABLE apply_probe(a bigint NOT NULL DEFAULT 7) ENGINE=InnoDB",
    false
);

#[test]
fn c12_current_guard_checks_each_driver_resume() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let valid = Cell::new(true);
        let polls = Cell::new(0);
        let driver = std::future::poll_fn(|cx| {
            polls.set(polls.get() + 1);
            cx.waker().wake_by_ref();
            std::task::Poll::<()>::Pending
        });
        let mut future = Box::pin(guarded(driver, || {
            if valid.get() {
                Ok(())
            } else {
                Err(ApplyError::Mismatch)
            }
        }));
        std::future::poll_fn(|cx| {
            assert!(future.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        assert_eq!(polls.get(), 1);
        valid.set(false);
        assert_eq!(future.await, Err(ApplyError::Mismatch));
        assert_eq!(polls.get(), 1, "no driver resume after current loss");
        let polls = Cell::new(0);
        let driver = std::future::poll_fn(|cx| {
            polls.set(polls.get() + 1);
            if polls.get() == 1 {
                cx.waker().wake_by_ref();
                std::task::Poll::Pending
            } else {
                std::task::Poll::Ready(42)
            }
        });
        assert_eq!(guarded(driver, || Ok(())).await, Ok(42));
        assert_eq!(polls.get(), 2, "detecting positive resume control");
    });
}
