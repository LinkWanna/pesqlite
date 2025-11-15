mod common;

use common::test_parse;
use pesqlite::*;

test_parse!(
    test_create_table,
    Rule::create_table,
    CreateTable::parse,
    [
        (
            r"create table users (
            id integer,
            name varchar(255) unique not null,
            age int unique not null,
            constraint pk primary key (id asc)
        )",
            CreateTable {
                temp: false,
                if_not_exists: false,
                schema_table: SchemaObject {
                    schema_name: None,
                    name: "users".to_owned(),
                },
                body: CreateTableBody::Columns {
                    columns: vec![
                        ColumnDef {
                            col_name: "id".to_owned(),
                            col_type: Some(TypeName {
                                name: "integer".to_owned(),
                                size: None,
                            }),
                            constraints: vec![],
                        },
                        ColumnDef {
                            col_name: "name".to_owned(),
                            col_type: Some(TypeName {
                                name: "varchar".to_owned(),
                                size: Some(TypeSize::MaxSize("255".to_owned())),
                            }),
                            constraints: vec![
                                ColumnConstraint {
                                    name: None,
                                    ty: ColumnConstraintType::Unique
                                },
                                ColumnConstraint {
                                    name: None,
                                    ty: ColumnConstraintType::NotNull
                                },
                            ],
                        },
                        ColumnDef {
                            col_name: "age".to_owned(),
                            col_type: Some(TypeName {
                                name: "int".to_owned(),
                                size: None,
                            }),
                            constraints: vec![
                                ColumnConstraint {
                                    name: None,
                                    ty: ColumnConstraintType::Unique
                                },
                                ColumnConstraint {
                                    name: None,
                                    ty: ColumnConstraintType::NotNull
                                },
                            ],
                        },
                    ],
                    table_constraints: vec![TableConstraint {
                        name: Some("pk".to_owned()),
                        cols: vec![IndexedColumn {
                            name: "id".to_owned(),
                            asc: true,
                        }],
                        ty: TableConstraintType::PrimaryKey,
                    },],
                    table_options: vec![],
                }
            }
        ),
        (
            r"create table x(id)",
            CreateTable {
                temp: false,
                if_not_exists: false,
                schema_table: SchemaObject {
                    schema_name: None,
                    name: "x".to_owned(),
                },
                body: CreateTableBody::Columns {
                    columns: vec![ColumnDef {
                        col_name: "id".to_owned(),
                        col_type: None,
                        constraints: vec![],
                    }],
                    table_constraints: vec![],
                    table_options: vec![],
                }
            }
        ),
    ]
);

test_parse!(
    test_drop_table,
    Rule::drop_table,
    DropTable::parse,
    [(
        "drop table if exists sql.users",
        DropTable {
            if_exists: true,
            schema_table: SchemaObject {
                schema_name: Some("sql".to_owned()),
                name: "users".to_owned(),
            },
        }
    ),]
);

test_parse!(
    test_alter_table,
    Rule::alter_table,
    AlterTable::parse,
    [
        (
            "ALTER TABLE employee ADD COLUMN email INT",
            AlterTable {
                schema_table: SchemaObject {
                    schema_name: None,
                    name: "employee".to_owned(),
                },
                action: AlterTableAction::AddColumn(ColumnDef {
                    col_name: "email".to_owned(),
                    col_type: Some(TypeName {
                        name: "int".to_owned(),
                        size: None,
                    }),
                    constraints: vec![],
                }),
            }
        ),
        (
            "ALTER TABLE employee RENAME TO employer",
            AlterTable {
                schema_table: SchemaObject {
                    schema_name: None,
                    name: "employee".to_owned(),
                },
                action: AlterTableAction::RenameTable("employer".to_owned()),
            }
        ),
    ]
);

test_parse!(
    test_create_index,
    Rule::create_index,
    CreateIndex::parse,
    [(
        "CREATE UNIQUE INDEX IF NOT EXISTS Index_eage ON employee(eage desc)",
        CreateIndex {
            unique: true,
            if_not_exists: true,
            schema_index: SchemaObject {
                schema_name: None,
                name: "index_eage".to_owned(),
            },
            table_name: "employee".to_owned(),
            indexed_cols: vec![IndexedColumn {
                name: "eage".to_owned(),
                asc: false,
            },],
            where_cond: None,
        }
    ),]
);

