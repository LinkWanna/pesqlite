mod common;

use common::test_parse;
use pesqlite::{Parser, *};

test_parse!(
    test_schema_object,
    Rule::schema_object,
    SchemaObject::parse,
    [
        (
            "sql.users",
            SchemaObject {
                schema_name: Some("sql".to_owned()),
                name: "users".to_owned(),
            }
        ),
        (
            "sql.users as u",
            SchemaObject {
                schema_name: Some("sql".to_owned()),
                name: "users".to_owned(),
            }
        )
    ]
);

test_parse!(
    test_qualified_table,
    Rule::qualified_table,
    QualifiedTable::parse,
    [
        (
            "sql.users as u",
            QualifiedTable {
                schema_table: SchemaObject {
                    schema_name: Some("sql".to_owned()),
                    name: "users".to_owned(),
                },
                alias: Some("u".to_owned()),
                indexed: None,
            }
        ),
        (
            "sql.users as u indexed by idx_users",
            QualifiedTable {
                schema_table: SchemaObject {
                    schema_name: Some("sql".to_owned()),
                    name: "users".to_owned(),
                },
                alias: Some("u".to_owned()),
                indexed: Some(Indexed::By("idx_users".to_owned())),
            }
        ),
        (
            "sql.users indexed by idx_users",
            QualifiedTable {
                schema_table: SchemaObject {
                    schema_name: Some("sql".to_owned()),
                    name: "users".to_owned(),
                },
                alias: None,
                indexed: Some(Indexed::By("idx_users".to_owned())),
            }
        )
    ]
);

test_parse!(
    test_select_core,
    Rule::select_core,
    SelectCore::parse,
    [
        (
            "VALUES (1, 'Alice'), (2, 'Bob')",
            SelectCore::Values(vec![
                vec![
                    Expr::Literal(Literal::Integer("1".to_owned())),
                    Expr::Literal(Literal::String("Alice".to_owned())),
                ],
                vec![
                    Expr::Literal(Literal::Integer("2".to_owned())),
                    Expr::Literal(Literal::String("Bob".to_owned())),
                ],
            ])
        ),
        (
            "SELECT id, name FROM sql.users WHERE age > 30",
            SelectCore::Query {
                is_distinct: false,
                columns: vec![
                    ResultColumn::Expr(Expr::QualifiedColumn(None, None, "id".to_owned()), None),
                    ResultColumn::Expr(Expr::QualifiedColumn(None, None, "name".to_owned()), None),
                ],
                from_clause: Some(FromClause::TableOrQuerys(vec![QualifiedTable {
                    schema_table: SchemaObject {
                        schema_name: Some("sql".to_owned()),
                        name: "users".to_owned(),
                    },
                    alias: None,
                    indexed: None,
                }])),
                where_clause: Some(Expr::Binary(
                    Box::new(Expr::QualifiedColumn(None, None, "age".to_owned())),
                    BinaryOp::Gt,
                    Box::new(Expr::Literal(Literal::Integer("30".to_owned()))),
                )),
                group_by: vec![],
                having: None
            }
        )
    ]
);

