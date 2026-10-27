use super::*;
fn id(s: &str) -> Ident {
    Ident::new(s).unwrap()
}
fn col(name: &str, ty: SqlType) -> MigrationColumn {
    MigrationColumn::new(
        id(name),
        Observed::Known(ty),
        Observed::Known(true),
        MigrationDefault::Absent,
    )
    .unwrap()
}
fn complete() -> TableCoverage {
    TableCoverage {
        columns: Knowledge::Complete,
        indexes: Knowledge::Complete,
        foreign_keys: Knowledge::Complete,
        checks: Knowledge::Complete,
    }
}
fn schema(d: Dialect) -> MigrationSchema {
    MigrationSchema::new(
        MigrationTarget::new("CANARY_database".into(), "CANARY_route".into(), id("ns"), d).unwrap(),
        SchemaAuthority::ApplicationDeclared,
        Knowledge::Complete,
        vec![
            MigrationTable::new(
                MigrationName::new(id("ns"), id("probe")),
                TableEffects::Ordinary,
                complete(),
                vec![col("a", SqlType::I64), col("b", SqlType::I32)],
                vec![],
                vec![],
                vec![],
            )
            .unwrap(),
        ],
    )
    .unwrap()
}
fn area(plan: &MigrationPlan, area: MigrationArea) -> AreaView {
    *plan
        .public_view()
        .areas()
        .iter()
        .find(|a| a.area == area)
        .unwrap()
}
fn refused(b: &MigrationSchema, a: &MigrationSchema) -> MigrationPlan {
    let p = plan_migration(b, a);
    assert_eq!(p.public_view().status(), MigrationStatus::Refused);
    assert!(p.sensitive_source().sql().is_empty());
    assert_eq!(p.public_view().risk(), Risk::Blocked);
    p
}
#[test]
fn c22_both_dialects_exact_finite_sql_and_nochange() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let b = schema(d);
        assert_eq!(
            plan_migration(&b, &b).public_view().status(),
            MigrationStatus::NoChange
        );
        let mut a = b.clone();
        for (name, ty) in [
            ("small", SqlType::I16),
            ("normal", SqlType::I32),
            ("big", SqlType::I64),
        ] {
            a.tables[0].columns.push(col(name, ty));
        }
        let p = plan_migration(&b, &a);
        assert_eq!(p.public_view().status(), MigrationStatus::SqlPlanned);
        assert_eq!(p.public_view().steps(), 3);
        assert_eq!(p.public_view().risk(), Risk::High);
        let q = if d == Dialect::Postgres { '"' } else { '`' };
        let expected: Vec<_> = [
            ("small", "SMALLINT"),
            ("normal", "INTEGER"),
            ("big", "BIGINT"),
        ]
        .map(|(n, t)| format!("ALTER TABLE {q}ns{q}.{q}probe{q} ADD COLUMN {q}{n}{q} {t} NULL"))
        .into();
        assert_eq!(p.sensitive_source().sql(), expected);
        assert_eq!(area(&p, MigrationArea::Columns).changes(), 3);
        assert!(
            p.public_view()
                .areas()
                .iter()
                .all(|a| !a.unknown() && !a.unsupported())
        );
    }
}
#[test]
fn c22_column_type_nullable_default_removal_order_and_mixed_refusal() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let b = schema(d);
        for (area_id, change) in [
            (MigrationArea::Types, 0),
            (MigrationArea::Nullability, 1),
            (MigrationArea::Defaults, 2),
            (MigrationArea::Columns, 3),
            (MigrationArea::Columns, 4),
        ] {
            let mut a = b.clone();
            match change {
                0 => a.tables[0].columns[0].native = Observed::Known(SqlType::I32),
                1 => a.tables[0].columns[0].nullable = Observed::Known(false),
                2 => a.tables[0].columns[0].default = MigrationDefault::Literal(Value::I64(7)),
                3 => {
                    a.tables[0].columns.remove(0);
                }
                _ => a.tables[0].columns.reverse(),
            }
            a.tables[0].columns.push(col("safe_add", SqlType::I64));
            let p = refused(&b, &a);
            assert!(area(&p, area_id).changes() > 0);
            assert!(area(&p, area_id).unsupported());
        }
    }
}
#[test]
fn c22_defaults_absent_explicit_null_literal_opaque_unobserved_distinct() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let defaults = [
            MigrationDefault::Absent,
            MigrationDefault::Literal(Value::Null(SqlType::I64)),
            MigrationDefault::Literal(Value::I64(0)),
            MigrationDefault::Unobserved,
            MigrationDefault::Unknown,
            MigrationDefault::Opaque(OpaqueDefinition::new("CANARY_default()".into()).unwrap()),
        ];
        for (i, one) in defaults.iter().enumerate() {
            for (j, two) in defaults.iter().enumerate() {
                let mut b = schema(d);
                let mut a = b.clone();
                b.tables[0].columns[0].default = one.clone();
                a.tables[0].columns[0].default = two.clone();
                let p = plan_migration(&b, &a);
                assert_eq!(
                    area(&p, MigrationArea::Defaults).changes(),
                    usize::from(i != j)
                );
                if i != j || i >= 3 {
                    assert_eq!(p.public_view().status(), MigrationStatus::Refused);
                } else {
                    assert_eq!(p.public_view().status(), MigrationStatus::NoChange);
                }
            }
        }
    }
}
#[test]
fn c22_same_name_index_definition_unique_direction_and_composite_order() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let mut b = schema(d);
        let index = IndexDefinition::plain_btree(
            IndexKind::NonUnique,
            vec![(id("a"), Direction::Asc), (id("b"), Direction::Desc)],
        )
        .unwrap();
        b.tables[0].indexes.push(MigrationIndex::new(
            id("same_index"),
            Observed::Known(index),
        ));
        assert_eq!(
            plan_migration(&b, &b).public_view().status(),
            MigrationStatus::NoChange
        );
        for variant in 0..6 {
            let mut a = b.clone();
            match variant {
                0 => {
                    if let Observed::Known(i) = &mut a.tables[0].indexes[0].definition {
                        i.columns.reverse();
                    }
                }
                1 => {
                    if let Observed::Known(i) = &mut a.tables[0].indexes[0].definition {
                        i.kind = IndexKind::Unique;
                    }
                }
                2 => {
                    if let Observed::Known(i) = &mut a.tables[0].indexes[0].definition {
                        i.columns[0].1 = Direction::Desc;
                    }
                }
                3 => a.tables[0].indexes[0].definition = Observed::Unknown,
                4 => a.tables[0].indexes.clear(),
                _ => a.tables[0]
                    .indexes
                    .push(MigrationIndex::new(id("new"), Observed::Unknown)),
            }
            let p = refused(&b, &a);
            assert!(area(&p, MigrationArea::Indexes).changes() > 0);
        }
        b.tables[0].indexes[0].definition = Observed::Unknown;
        assert!(area(&refused(&b, &b), MigrationArea::Indexes).unknown());
    }
}
#[test]
fn c22_fk_same_name_order_target_actions_and_checks() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let mut b = schema(d);
        let mut parent = b.tables[0].clone();
        parent.name.table = id("parent");
        b.tables.push(parent);
        let fk = ForeignKeyDefinition::simple(
            vec![id("a"), id("b")],
            b.tables[1].name.clone(),
            vec![id("a"), id("b")],
            ReferentialAction::NoAction,
            ReferentialAction::Cascade,
        )
        .unwrap();
        b.tables[0]
            .foreign_keys
            .push(MigrationForeignKey::new(id("same_fk"), Observed::Known(fk)));
        for change in 0..6 {
            let mut a = b.clone();
            if let Observed::Known(fk) = &mut a.tables[0].foreign_keys[0].definition {
                match change {
                    0 => fk.columns.reverse(),
                    1 => fk.referenced_columns.reverse(),
                    2 => fk.referenced.table = id("probe"),
                    3 => fk.on_delete = ReferentialAction::Restrict,
                    4 => fk.on_update = ReferentialAction::SetNull,
                    _ => {}
                }
            }
            if change == 5 {
                a.tables[0].foreign_keys[0].definition = Observed::Unknown;
            }
            assert_eq!(
                area(&refused(&b, &a), MigrationArea::ForeignKeys).changes(),
                1
            );
        }
        b.tables[0].checks.push(MigrationCheck::new(
            id("same_check"),
            Observed::Known(OpaqueDefinition::new("CANARY_check_a > 0".into()).unwrap()),
        ));
        assert!(area(&refused(&b, &b), MigrationArea::Checks).unknown());
        let mut a = b.clone();
        a.tables[0].checks[0].definition =
            Observed::Known(OpaqueDefinition::new("CANARY_check_a >= 0".into()).unwrap());
        assert_eq!(area(&refused(&b, &a), MigrationArea::Checks).changes(), 1);
    }
}
#[test]
fn c19_c22_unknown_coverage_cannot_be_nochange() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let b = schema(d);
        for change in 0..10 {
            let mut a = b.clone();
            match change {
                0 => a.coverage = Knowledge::Unknown,
                1 => a.tables[0].coverage.columns = Knowledge::Partial,
                2 => a.tables[0].coverage.indexes = Knowledge::Unknown,
                3 => a.tables[0].coverage.foreign_keys = Knowledge::Partial,
                4 => a.tables[0].coverage.checks = Knowledge::Unknown,
                5 => a.tables[0].effects = TableEffects::Unknown,
                6 => a.tables[0].columns[0].native = Observed::Unknown,
                7 => a.tables[0].columns[0].nullable = Observed::Unknown,
                8 => a.tables[0].columns[0].default = MigrationDefault::Unobserved,
                _ => a.tables[0].effects = TableEffects::Unsupported,
            }
            let p = refused(&a, &a);
            assert!(
                p.public_view()
                    .areas()
                    .iter()
                    .any(|a| a.unknown() || a.unsupported())
            );
            let mut after = a.clone();
            after.tables[0].columns.push(col("safe_add", SqlType::I64));
            refused(&a, &after);
        }
    }
}
#[test]
fn c12_c19_c22_target_namespace_authority_and_table_order_exact() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let b = schema(d);
        for change in 0..6 {
            let mut a = b.clone();
            match change {
                0 => a.target.address.push('2'),
                1 => a.target.database.push('2'),
                2 => a.target.catalog = id("other"),
                3 => {
                    a.target.dialect = if d == Dialect::Postgres {
                        Dialect::Mysql
                    } else {
                        Dialect::Postgres
                    }
                }
                4 => a.authority = SchemaAuthority::UnverifiedSource,
                _ => a.tables[0].name.namespace = id("other"),
            }
            let p = refused(&b, &a);
            assert!(p.public_view().areas().iter().any(|a| a.changes() > 0));
        }
        let mut b = b;
        let mut other = b.tables[0].clone();
        other.name.table = id("other");
        b.tables.push(other);
        let mut a = b.clone();
        a.tables.reverse();
        assert_eq!(area(&refused(&b, &a), MigrationArea::Tables).changes(), 1);
        if d == Dialect::Postgres {
            let mut a = b.clone();
            a.tables[1].name.namespace = id("elsewhere");
            a.tables[1].name.table = id("probe");
            assert_eq!(area(&refused(&b, &a), MigrationArea::Tables).changes(), 2);
        }
    }
}
#[test]
fn c22_index_fk_check_enumeration_order_preserved() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let mut b = schema(d);
        for n in ["one", "two"] {
            b.tables[0]
                .indexes
                .push(MigrationIndex::new(id(n), Observed::Unknown));
            b.tables[0]
                .foreign_keys
                .push(MigrationForeignKey::new(id(n), Observed::Unknown));
            b.tables[0]
                .checks
                .push(MigrationCheck::new(id(n), Observed::Unknown));
        }
        let mut a = b.clone();
        a.tables[0].indexes.reverse();
        a.tables[0].foreign_keys.reverse();
        a.tables[0].checks.reverse();
        let p = refused(&b, &a);
        for area_id in [
            MigrationArea::Indexes,
            MigrationArea::ForeignKeys,
            MigrationArea::Checks,
        ] {
            assert_eq!(area(&p, area_id).changes(), 1);
        }
    }
}
#[test]
fn c22_quoted_case_and_native_dialect_capabilities() {
    let b = schema(Dialect::Postgres);
    let mut a = b.clone();
    a.tables[0]
        .columns
        .push(col("CANARY_\"quoted`", SqlType::I64));
    assert!(
        plan_migration(&b, &a).sensitive_source().sql()[0]
            .contains("\"CANARY_\"\"quoted`\" BIGINT")
    );
    for d in [Dialect::Postgres, Dialect::Mysql] {
        for ty in [
            SqlType::I8,
            SqlType::U8,
            SqlType::U16,
            SqlType::U32,
            SqlType::U64,
            SqlType::Bool,
            SqlType::Text,
        ] {
            let b = schema(d);
            let mut a = b.clone();
            a.tables[0].columns.push(col("extra", ty));
            let p = plan_migration(&b, &a);
            assert_eq!(
                p.public_view().status(),
                if d == Dialect::Mysql && ty == SqlType::I8 {
                    MigrationStatus::SqlPlanned
                } else {
                    MigrationStatus::Refused
                }
            );
        }
        let b = schema(d);
        let mut a = b.clone();
        a.tables[0].columns[0].name = id("A");
        refused(&b, &a);
        let mut a = b.clone();
        a.tables[0].columns.insert(0, col("first", SqlType::I64));
        refused(&b, &a);
    }
}
#[test]
fn c24_constructor_budget_shape_and_type_boundaries() {
    assert!(OpaqueDefinition::new("x".repeat(4096)).is_ok());
    assert_eq!(
        OpaqueDefinition::new("x".repeat(4097)).unwrap_err(),
        PlanError::Budget
    );
    assert!(
        MigrationTarget::new(
            "x".repeat(128),
            "x".repeat(128),
            id("ns"),
            Dialect::Postgres
        )
        .is_ok()
    );
    assert!(
        MigrationTarget::new("x".repeat(129), "x".into(), id("ns"), Dialect::Postgres).is_err()
    );
    assert_eq!(
        MigrationColumn::new(
            id("a"),
            Observed::Known(SqlType::I32),
            Observed::Known(true),
            MigrationDefault::Literal(Value::I64(0))
        )
        .unwrap_err(),
        PlanError::Type
    );
    assert!(
        MigrationColumn::new(
            id("a"),
            Observed::Known(SqlType::Text),
            Observed::Known(true),
            MigrationDefault::Literal(Value::Text("x".repeat(4096)))
        )
        .is_ok()
    );
    assert_eq!(
        MigrationColumn::new(
            id("a"),
            Observed::Known(SqlType::Text),
            Observed::Known(true),
            MigrationDefault::Literal(Value::Text("x".repeat(4097)))
        )
        .unwrap_err(),
        PlanError::Budget
    );
    assert!(IndexDefinition::plain_btree(IndexKind::Unique, vec![]).is_err());
    let cols: Vec<_> = (0..16)
        .map(|i| (id(&format!("c{i}")), Direction::Asc))
        .collect();
    assert!(IndexDefinition::plain_btree(IndexKind::Unique, cols.clone()).is_ok());
    let mut too_many = cols;
    too_many.push((id("extra"), Direction::Asc));
    assert_eq!(
        IndexDefinition::plain_btree(IndexKind::Unique, too_many).unwrap_err(),
        PlanError::Budget
    );
    assert!(
        ForeignKeyDefinition::simple(
            vec![id("a")],
            MigrationName::new(id("ns"), id("parent")),
            vec![id("a"), id("b")],
            ReferentialAction::NoAction,
            ReferentialAction::NoAction
        )
        .is_err()
    );
    let make_table = |count: usize| {
        MigrationTable::new(
            MigrationName::new(id("ns"), id("t")),
            TableEffects::Ordinary,
            complete(),
            (0..count)
                .map(|i| col(&format!("c{i}"), SqlType::I64))
                .collect(),
            vec![],
            vec![],
            vec![],
        )
    };
    assert!(make_table(256).is_ok());
    assert_eq!(make_table(257).unwrap_err(), PlanError::Budget);
    assert_eq!(make_table(0).unwrap_err(), PlanError::Shape);
    let b = schema(Dialect::Postgres);
    for count in [16, 17] {
        let tables = (0..count)
            .map(|i| {
                let mut t = b.tables[0].clone();
                t.name.table = id(&format!("t{i}"));
                t
            })
            .collect();
        let result = MigrationSchema::new(b.target.clone(), b.authority, b.coverage, tables);
        assert_eq!(result.is_ok(), count == 16);
    }
    let mut t = b.tables[0].clone();
    t.columns.push(t.columns[0].clone());
    assert!(
        MigrationTable::new(
            t.name,
            t.effects,
            t.coverage,
            t.columns,
            vec![],
            vec![],
            vec![]
        )
        .is_err()
    );
}
#[test]
fn c24_named_limits_dangling_and_mysql_case_collision() {
    let b = schema(Dialect::Mysql);
    let t = &b.tables[0];
    for count in [16, 17] {
        let indexes = (0..count)
            .map(|i| MigrationIndex::new(id(&format!("i{i}")), Observed::Unknown))
            .collect();
        let fks = (0..count)
            .map(|i| MigrationForeignKey::new(id(&format!("f{i}")), Observed::Unknown))
            .collect();
        let checks = (0..count)
            .map(|i| MigrationCheck::new(id(&format!("k{i}")), Observed::Unknown))
            .collect();
        for result in [
            MigrationTable::new(
                t.name.clone(),
                t.effects,
                t.coverage,
                t.columns.clone(),
                indexes,
                vec![],
                vec![],
            ),
            MigrationTable::new(
                t.name.clone(),
                t.effects,
                t.coverage,
                t.columns.clone(),
                vec![],
                fks,
                vec![],
            ),
            MigrationTable::new(
                t.name.clone(),
                t.effects,
                t.coverage,
                t.columns.clone(),
                vec![],
                vec![],
                checks,
            ),
        ] {
            assert_eq!(result.is_ok(), count == 16);
        }
    }
    let bad =
        IndexDefinition::plain_btree(IndexKind::Unique, vec![(id("missing"), Direction::Asc)])
            .unwrap();
    assert!(
        MigrationTable::new(
            t.name.clone(),
            t.effects,
            t.coverage,
            t.columns.clone(),
            vec![MigrationIndex::new(id("bad"), Observed::Known(bad))],
            vec![],
            vec![]
        )
        .is_err()
    );
    let mut t = t.clone();
    t.columns.push(col("A", SqlType::I64));
    assert!(MigrationSchema::new(b.target.clone(), b.authority, b.coverage, vec![t]).is_err());
    let mut t = b.tables[0].clone();
    t.name.namespace = id("other");
    assert!(MigrationSchema::new(b.target, b.authority, b.coverage, vec![t]).is_err());
}
#[test]
fn c01_c12_determinism_and_retained_immutable_snapshots() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let b = schema(d);
        let mut a = b.clone();
        a.tables[0].columns.push(col("extra", SqlType::I64));
        let p = plan_migration(&b, &a);
        let json = p.public_view().to_json();
        let sql = p.sensitive_source().sql().to_vec();
        for _ in 0..32 {
            let again = plan_migration(&b, &a);
            assert_eq!(again.public_view().to_json(), json);
            assert_eq!(again.sensitive_source().sql(), sql);
        }
        a.tables.clear();
        assert_eq!(p.public_view().to_json(), json);
        assert_eq!(p.sensitive_source().sql(), sql);
        assert_eq!(p.sensitive_source().schemas().1.tables.len(), 1);
    }
}
#[test]
fn c15_public_canary_and_positive_controls() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let mut b = schema(d);
        b.tables[0].name.table = id("canary_table");
        let mut a = b.clone();
        a.tables[0].columns.push(col("canary_column", SqlType::I64));
        let p = plan_migration(&b, &a);
        // Detection positive control: sensitive source really contains canaries.
        assert!(p.sensitive_source().sql()[0].contains("canary_table"));
        assert!(p.sensitive_source().sql()[0].contains("canary_column"));
        let public = format!(
            "{p:?} {:?} {} {:?}",
            p.public_view(),
            p.public_view(),
            p.sensitive_source()
        );
        for canary in [
            "canary_table",
            "canary_column",
            "CANARY_database",
            "CANARY_route",
            "ALTER TABLE",
            "BIGINT",
        ] {
            assert!(!public.contains(canary), "public leak");
        }
        a.tables[0].columns[0].default =
            MigrationDefault::Opaque(OpaqueDefinition::new("CANARY_default_expr".into()).unwrap());
        let p = refused(&b, &a);
        assert!(!format!("{p:?} {} {:?}", p.public_view(), a).contains("CANARY_default_expr"));
        let error = MigrationColumn::new(
            id("canary_column"),
            Observed::Known(SqlType::I64),
            Observed::Known(true),
            MigrationDefault::Literal(Value::Text("CANARY_value".into())),
        )
        .unwrap_err();
        assert!(!format!("{error} {error:?}").contains("CANARY"));
        assert!(std::error::Error::source(&error).is_none());
    }
}

