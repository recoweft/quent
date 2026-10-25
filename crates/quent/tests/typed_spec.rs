#![cfg(feature = "operation-spec")]
use quent::operation_spec::*;
use quent_core::*;
use serde_json::{Value as Json, json};
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn table() -> Table {
    Table::new(vec![id("catalog"), id("items")], None).unwrap()
}
fn column(name: &str, ty: SqlType, nullable: bool) -> Column {
    table().column(id(name), ty, nullable)
}
fn columns(ty: SqlType, nullable: bool) -> Vec<Column> {
    vec![
        column("id", SqlType::I64, false),
        column("tenant", SqlType::I64, false),
        column("deleted", SqlType::Bool, false),
        column("payload", ty, nullable),
    ]
}
fn context(
    d: Dialect,
    ty: SqlType,
    nullable: bool,
    subject: &str,
    extras: Vec<PolicyDeclaration>,
) -> DatabaseContext {
    let b = DatabaseBinding::application_supplied(
        DatabaseIdentity::new("db").unwrap(),
        TargetAddress::new("route").unwrap(),
        id("catalog"),
        d,
    );
    let g = Generation::new(1).unwrap();
    let mut ps = vec![
        PolicyDeclaration::Tenant(column("tenant", SqlType::I64, false)),
        PolicyDeclaration::SoftDelete(column("deleted", SqlType::Bool, false)),
    ];
    ps.extend(extras);
    let facts = ScopeFacts::application_asserted(
        table(),
        d,
        columns(ty, nullable),
        vec![Key::Primary(vec![
            column("tenant", SqlType::I64, false),
            column("id", SqlType::I64, false),
        ])],
        TableContext::OrdinaryBaseTableWithoutEffects,
    )
    .unwrap();
    DatabaseContext::new(
        Settings::new(
            b.clone(),
            SettingsIdentity::new("settings").unwrap(),
            g,
            PolicySet::new(b.clone(), PolicyIdentity::new("policy").unwrap(), g, ps).unwrap(),
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
            Some(SubjectIdentity::new(subject).unwrap()),
            Some(
                TenantSnapshot::application_supplied(
                    b,
                    column("tenant", SqlType::I64, false),
                    Value::I64(11),
                )
                .unwrap(),
            ),
        )
        .unwrap(),
    )
    .unwrap()
}
fn registered(ty: SqlType, nullable: bool) -> Vec<TypedColumn> {
    columns(ty, nullable)
        .into_iter()
        .map(|c| {
            TypedColumn::application_supplied(
                c.sensitive_name().sensitive_text(),
                c.sensitive_name().clone(),
                ColumnType::Native(c.sql_type()),
                c.nullable(),
                if c.sensitive_name().sensitive_text() == "tenant" {
                    ColumnRole::Tenant
                } else {
                    ColumnRole::Ordinary
                },
                true,
            )
            .unwrap()
        })
        .collect()
}
fn mapping(c: &DatabaseContext, ty: SqlType, nullable: bool) -> TypedModelContract {
    TypedModelContract::application_asserted(
        "items",
        table(),
        registered(ty, nullable),
        ModelCoverage::Complete,
        c,
    )
    .unwrap()
}
fn source(filter: Option<Json>) -> SpecSource {
    let mut j = json!({"kind":"quent_operation_spec","version":1,"model":"items","fields":["id","payload"],"order":[{"field":"id","direction":"desc"}],"limit":10});
    if let Some(f) = filter {
        j["filter"] = f;
    }
    SpecSource::parse(&serde_json::to_vec(&j).unwrap()).unwrap()
}
fn cmp(tag: &str, v: Json) -> Json {
    json!({"op":"eq","field":"payload","value":{"type":tag,"value":v}})
}
#[test]
fn c01_c06_same_record_independent_sql_args_metadata_deterministic() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, SqlType::Text, true, "user", vec![]);
        let m = mapping(&c, SqlType::Text, true);
        for _ in 0..3 {
            let r = source(Some(
                json!({"op":"eq","field":"id","value":{"type":"i64","value":7}}),
            ))
            .compile(&m, &c)
            .unwrap();
            let i = r.inspect();
            let s = i.sensitive_source();
            let sql = if d == Dialect::Postgres {
                "SELECT \"items\".\"id\", \"items\".\"payload\" FROM \"catalog\".\"items\" WHERE ((\"items\".\"id\" = $1) AND (\"items\".\"tenant\" = $2) AND (\"items\".\"deleted\" = $3)) ORDER BY \"items\".\"id\" DESC LIMIT $4"
            } else {
                "SELECT `items`.`id`, `items`.`payload` FROM `catalog`.`items` WHERE ((`items`.`id` = ?) AND (`items`.`tenant` = ?) AND (`items`.`deleted` = ?)) ORDER BY `items`.`id` DESC LIMIT ?"
            };
            assert_eq!(s.sql(), sql);
            assert_eq!(
                s.args(),
                &[
                    Value::I64(7),
                    Value::I64(11),
                    Value::Bool(false),
                    Value::I64(10)
                ]
            );
            assert_eq!(s.metadata().family(), Family::Select);
            assert_eq!(s.metadata().limit(), Some(10));
            assert!(!s.caller_missing_where());
            assert_eq!(i.scope(), Scope::Single);
            assert_eq!(r.public_view(), i.public_view());
            assert!(r.matches_current(&c, &m));
        }
    }
}
#[test]
fn c07_width_sign_extrema_checked_without_casts() {
    let cases = [
        (
            SqlType::I8,
            "i8",
            json!(i8::MIN),
            json!(i8::MAX),
            Value::I8(i8::MIN),
            Value::I8(i8::MAX),
        ),
        (
            SqlType::I16,
            "i16",
            json!(i16::MIN),
            json!(i16::MAX),
            Value::I16(i16::MIN),
            Value::I16(i16::MAX),
        ),
        (
            SqlType::I32,
            "i32",
            json!(i32::MIN),
            json!(i32::MAX),
            Value::I32(i32::MIN),
            Value::I32(i32::MAX),
        ),
        (
            SqlType::I64,
            "i64",
            json!(i64::MIN),
            json!(i64::MAX),
            Value::I64(i64::MIN),
            Value::I64(i64::MAX),
        ),
        (
            SqlType::U8,
            "u8",
            json!(0),
            json!(u8::MAX),
            Value::U8(0),
            Value::U8(u8::MAX),
        ),
        (
            SqlType::U16,
            "u16",
            json!(0),
            json!(u16::MAX),
            Value::U16(0),
            Value::U16(u16::MAX),
        ),
        (
            SqlType::U32,
            "u32",
            json!(0),
            json!(u32::MAX),
            Value::U32(0),
            Value::U32(u32::MAX),
        ),
        (
            SqlType::U64,
            "u64",
            json!(0),
            json!(u64::MAX),
            Value::U64(0),
            Value::U64(u64::MAX),
        ),
        (
            SqlType::Bool,
            "bool",
            json!(false),
            json!(true),
            Value::Bool(false),
            Value::Bool(true),
        ),
        (
            SqlType::Text,
            "text",
            json!(""),
            json!("雪"),
            Value::Text("".into()),
            Value::Text("雪".into()),
        ),
    ];
    for d in [Dialect::Postgres, Dialect::Mysql] {
        for (ty, tag, min, max, lo, hi) in &cases {
            let c = context(d, *ty, true, "user", vec![]);
            let m = mapping(&c, *ty, true);
            for (v, expected) in [(min, lo), (max, hi)] {
                let result = source(Some(cmp(tag, v.clone()))).compile(&m, &c);
                if d == Dialect::Postgres
                    && matches!(
                        ty,
                        SqlType::I8 | SqlType::U8 | SqlType::U16 | SqlType::U32 | SqlType::U64
                    )
                {
                    assert_eq!(result.unwrap_err(), TypedError::Dialect);
                } else {
                    let r = result.unwrap();
                    assert_eq!(&r.inspect().sensitive_source().args()[0], expected);
                }
            }
        }
    }
}
#[test]
fn c07_null_not_in_operator_and_tag_matrix() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        for nullable in [false, true] {
            let c = context(d, SqlType::I32, nullable, "user", vec![]);
            let m = mapping(&c, SqlType::I32, nullable);
            for p in [
                cmp("i32", Json::Null),
                json!({"op":"not","arg":cmp("i32",Json::Null)}),
            ] {
                assert_eq!(
                    source(Some(p)).compile(&m, &c).unwrap_err(),
                    if nullable {
                        TypedError::NullComparison
                    } else {
                        TypedError::NonNullableNull
                    }
                );
            }
            for p in [
                json!({"op":"is_null","field":"payload"}),
                json!({"op":"not","arg":{"op":"is_null","field":"payload"}}),
            ] {
                assert!(source(Some(p)).compile(&m, &c).is_ok());
            }
            let p = json!({"op":"in","field":"payload","values":[{"type":"i32","value":3},{"type":"i32","value":null}]});
            let r = source(Some(p)).compile(&m, &c);
            if nullable {
                assert_eq!(
                    &r.unwrap().inspect().sensitive_source().args()[..2],
                    &[Value::I32(3), Value::Null(SqlType::I32)]
                );
            } else {
                assert_eq!(r.unwrap_err(), TypedError::NonNullableNull);
            }
            for p in [
                cmp("i64", json!(3)),
                json!({"op":"in","field":"payload","values":[{"type":"i32","value":3},{"type":"u32","value":3}]}),
                json!({"op":"not","arg":cmp("text",json!("3"))}),
            ] {
                assert_eq!(
                    source(Some(p)).compile(&m, &c).unwrap_err(),
                    TypedError::TypeMismatch
                );
            }
        }
        for (ty, tag, v) in [
            (SqlType::Text, "text", json!("value")),
            (SqlType::Bool, "bool", json!(true)),
        ] {
            let c = context(d, ty, true, "user", vec![]);
            let m = mapping(&c, ty, true);
            for op in ["lt", "le", "gt", "ge"] {
                let mut p = cmp(tag, v.clone());
                p["op"] = json!(op);
                assert_eq!(
                    source(Some(p)).compile(&m, &c).unwrap_err(),
                    TypedError::Operator
                );
            }
        }
    }
}
#[test]
fn c08_c11_c12_full_mapping_context_and_all_branch_tenant_refusals() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, SqlType::I32, true, "user", vec![]);
        let m = mapping(&c, SqlType::I32, true);
        for v in [11, 12] {
            let p = json!({"op":"eq","field":"tenant","value":{"type":"i64","value":v}});
            for f in [
                p.clone(),
                json!({"op":"not","arg":p}),
                json!({"op":"or","args":[cmp("i32",json!(1)),p]}),
                json!({"op":"in","field":"tenant","values":[{"type":"i64","value":v}]}),
            ] {
                assert_eq!(
                    source(Some(f)).compile(&m, &c).unwrap_err(),
                    TypedError::TenantPredicate
                );
            }
        }
        let r = source(None).compile(&m, &c).unwrap();
        let other = context(d, SqlType::I32, true, "other", vec![]);
        assert!(!r.matches_current(&other, &m));
        assert_eq!(
            source(None).compile(&m, &other).unwrap_err(),
            TypedError::SnapshotMismatch
        );
        let mut cs = registered(SqlType::I32, true);
        cs[3] = TypedColumn::application_supplied(
            "payload",
            id("payload"),
            ColumnType::Native(SqlType::I64),
            true,
            ColumnRole::Ordinary,
            true,
        )
        .unwrap();
        let changed = TypedModelContract::application_asserted(
            "items",
            table(),
            cs,
            ModelCoverage::Complete,
            &c,
        )
        .unwrap();
        assert!(!r.matches_current(&c, &changed));
        assert_eq!(
            source(None).compile(&changed, &c).unwrap_err(),
            TypedError::Mapping
        );
        let unknown = TypedModelContract::application_asserted(
            "items",
            table(),
            registered(SqlType::I32, true),
            ModelCoverage::Unknown,
            &c,
        )
        .unwrap();
        assert_eq!(
            source(None).compile(&unknown, &c).unwrap_err(),
            TypedError::UnknownCoverage
        );
        let mut cs = registered(SqlType::I32, true);
        cs.push(
            TypedColumn::application_supplied(
                "alias",
                id("tenant"),
                ColumnType::Native(SqlType::I64),
                false,
                ColumnRole::Ordinary,
                true,
            )
            .unwrap(),
        );
        assert_eq!(
            TypedModelContract::application_asserted(
                "items",
                table(),
                cs,
                ModelCoverage::Complete,
                &c
            )
            .unwrap_err(),
            TypedError::Mapping
        );
    }
}
#[test]
fn c15_diagnostics_unobserved_unsupported_and_policy_are_source_free() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, SqlType::Text, true, "CANARY_subject", vec![]);
        for ty in [
            ColumnType::Unobserved,
            ColumnType::Unsupported(NativeFamily::Decimal),
            ColumnType::Unsupported(NativeFamily::Uuid),
            ColumnType::Unsupported(NativeFamily::Date),
            ColumnType::Unsupported(NativeFamily::Time),
            ColumnType::Unsupported(NativeFamily::Bytes),
            ColumnType::Unsupported(NativeFamily::Float),
            ColumnType::Unsupported(NativeFamily::Other),
        ] {
            let mut cs = registered(SqlType::Text, true);
            cs[3] = TypedColumn::application_supplied(
                "payload",
                id("payload"),
                ty,
                true,
                ColumnRole::Ordinary,
                true,
            )
            .unwrap();
            let m = TypedModelContract::application_asserted(
                "items",
                table(),
                cs,
                ModelCoverage::Complete,
                &c,
            )
            .unwrap();
            let e = source(Some(cmp("text", json!("CANARY_value_/path"))))
                .compile(&m, &c)
                .unwrap_err();
            let outputs = format!(
                "{m:?} {e:?} {e} {} {:?}",
                e.public_view().to_json(),
                e.public_view()
            );
            assert!(!outputs.contains("CANARY"));
            assert!(std::error::Error::source(&e).is_none());
            assert_eq!(
                e,
                if ty == ColumnType::Unobserved {
                    TypedError::Unobserved
                } else {
                    TypedError::UnsupportedFamily
                }
            );
        }
        for policy in [
            PolicyDeclaration::Pii(column("payload", SqlType::Text, true)),
            PolicyDeclaration::RequiredFilter(column("payload", SqlType::Text, true)),
        ] {
            let c = context(d, SqlType::Text, true, "user", vec![policy]);
            let m = mapping(&c, SqlType::Text, true);
            assert_eq!(
                source(None).compile(&m, &c).unwrap_err(),
                TypedError::PolicyRefused
            );
        }
    }
}