test_parse!(
    test_select,
    Rule::select,
    Select::parse,
    [
        (
            r"SELECT id, name FROM sql.users as u
            WHERE age > 30
            group by age
            having age > 20
            order by name
            limit 10 offset 5",
            Select {
                core: SelectCore::Query {
                    is_distinct: false,
                    columns: vec![
                        ResultColumn::Expr(
                            Expr::QualifiedColumn(None, None, "id".to_owned()),
                            None
                        ),
                        ResultColumn::Expr(
                            Expr::QualifiedColumn(None, None, "name".to_owned()),
                            None
                        ),
                    ],
                    from_clause: Some(FromClause::TableOrQuerys(vec![QualifiedTable {
                        schema_table: SchemaObject {
                            schema_name: Some("sql".to_owned()),
                            name: "users".to_owned(),
                        },
                        alias: Some("u".to_owned()),
                        indexed: None,
                    }])),
                    where_clause: Some(Expr::Binary(
                        Box::new(Expr::QualifiedColumn(None, None, "age".to_owned())),
                        BinaryOp::Gt,
                        Box::new(Expr::Literal(Literal::Integer("30".to_owned()))),
                    )),
                    group_by: vec![Expr::QualifiedColumn(None, None, "age".to_owned())],
                    having: Some(Expr::Binary(
                        Box::new(Expr::QualifiedColumn(None, None, "age".to_owned())),
                        BinaryOp::Gt,
                        Box::new(Expr::Literal(Literal::Integer("20".to_owned()))),
                    )),
                },
                compound: vec![],
                order_by: vec![OrderingTerm {
                    expr: Expr::QualifiedColumn(None, None, "name".to_owned()),
                    asc: true,
                    nulls_first: true,
                }],
                limit: Some(Expr::Literal(Literal::Integer("10".to_owned()))),
                offset: Some(Expr::Literal(Literal::Integer("5".to_owned()))),
            }
        ),
        (
            r"SELECT id, name FROM sql.users UNION ALL SELECT age FROM sql.users
            order by name
            limit 10 offset 5",
            Select {
                core: SelectCore::Query {
                    is_distinct: false,
                    columns: vec![
                        ResultColumn::Expr(
                            Expr::QualifiedColumn(None, None, "id".to_owned()),
                            None
                        ),
                        ResultColumn::Expr(
                            Expr::QualifiedColumn(None, None, "name".to_owned()),
                            None
                        ),
                    ],
                    from_clause: Some(FromClause::TableOrQuerys(vec![QualifiedTable {
                        schema_table: SchemaObject {
                            schema_name: Some("sql".to_owned()),
                            name: "users".to_owned(),
                        },
                        alias: None,
                        indexed: None,
                    }])),
                    where_clause: None,
                    group_by: vec![],
                    having: None,
                },
                compound: vec![(
                    CompoundOperator::Union(true),
                    SelectCore::Query {
                        is_distinct: false,
                        columns: vec![ResultColumn::Expr(
                            Expr::QualifiedColumn(None, None, "age".to_owned()),
                            None,
                        )],
                        from_clause: Some(FromClause::TableOrQuerys(vec![QualifiedTable {
                            schema_table: SchemaObject {
                                schema_name: Some("sql".to_owned()),
                                name: "users".to_owned(),
                            },
                            alias: None,
                            indexed: None,
                        }])),
                        where_clause: None,
                        group_by: vec![],
                        having: None,
                    }
                )],
                order_by: vec![OrderingTerm {
                    expr: Expr::QualifiedColumn(None, None, "name".to_owned()),
                    asc: true,
                    nulls_first: true,
                }],
                limit: Some(Expr::Literal(Literal::Integer("10".to_owned()))),
                offset: Some(Expr::Literal(Literal::Integer("5".to_owned()))),
            }
        )
    ]
);

test_parse!(
    test_insert,
    Rule::insert,
    Insert::parse,
    [
        (
            "Replace into sql.users (id, name) values (1, 'Alice'), (2, 'Bob')",
            Insert {
                header: InsertHeader::Replace,
                schema_table: SchemaObject {
                    schema_name: Some("sql".to_owned()),
                    name: "users".to_owned(),
                },
                alias: None,
                columns: vec!["id".to_owned(), "name".to_owned()],
                values: InsertValues::Values {
                    values: vec![
                        vec![
                            Expr::Literal(Literal::Integer("1".to_owned())),
                            Expr::Literal(Literal::String("Alice".to_owned())),
                        ],
                        vec![
                            Expr::Literal(Literal::Integer("2".to_owned())),
                            Expr::Literal(Literal::String("Bob".to_owned())),
                        ],
                    ],
                    upsert: None
                },
                return_clause: vec![],
            }
        ),
        (
            "Insert OR ROLLBACK into sql.users (id, name) values (1, 'Alice'), (2, 'Bob') returning id",
            Insert {
                header: InsertHeader::Insert(ConflictResolution::Rollback),
                schema_table: SchemaObject {
                    schema_name: Some("sql".to_owned()),
                    name: "users".to_owned(),
                },
                alias: None,
                columns: vec!["id".to_owned(), "name".to_owned()],
                values: InsertValues::Values {
                    values: vec![
                        vec![
                            Expr::Literal(Literal::Integer("1".to_owned())),
                            Expr::Literal(Literal::String("Alice".to_owned())),
                        ],
                        vec![
                            Expr::Literal(Literal::Integer("2".to_owned())),
                            Expr::Literal(Literal::String("Bob".to_owned())),
                        ],
                    ],
                    upsert: None
                },
                return_clause: vec![ReturnSubClause::Expr(
                    Expr::QualifiedColumn(None, None, "id".to_owned()),
                    None
                )],
            }
        )
    ]
);