#[test]
fn c22_opaque_partial_expression_indexes_never_establish_equivalence() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let mut b = schema(d);
        b.tables[0].indexes.push(MigrationIndex::new(
            id("same"),
            Observed::Known(IndexDefinition::opaque(
                OpaqueDefinition::new("CANARY_expression WHERE a > 0".into()).unwrap(),
            )),
        ));
        assert!(area(&refused(&b, &b), MigrationArea::Indexes).unknown());
        let mut a = b.clone();
        a.tables[0].indexes[0].definition = Observed::Known(IndexDefinition::opaque(
            OpaqueDefinition::new("CANARY_expression WHERE a >= 0".into()).unwrap(),
        ));
        assert_eq!(area(&refused(&b, &a), MigrationArea::Indexes).changes(), 1);
    }
}
#[test]
fn c24_maximum_retained_plan_and_last_table_refusal_are_bounded() {
    for d in [Dialect::Postgres, Dialect::Mysql] {
        let base = schema(d);
        let before = (0..16)
            .map(|i| {
                let mut t = base.tables[0].clone();
                t.name.table = id(&format!("t{i}"));
                t.columns.truncate(1);
                t
            })
            .collect::<Vec<_>>();
        let after = before
            .iter()
            .map(|t| {
                let mut t = t.clone();
                t.columns
                    .extend((1..256).map(|i| col(&format!("c{i}"), SqlType::I64)));
                t
            })
            .collect();
        let b = MigrationSchema::new(base.target.clone(), base.authority, base.coverage, before)
            .unwrap();
        let mut a =
            MigrationSchema::new(base.target, base.authority, base.coverage, after).unwrap();
        let p = plan_migration(&b, &a);
        assert_eq!(p.public_view().steps(), 16 * 255);
        assert!(p.sensitive_source().sql().iter().all(|s| s.len() < 512));
        a.tables[15].coverage.checks = Knowledge::Unknown;
        refused(&b, &a);
    }
}