test_parse!(
    test_drop_index,
    Rule::drop_index,
    DropIndex::parse,
    [(
        "drop INDEX Index_eage",
        DropIndex {
            if_exists: false,
            schema_index: SchemaObject {
                schema_name: None,
                name: "index_eage".to_owned(),
            },
        }
    ),]
);

test_parse!(
    test_create_view,
    Rule::create_view,
    CreateView::parse,
    [(
        "CREATE TEMP VIEW View_employee(ename, eage) AS SELECT ename, eage FROM employee WHERE eage > 30",
        CreateView {
            temp: true,
            if_not_exists: false,
            schema_view: SchemaObject {
                schema_name: None,
                name: "view_employee".to_owned(),
            },
            columns: vec!["ename".to_owned(), "eage".to_owned()],
            select: Select {
                core: SelectCore::Query {
                    is_distinct: false,
                    columns: vec![
                        ResultColumn::Expr(
                            Expr::QualifiedColumn(None, None, "ename".to_owned()),
                            None
                        ),
                        ResultColumn::Expr(
                            Expr::QualifiedColumn(None, None, "eage".to_owned()),
                            None
                        ),
                    ],
                    from_clause: Some(FromClause::TableOrQuerys(vec![QualifiedTable {
                        schema_table: SchemaObject {
                            schema_name: None,
                            name: "employee".to_owned(),
                        },
                        alias: None,
                        indexed: None,
                    }])),
                    where_clause: Some(Expr::Binary(
                        Box::new(Expr::QualifiedColumn(None, None, "eage".to_owned())),
                        BinaryOp::Gt,
                        Box::new(Expr::Literal(Literal::Integer("30".to_owned()))),
                    )),
                    group_by: vec![],
                    having: None
                },
                compound: vec![],
                order_by: vec![],
                limit: None,
                offset: None,
            },
        }
    ),]
);

test_parse!(
    test_drop_view,
    Rule::drop_view,
    DropView::parse,
    [(
        "DROP VIEW View_employee",
        DropView {
            if_exists: false,
            schema_view: SchemaObject {
                schema_name: None,
                name: "view_employee".to_owned(),
            },
        }
    ),]
);

test_parse!(
    test_column_constraint,
    Rule::column_constraint,
    ColumnConstraint::parse,
    [
        (
            "primary key",
            ColumnConstraint {
                name: None,
                ty: ColumnConstraintType::PrimaryKey {
                    asc: true,
                    auto_inc: false
                }
            }
        ),
        (
            "CONSTRAINT pk primary key AUTOINCREMENT",
            ColumnConstraint {
                name: Some("pk".to_owned()),
                ty: ColumnConstraintType::PrimaryKey {
                    asc: true,
                    auto_inc: true
                }
            }
        ),
        (
            "default 0.",
            ColumnConstraint {
                name: None,
                ty: ColumnConstraintType::Default(Literal::Decimal("0.".to_owned()))
            }
        ),
    ]
);

test_parse!(
    test_type_name,
    Rule::type_name,
    TypeName::parse,
    [
        (
            "decimal(10, 2)",
            TypeName {
                name: "decimal".to_owned(),
                size: Some(TypeSize::TypeSize("10".to_owned(), "2".to_owned())),
            }
        ),
        (
            "varchar(255)",
            TypeName {
                name: "varchar".to_owned(),
                size: Some(TypeSize::MaxSize("255".to_owned())),
            }
        ),
    ]
);

test_parse!(
    test_indexed_column,
    Rule::indexed_column,
    IndexedColumn::parse,
    [
        (
            "name desc",
            IndexedColumn {
                name: "name".to_owned(),
                asc: false
            }
        ),
        (
            "name asc",
            IndexedColumn {
                name: "name".to_owned(),
                asc: true
            }
        ),
    ]
);

test_parse!(
    test_table_constraint,
    Rule::table_constraint,
    TableConstraint::parse,
    [
        (
            "constraint prime primary key (name desc)",
            TableConstraint {
                name: Some("prime".to_owned()),
                cols: vec![IndexedColumn {
                    name: "name".to_owned(),
                    asc: false,
                }],
                ty: TableConstraintType::PrimaryKey,
            }
        ),
        (
            "constraint uni unique (name desc, age asc)",
            TableConstraint {
                name: Some("uni".to_owned()),
                cols: vec![
                    IndexedColumn {
                        name: "name".to_owned(),
                        asc: false,
                    },
                    IndexedColumn {
                        name: "age".to_owned(),
                        asc: true,
                    }
                ],
                ty: TableConstraintType::Unique,
            }
        ),
    ]
);