test_parse!(
    test_set_sub_clause,
    Rule::set_sub_clause,
    SetSubClause::parse,
    [
        (
            "name = 'Alice'",
            SetSubClause {
                columns: vec!["name".to_owned()],
                value: Expr::Literal(Literal::String("Alice".to_owned())),
            }
        ),
        (
            "(name, job) = ('Bob', 'Teacher')",
            SetSubClause {
                columns: vec!["name".to_owned(), "job".to_owned()],
                value: Expr::ExprList(vec![
                    Expr::Literal(Literal::String("Bob".to_owned())),
                    Expr::Literal(Literal::String("Teacher".to_owned())),
                ]),
            }
        ),
    ]
);

test_parse!(
    test_join_constraint,
    Rule::join_constraint,
    JoinConstraint::parse,
    [
        (
            "on table1",
            JoinConstraint::Expr(Expr::QualifiedColumn(None, None, "table1".to_owned()))
        ),
        (
            "using (table1, table2, table3)",
            JoinConstraint::Using(vec![
                "table1".to_owned(),
                "table2".to_owned(),
                "table3".to_owned()
            ])
        )
    ]
);

test_parse!(
    test_join_operator,
    Rule::join_operator,
    JoinOperator::parse,
    [
        (",", JoinOperator::Comma),
        ("cross join", JoinOperator::Cross),
        ("join", JoinOperator::Inner(false)),
        ("inner join", JoinOperator::Inner(false)),
        ("natural inner join", JoinOperator::Inner(true)),
        (
            "left outer join",
            JoinOperator::Outer(false, OuterJoinType::Left)
        ),
        (
            "natural left outer join",
            JoinOperator::Outer(true, OuterJoinType::Left)
        ),
        (
            "right outer join",
            JoinOperator::Outer(false, OuterJoinType::Right)
        ),
        (
            "natural right outer join",
            JoinOperator::Outer(true, OuterJoinType::Right)
        ),
        (
            "full outer join",
            JoinOperator::Outer(false, OuterJoinType::Full)
        ),
        (
            "natural full outer join",
            JoinOperator::Outer(true, OuterJoinType::Full)
        ),
    ]
);

test_parse!(
    test_join_clause,
    Rule::join_clause,
    JoinClause::parse,
    [(
        "user left join orders on user.id = orders.user_id",
        JoinClause {
            table_or_subquery: QualifiedTable {
                schema_table: SchemaObject {
                    schema_name: None,
                    name: "user".to_owned()
                },
                alias: None,
                indexed: None
            },
            joins: vec![JoinSubClause {
                operator: JoinOperator::Outer(false, OuterJoinType::Left),
                table_or_subquery: QualifiedTable {
                    schema_table: SchemaObject {
                        schema_name: None,
                        name: "orders".to_owned()
                    },
                    alias: None,
                    indexed: None
                },
                constraint: Some(JoinConstraint::Expr(Expr::Binary(
                    Box::new(Expr::QualifiedColumn(
                        None,
                        Some("user".to_owned()),
                        "id".to_owned()
                    )),
                    BinaryOp::Eq,
                    Box::new(Expr::QualifiedColumn(
                        None,
                        Some("orders".to_owned()),
                        "user_id".to_owned()
                    ))
                )))
            }]
        }
    ),]
);