#[test]
fn c24_registration_refuses_incomplete_alias_wrong_target_roles_and_read() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, SqlType::I32, true, "user", vec![]);
        for (role, read, expected) in [
            (ColumnRole::Unknown, true, TypedError::Role),
            (ColumnRole::Ordinary, false, TypedError::ReadDenied),
        ] {
            let mut cs = registered(SqlType::I32, true);
            cs[3] = TypedColumn::application_supplied(
                "payload",
                id("payload"),
                ColumnType::Native(SqlType::I32),
                true,
                role,
                read,
            )
            .unwrap();
            let m = TypedModelContract::application_asserted(
                "items",
                table(),
                cs,
                ModelCoverage::Complete,
                &c,
            )
            .unwrap();
            assert_eq!(source(None).compile(&m, &c).unwrap_err(), expected);
        }
        let mut cs = registered(SqlType::I32, true);
        cs.pop();
        let m = TypedModelContract::application_asserted(
            "items",
            table(),
            cs,
            ModelCoverage::Complete,
            &c,
        )
        .unwrap();
        assert_eq!(
            source(None).compile(&m, &c).unwrap_err(),
            TypedError::Mapping
        );
        let other = Table::new(vec![id("other"), id("items")], None).unwrap();
        let m = TypedModelContract::application_asserted(
            "items",
            other,
            registered(SqlType::I32, true),
            ModelCoverage::Complete,
            &c,
        )
        .unwrap();
        assert_eq!(
            source(None).compile(&m, &c).unwrap_err(),
            TypedError::Mapping
        );
        let alias = Table::new(vec![id("catalog"), id("items")], Some(id("alias"))).unwrap();
        assert_eq!(
            TypedModelContract::application_asserted(
                "items",
                alias,
                registered(SqlType::I32, true),
                ModelCoverage::Complete,
                &c
            )
            .unwrap_err(),
            TypedError::Mapping
        );
        let m = TypedModelContract::application_asserted(
            "other",
            table(),
            registered(SqlType::I32, true),
            ModelCoverage::Complete,
            &c,
        )
        .unwrap();
        assert_eq!(source(None).compile(&m, &c).unwrap_err(), TypedError::Role);
    }
}
#[test]
fn c15_fixed_diagnostic_register_and_successful_carrier_canaries() {
    for (code, action) in [
        (TypedError::Mapping, "check_application_registration"),
        (
            TypedError::UnknownCoverage,
            "check_application_registration",
        ),
        (TypedError::Unobserved, "check_application_registration"),
        (TypedError::Role, "check_application_registration"),
        (TypedError::UnsupportedFamily, "check_native_support"),
        (TypedError::Dialect, "check_native_support"),
        (TypedError::Operator, "check_native_support"),
        (TypedError::NativeRefused, "check_native_support"),
        (TypedError::TypeMismatch, "check_literal_type"),
        (TypedError::NonNullableNull, "check_literal_type"),
        (TypedError::NullComparison, "use_explicit_null_predicate"),
        (TypedError::ReadDenied, "check_read_policy"),
        (TypedError::TenantPredicate, "check_read_policy"),
        (TypedError::PolicyRefused, "check_read_policy"),
        (TypedError::SnapshotMismatch, "recompile_current_snapshot"),
    ] {
        let out: Json = serde_json::from_str(&code.public_view().to_json()).unwrap();
        assert_eq!(out["kind"], "quent_typed_spec_diagnostic");
        assert_eq!(out["version"], 1);
        assert_eq!(out["state"], "refused");
        assert_eq!(out["action"], action);
        assert_eq!(out.as_object().unwrap().len(), 5);
        assert!(std::error::Error::source(&code).is_none());
    }
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, SqlType::Text, true, "CANARY_subject", vec![]);
        let m = mapping(&c, SqlType::Text, true);
        let s = source(Some(cmp(
            "text",
            json!("CANARY_SQL_value_tenant_default_/path"),
        )));
        assert!(!format!("{s:?} {m:?}").contains("CANARY"));
        let r = s.compile(&m, &c).unwrap();
        assert!(!format!("{r:?} {}", r.public_view().to_json()).contains("CANARY"));
    }
}
#[test]
fn c07_every_integer_operator_not_and_in_preserve_native_arguments() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        for (ty, tag, v, expected) in [
            (SqlType::I8, "i8", json!(-8), Value::I8(-8)),
            (SqlType::I16, "i16", json!(-16), Value::I16(-16)),
            (SqlType::I32, "i32", json!(-32), Value::I32(-32)),
            (SqlType::I64, "i64", json!(-64), Value::I64(-64)),
            (SqlType::U8, "u8", json!(8), Value::U8(8)),
            (SqlType::U16, "u16", json!(16), Value::U16(16)),
            (SqlType::U32, "u32", json!(32), Value::U32(32)),
            (SqlType::U64, "u64", json!(64), Value::U64(64)),
        ] {
            let c = context(d, ty, true, "user", vec![]);
            let m = mapping(&c, ty, true);
            for op in ["eq", "ne", "lt", "le", "gt", "ge", "in"] {
                let p = if op == "in" {
                    json!({"op":op,"field":"payload","values":[{"type":tag,"value":v},{"type":tag,"value":null}]})
                } else {
                    json!({"op":op,"field":"payload","value":{"type":tag,"value":v}})
                };
                for f in [p.clone(), json!({"op":"not","arg":p})] {
                    let result = source(Some(f)).compile(&m, &c);
                    if d == Dialect::Postgres
                        && matches!(
                            ty,
                            SqlType::I8 | SqlType::U8 | SqlType::U16 | SqlType::U32 | SqlType::U64
                        )
                    {
                        assert_eq!(result.unwrap_err(), TypedError::Dialect);
                    } else {
                        let r = result.unwrap();
                        assert_eq!(r.inspect().sensitive_source().args()[0], expected);
                        if op == "in" {
                            assert_eq!(r.inspect().sensitive_source().args()[1], Value::Null(ty));
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "review")]
#[test]
fn c12_review_uses_exact_typed_compiler_and_independent_current() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let c = context(d, SqlType::Text, true, "user", vec![]);
        let m = mapping(&c, SqlType::Text, true);
        let make = || {
            source(Some(
                json!({"op":"eq","field":"id","value":{"type":"i64","value":7}}),
            ))
        };
        let expected = make().compile(&m, &c).unwrap();
        let r = quent::review::typed_spec(make(), &m, &c).unwrap();
        let j: Json = serde_json::from_str(&r.to_json()).unwrap();
        assert_eq!(
            j["inspection"],
            serde_json::from_str::<Json>(&expected.public_view().to_json()).unwrap()
        );
        assert_eq!(j["precision"]["binding"], "exact_application_snapshot");
        assert_eq!(j["precision"]["runtime"], "unverified");
        assert_eq!(j["coverage"], "partial"); // bool guard is still partial, never hidden.
        assert_eq!(r.exit_code(), 5);
        let other = context(d, SqlType::Text, true, "changed", vec![]);
        assert_eq!(
            quent::review::typed_spec(make(), &m, &other),
            Err(TypedError::SnapshotMismatch)
        );
        let protected = source(Some(
            json!({"op":"eq","field":"tenant","value":{"type":"i64","value":11}}),
        ));
        assert_eq!(
            quent::review::typed_spec(protected, &m, &c),
            Err(TypedError::TenantPredicate)
        );
    }
}