test_parse!(
    test_column_def,
    Rule::column_def,
    ColumnDef::parse,
    [
        (
            "name varchar(255) unique not null",
            ColumnDef {
                col_name: "name".to_owned(),
                col_type: Some(TypeName {
                    name: "varchar".to_owned(),
                    size: Some(TypeSize::MaxSize("255".to_owned())),
                }),
                constraints: vec![
                    ColumnConstraint {
                        name: None,
                        ty: ColumnConstraintType::Unique
                    },
                    ColumnConstraint {
                        name: None,
                        ty: ColumnConstraintType::NotNull
                    },
                ],
            }
        ),
        (
            "name int",
            ColumnDef {
                col_name: "name".to_owned(),
                col_type: Some(TypeName {
                    name: "int".to_owned(),
                    size: None,
                }),
                constraints: vec![],
            }
        ),
        (
            "name not null",
            ColumnDef {
                col_name: "name".to_owned(),
                col_type: None,
                constraints: vec![ColumnConstraint {
                    name: None,
                    ty: ColumnConstraintType::NotNull
                }],
            }
        ),
    ]
);

test_parse!(
    test_create_table_body,
    Rule::create_table_body,
    CreateTableBody::parse,
    [
        (
            "AS SELECT * FROM users",
            CreateTableBody::Select(Select {
                core: SelectCore::Query {
                    is_distinct: false,
                    columns: vec![ResultColumn::Star],
                    from_clause: Some(FromClause::TableOrQuerys(vec![QualifiedTable {
                        schema_table: SchemaObject {
                            schema_name: None,
                            name: "users".to_owned(),
                        },
                        alias: None,
                        indexed: None,
                    }])),
                    where_clause: None,
                    group_by: vec![],
                    having: None,
                },
                compound: vec![],
                order_by: vec![],
                limit: None,
                offset: None
            })
        ),
        (
            "(id integer, unique (name) on conflict rollback) strict",
            CreateTableBody::Columns {
                columns: vec![ColumnDef {
                    col_name: "id".to_owned(),
                    col_type: Some(TypeName {
                        name: "integer".to_owned(),
                        size: None,
                    }),
                    constraints: vec![],
                },],
                table_constraints: vec![TableConstraint {
                    name: None,
                    cols: vec![IndexedColumn {
                        name: "name".to_owned(),
                        asc: true,
                    }],
                    ty: TableConstraintType::Unique,
                },],
                table_options: vec![TableOption::Strict]
            }
        ),
    ]
);

test_parse!(
    test_create_trigger,
    Rule::create_trigger,
    CreateTrigger::parse,
    [(
        "CREATE TRIGGER update_employee_age
        AFTER UPDATE ON employee
        BEGIN
            UPDATE employee SET age = age + 1 WHERE id = NEW.id;
        END",
        CreateTrigger {
            temp: false,
            if_not_exists: false,
            schema_trigger: SchemaObject {
                schema_name: None,
                name: "update_employee_age".to_owned(),
            },
            timing: TriggerTiming::After,
            event: TriggerEvent::Update(vec![]),
            table_name: "employee".to_owned(),
            when_cond: None,
            statements: vec![Dml::Update(Update {
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
                    columns: vec!["age".to_owned()],
                    value: Expr::Binary(
                        Box::new(Expr::QualifiedColumn(None, None, "age".to_owned())),
                        BinaryOp::Plus,
                        Box::new(Expr::Literal(Literal::Integer("1".to_owned()))),
                    )
                }],
                from_clause: None,
                where_clause: Some(Expr::Binary(
                    Box::new(Expr::QualifiedColumn(None, None, "id".to_owned(),)),
                    BinaryOp::Eq,
                    Box::new(Expr::QualifiedColumn(
                        None,
                        Some("new".to_owned()),
                        "id".to_owned(),
                    )),
                )),
                return_clause: vec![]
            })]
        }
    ),]
);

test_parse!(
    test_drop_trigger,
    Rule::drop_trigger,
    DropTrigger::parse,
    [(
        "DROP TRIGGER IF EXISTS update_employee_age",
        DropTrigger {
            if_exists: true,
            schema_trigger: SchemaObject {
                schema_name: None,
                name: "update_employee_age".to_owned(),
            },
        }
    ),]
);