test_parse!(
    test_update,
    Rule::update,
    Update::parse,
    [
        (
            "update employee set eage = eage + 1 returning *",
            Update {
                conflict: ConflictResolution::Abort,
                qualified_table: QualifiedTable {
                    schema_table: SchemaObject {
                        schema_name: None,
                        name: "employee".to_owned(),
                    },
                    alias: None,
                    indexed: None
                },
                set_clause: vec![SetSubClause {
                    columns: vec!["eage".to_owned()],
                    value: Expr::Binary(
                        Box::new(Expr::QualifiedColumn(None, None, "eage".to_owned())),
                        BinaryOp::Plus,
                        Box::new(Expr::Literal(Literal::Integer("1".to_owned()))),
                    ),
                }],
                from_clause: None,
                where_clause: None,
                return_clause: vec![ReturnSubClause::Star],
            }
        ),
        (
            "update OR ROLLBACK employee set eage = eage + 1",
            Update {
                conflict: ConflictResolution::Rollback,
                qualified_table: QualifiedTable {
                    schema_table: SchemaObject {
                        schema_name: None,
                        name: "employee".to_owned(),
                    },
                    alias: None,
                    indexed: None
                },
                set_clause: vec![SetSubClause {
                    columns: vec!["eage".to_owned()],
                    value: Expr::Binary(
                        Box::new(Expr::QualifiedColumn(None, None, "eage".to_owned())),
                        BinaryOp::Plus,
                        Box::new(Expr::Literal(Literal::Integer("1".to_owned()))),
                    ),
                }],
                from_clause: None,
                where_clause: None,
                return_clause: vec![],
            }
        )
    ]
);

test_parse!(
    test_delete,
    Rule::delete,
    Delete::parse,
    [
        (
            "delete from employee where eid = 1001",
            Delete {
                qualified_table: QualifiedTable {
                    schema_table: SchemaObject {
                        schema_name: None,
                        name: "employee".to_owned(),
                    },
                    alias: None,
                    indexed: None
                },
                where_clause: Some(Expr::Binary(
                    Box::new(Expr::QualifiedColumn(None, None, "eid".to_owned())),
                    BinaryOp::Eq,
                    Box::new(Expr::Literal(Literal::Integer("1001".to_owned()))),
                )),
                return_clause: vec![],
            }
        ),
        (
            "delete from employee returning *",
            Delete {
                qualified_table: QualifiedTable {
                    schema_table: SchemaObject {
                        schema_name: None,
                        name: "employee".to_owned(),
                    },
                    alias: None,
                    indexed: None
                },
                where_clause: None,
                return_clause: vec![ReturnSubClause::Star],
            }
        )
    ]
);

test_parse!(
    test_ordering_term,
    Rule::ordering_term,
    OrderingTerm::parse,
    [(
        "eid + 1001 DESC NULLS LAST",
        OrderingTerm {
            expr: Expr::Binary(
                Box::new(Expr::QualifiedColumn(None, None, "eid".to_owned())),
                BinaryOp::Plus,
                Box::new(Expr::Literal(Literal::Integer("1001".to_owned()))),
            ),
            asc: false,
            nulls_first: false,
        }
    )]
);

test_parse!(
    test_result_column,
    Rule::result_column,
    ResultColumn::parse,
    [
        (
            "id",
            ResultColumn::Expr(Expr::QualifiedColumn(None, None, "id".to_owned()), None)
        ),
        (
            "id as identifier",
            ResultColumn::Expr(
                Expr::QualifiedColumn(None, None, "id".to_owned()),
                Some("identifier".to_owned())
            )
        ),
    ]
);

test_parse!(
    test_upsert_sub_clause,
    Rule::upsert_sub_clause,
    UpsertSubClause::parse,
    [
        (
            "ON CONFLICT (id) DO NOTHING",
            UpsertSubClause {
                indexed_cols: vec![IndexedColumn {
                    name: "id".to_owned(),
                    asc: true
                }],
                where_clause: None,
                upsert_type: UpsertType::Nothing,
            }
        ),
        (
            "ON CONFLICT (id desc, name) where name != 'Alice' DO UPDATE SET name = 'Bob'",
            UpsertSubClause {
                indexed_cols: vec![
                    IndexedColumn {
                        name: "id".to_owned(),
                        asc: false
                    },
                    IndexedColumn {
                        name: "name".to_owned(),
                        asc: true
                    }
                ],
                where_clause: Some(Expr::Binary(
                    Box::new(Expr::QualifiedColumn(None, None, "name".to_owned())),
                    BinaryOp::Ne,
                    Box::new(Expr::Literal(Literal::String("Alice".to_owned()))),
                )),
                upsert_type: UpsertType::Update {
                    set_clause: vec![SetSubClause {
                        columns: vec!["name".to_owned()],
                        value: Expr::Literal(Literal::String("Bob".to_owned())),
                    }],
                    where_clause: None
                },
            }
        ),
    ]
);
