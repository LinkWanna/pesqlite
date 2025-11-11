use crate::{Rule, ast::*, parser::Parser};
use pest::iterators::Pair;

impl Parser for Select {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析 SELECT 语句核心
        let core = SelectCore::parse(pair);
        let mut pair = inner.next();

        // 解析复合操作符和后续的 SELECT 语句核心
        let mut compound = Vec::new();
        loop {
            match pair {
                Some(p) if p.as_rule() == Rule::compound_operator => {
                    println!("P: {:?}", p);

                    // 解析复合操作符
                    let inside = p.into_inner().next().unwrap();
                    let operator = match inside.as_rule() {
                        Rule::union => CompoundOperator::Union(false),
                        Rule::union_all => CompoundOperator::Union(true),
                        Rule::intersect => CompoundOperator::Intersect,
                        Rule::except => CompoundOperator::Except,
                        rule => panic!("Unexpected rule: {:?}", rule),
                    };

                    // 解析下一个 SELECT 语句核心
                    let next_core = SelectCore::parse(inner.next().unwrap());

                    // 添加到复合列表中
                    compound.push((operator, next_core));

                    // 继续解析下一个部分
                    pair = inner.next();
                }
                _ => break,
            }
        }

        // 解析 ORDER BY 子句（可选）
        let (order_by, pair) = match pair {
            Some(p) if p.as_rule() == Rule::ordering_terms => {
                let ordering_terms: Vec<OrderingTerm> = p
                    .into_inner()
                    .map(|pair| OrderingTerm::parse(pair))
                    .collect();
                (ordering_terms, inner.next())
            }
            _ => (vec![], pair),
        };

        // 解析 LIMIT 子句（可选）
        let (limit, offset) = match pair {
            Some(limit_pair) => {
                let limit = Expr::parse(limit_pair);
                let offset = inner.next().map(|pair| Expr::parse(pair));
                (Some(limit), offset)
            }
            _ => (None, None),
        };

        Self {
            compound,
            core,
            order_by,
            limit,
            offset,
        }
    }
}

impl Parser for SelectCore {
    fn parse(pair: Pair<Rule>) -> Self {
        match pair.as_rule() {
            Rule::select_core1 => {
                let mut inner = pair.into_inner();
                let pair = inner.next().unwrap();

                // 解析 distinct（可选）
                let (is_distinct, pair) = match pair.as_rule() {
                    Rule::distinct => (true, inner.next().unwrap()),
                    Rule::all => (false, inner.next().unwrap()),
                    _ => (false, pair),
                };

                // 解析结果列
                let columns: Vec<_> = pair
                    .into_inner()
                    .map(|col_pair| ResultColumn::parse(col_pair))
                    .collect();
                let pair = inner.next();

                // 解析 FROM 子句（可选）
                let (from_clause, pair) = match pair {
                    Some(pair) if pair.as_rule() == Rule::from_clause => {
                        (Some(FromClause::parse(pair)), inner.next())
                    }
                    _ => (None, pair),
                };

                // 解析 WHERE 子句（可选）
                let (where_clause, pair) = match pair {
                    Some(pair) if pair.as_rule() == Rule::expr => {
                        (Some(Expr::parse(pair)), inner.next())
                    }
                    _ => (None, pair),
                };

                // 解析 GROUP BY 子句（可选）
                let (group_by, pair) = match pair {
                    Some(pair) if pair.as_rule() == Rule::exprs => {
                        let group_exprs: Vec<Expr> = pair
                            .into_inner()
                            .map(|expr_pair| Expr::parse(expr_pair))
                            .collect();
                        (group_exprs, inner.next())
                    }
                    _ => (vec![], pair),
                };

                // 解析 HAVING 子句（可选）
                let having = pair.map(|p| Expr::parse(p));

                Self::Query {
                    is_distinct,
                    columns,
                    from_clause,
                    where_clause,
                    group_by,
                    having,
                }
            }
            Rule::select_core2 => {
                let mut inner = pair.into_inner();
                let pair = inner.next().unwrap();

                let values = pair
                    .into_inner()
                    .map(|exprs| {
                        exprs
                            .into_inner()
                            .map(|expr_pair| Expr::parse(expr_pair))
                            .collect()
                    })
                    .collect();
                Self::Values(values)
            }
            rule => panic!("Unexpected rule: {:?}", rule),
        }
    }
}

impl Parser for Insert {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析 insert 头部
        let header = match pair.as_rule() {
            Rule::insert_header1 => {
                let conflict = pair
                    .into_inner()
                    .next()
                    .map_or(ConflictResolution::Abort, |p| ConflictResolution::parse(p));

                InsertHeader::Insert(conflict)
            }
            Rule::insert_header2 => InsertHeader::Replace,
            rule => panic!("Unexpected rule: {:?}", rule),
        };
        let pair = inner.next().unwrap();

        // 解析表名
        let schema_table = SchemaObject::parse(pair);
        let pair = inner.next().unwrap();

        // 解析别名（可选）
        let (alias, pair) = match pair.as_rule() {
            Rule::ident => (Some(String::parse(pair)), inner.next().unwrap()),
            _ => (None, pair),
        };

        // 解析列名列表（可选）
        let (columns, pair) = match pair.as_rule() {
            Rule::idents => {
                let cols: Vec<_> = pair
                    .into_inner()
                    .map(|ident| String::parse(ident))
                    .collect();
                (cols, inner.next().unwrap())
            }
            _ => (vec![], pair),
        };

        // 解析插入值
        let values = match pair.as_rule() {
            Rule::insert_body1 => {
                let mut inner = pair.into_inner();
                let pair = inner.next().unwrap();

                // 解析 VALUES 子句
                let values = pair
                    .into_inner()
                    .map(|exprs| exprs.into_inner().map(|expr| Expr::parse(expr)).collect())
                    .collect();

                // 解析 UPSERT 子句（可选）
                let upsert = inner.next().map(|p| UpsertSubClause::parse(p));

                InsertValues::Values { values, upsert }
            }
            Rule::insert_body2 => {
                let mut inner = pair.into_inner();
                let pair = inner.next().unwrap();

                // 解析 SELECT 语句
                let select = Select::parse(pair);

                // 解析 UPSERT 子句（可选）
                let upsert = inner.next().map(|p| UpsertSubClause::parse(p));

                InsertValues::Select {
                    select: Box::new(select),
                    upsert,
                }
            }
            Rule::insert_body3 => InsertValues::Default,
            rule => unreachable!("Unexpected rule: {:?}", rule),
        };
        let pair = inner.next();

        // 解析 RETURNING 子句（可选）
        let return_clause: Vec<_> = pair.map_or(vec![], |pair| {
            pair.into_inner()
                .map(|pair| match pair.as_rule() {
                    Rule::return_sub_clause1 => ReturnSubClause::Star,
                    Rule::return_sub_clause2 => {
                        let mut inner = pair.into_inner();
                        let expr = Expr::parse(inner.next().unwrap());
                        let alias = inner.next().map(|ident| String::parse(ident));

                        ReturnSubClause::Expr(expr, alias)
                    }
                    rule => unreachable!("Unexpected rule: {:?}", rule),
                })
                .collect()
        });

        Self {
            header,
            schema_table,
            alias,
            columns,
            values,
            return_clause,
        }
    }
}

impl Parser for Update {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析冲突解决方案（可选）
        let (conflict, pair) = match pair.as_rule() {
            Rule::conflict_resolution => (ConflictResolution::parse(pair), inner.next().unwrap()),
            _ => (ConflictResolution::Abort, pair),
        };

        // 解析表名
        let qualified_table = QualifiedTable::parse(pair);
        let pair = inner.next().unwrap();

        // 解析赋值列表
        let set_clause: Vec<SetSubClause> = pair
            .into_inner()
            .map(|assignment| SetSubClause::parse(assignment))
            .collect();
        let pair = inner.next();

        // 解析 FROM 子句（可选）
        let (from_clause, pair) = match pair {
            Some(p) if p.as_rule() == Rule::from_clause => {
                (Some(FromClause::parse(p)), inner.next())
            }
            _ => (None, pair),
        };

        // 解析 WHERE 子句（可选）
        let (where_clause, pair) = match pair {
            Some(p) if p.as_rule() == Rule::expr => (Some(Expr::parse(p)), inner.next()),
            _ => (None, pair),
        };

        // 解析 RETURNING 子句（可选）
        let return_clause: Vec<_> = pair.map_or(vec![], |pair| {
            pair.into_inner()
                .map(|pair| match pair.as_rule() {
                    Rule::return_sub_clause1 => ReturnSubClause::Star,
                    Rule::return_sub_clause2 => {
                        let mut inner = pair.into_inner();
                        let expr = Expr::parse(inner.next().unwrap());
                        let alias = inner.next().map(|ident| String::parse(ident));

                        ReturnSubClause::Expr(expr, alias)
                    }
                    rule => unreachable!("Unexpected rule: {:?}", rule),
                })
                .collect()
        });

        Self {
            conflict,
            qualified_table,
            set_clause,
            from_clause,
            where_clause,
            return_clause,
        }
    }
}

impl Parser for Delete {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析表名
        let qualified_table = QualifiedTable::parse(pair);
        let pair = inner.next();

        // 解析 WHERE 子句（可选）
        let (where_clause, pair) = match pair {
            Some(p) if p.as_rule() == Rule::expr => (Some(Expr::parse(p)), inner.next()),
            _ => (None, pair),
        };

        // 解析 RETURNING 子句（可选）
        let return_clause: Vec<_> = pair.map_or(vec![], |pair| {
            pair.into_inner()
                .map(|pair| match pair.as_rule() {
                    Rule::return_sub_clause1 => ReturnSubClause::Star,
                    Rule::return_sub_clause2 => {
                        let mut inner = pair.into_inner();
                        let expr = Expr::parse(inner.next().unwrap());
                        let alias = inner.next().map(|ident| String::parse(ident));

                        ReturnSubClause::Expr(expr, alias)
                    }
                    rule => panic!("Unexpected rule: {:?}", rule),
                })
                .collect()
        });

        Self {
            qualified_table,
            where_clause,
            return_clause,
        }
    }
}

impl Parser for ResultColumn {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        match pair.as_rule() {
            Rule::result_column1 => {
                let mut inner = pair.into_inner();
                let expr = Expr::parse(inner.next().unwrap());
                let alias = inner.next().map(|ident| String::parse(ident));
                Self::Expr(expr, alias)
            }
            Rule::result_column2 => Self::Star,
            rule => panic!("Unexpected rule: {:?}", rule),
        }
    }
}

impl Parser for FromClause {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        match pair.as_rule() {
            Rule::from_clause1 => {
                let qualified_tables = pair
                    .into_inner()
                    .map(|table_pair| QualifiedTable::parse(table_pair))
                    .collect();
                FromClause::TableOrQuerys(qualified_tables)
            }
            Rule::from_clause2 => {
                FromClause::Join(JoinClause::parse(pair.into_inner().next().unwrap()))
            }
            rule => panic!("Unexpected rule: {:?}", rule),
        }
    }
}

impl Parser for OrderingTerm {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析表达式
        let expr = Expr::parse(pair);
        let pair = inner.next();

        // 解析排序方式（可选）
        let (asc, pair) = match pair {
            Some(p) if p.as_rule() == Rule::asc => (true, inner.next()),
            Some(p) if p.as_rule() == Rule::desc => (false, inner.next()),
            _ => (true, pair),
        };

        // 解析 NULLS FIRST/LAST（可选）
        let nulls_first = match pair {
            Some(p) if p.as_rule() == Rule::first => true,
            Some(p) if p.as_rule() == Rule::last => false,
            _ => true, // 默认 NULLS FIRST
        };

        Self {
            expr,
            asc,
            nulls_first,
        }
    }
}

impl Parser for SetSubClause {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析列名列表
        let (columns, pair) = match pair.as_rule() {
            Rule::idents => {
                let cols: Vec<_> = pair
                    .into_inner()
                    .map(|ident| String::parse(ident))
                    .collect();
                (cols, inner.next().unwrap())
            }
            Rule::ident => (vec![String::parse(pair)], inner.next().unwrap()),
            rule => unreachable!("Unexpected rule: {:?}", rule),
        };

        // 解析赋值表达式
        let value = Expr::parse(pair);

        Self { columns, value }
    }
}

impl Parser for JoinClause {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();

        // 解析连接操作符
        let qualified_table = QualifiedTable::parse(inner.next().unwrap());

        // 解析 JOIN SUB CLAUSE 列表
        let joins: Vec<_> = inner
            .map(|sub_clause_pair| JoinSubClause::parse(sub_clause_pair))
            .collect();

        Self {
            table_or_subquery: qualified_table,
            joins,
        }
    }
}

impl Parser for JoinSubClause {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析连接符
        let join_operator = JoinOperator::parse(pair);
        let pair = inner.next().unwrap();

        // 解析连接表
        let qualified_table = QualifiedTable::parse(pair);
        let pair = inner.next();

        // 解析连接约束（可选）
        let join_constraint = pair.map(|p| JoinConstraint::parse(p));

        Self {
            operator: join_operator,
            table_or_subquery: qualified_table,
            constraint: join_constraint,
        }
    }
}

impl Parser for JoinOperator {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();
        match pair.as_rule() {
            Rule::join_operator1 => Self::Comma,
            Rule::join_operator2 => Self::Cross,
            Rule::join_operator3 => {
                let mut inner = pair.into_inner();
                let pair = inner.next().unwrap();

                // 解析是否为 NATURAL 连接（可选）
                let (natural, pair) = match pair.as_rule() {
                    Rule::natural => (true, inner.next().unwrap()),
                    _ => (false, pair),
                };

                match pair.as_rule() {
                    Rule::join_operator31 => Self::Inner(natural),
                    Rule::left => Self::Outer(natural, OuterJoinType::Left),
                    Rule::right => Self::Outer(natural, OuterJoinType::Right),
                    Rule::full => Self::Outer(natural, OuterJoinType::Full),
                    rule => panic!("Unexpected rule: {:?}", rule),
                }
            }
            rule => panic!("Unexpected rule: {:?}", rule),
        }
    }
}

impl Parser for JoinConstraint {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();
        match pair.as_rule() {
            Rule::expr => Self::Expr(Expr::parse(pair)),
            Rule::idents => {
                let columns = pair
                    .into_inner()
                    .map(|ident| String::parse(ident))
                    .collect();
                Self::Using(columns)
            }
            rule => panic!("Unexpected rule: {:?}", rule),
        }
    }
}

impl Parser for QualifiedTable {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        let schema_table = SchemaObject::parse(pair);
        let pair = inner.next();

        // 解析别名（可选）
        let (alias, pair) = match pair {
            Some(p) if p.as_rule() == Rule::ident => (Some(String::parse(p)), inner.next()),
            _ => (None, pair),
        };

        // 解析索引信息（可选）
        let indexed = match pair {
            Some(pair) if pair.as_rule() == Rule::indexed1 => {
                let mut inner = pair.into_inner();
                let index = String::parse(inner.next().unwrap());
                Some(Indexed::By(index))
            }
            Some(pair) if pair.as_rule() == Rule::indexed2 => Some(Indexed::NotIndexed),
            _ => None,
        };

        Self {
            schema_table,
            alias,
            indexed,
        }
    }
}

impl Parser for UpsertSubClause {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析冲突列（可选）
        let (indexed_cols, where_clause, pair) = match pair.as_rule() {
            Rule::conflict_columns => {
                let mut inside = pair.into_inner();
                let pair = inside.next().unwrap();

                // 解析列名列表
                let cols: Vec<_> = pair
                    .into_inner()
                    .map(|col| IndexedColumn::parse(col))
                    .collect();
                let pair = inside.next();

                // 解析 WHERE 子句（可选）
                let where_clause = pair.map(|p| Expr::parse(p));

                (cols, where_clause, inner.next().unwrap())
            }
            _ => (vec![], None, pair),
        };

        // 解析 upsert 类型
        let upsert_type = match pair.as_rule() {
            Rule::upsert_sub_clause1 => UpsertType::Nothing,
            Rule::upsert_sub_clause2 => {
                let mut inner = pair.into_inner();
                let pair = inner.next().unwrap();

                // 解析赋值列表
                let set_clause: Vec<SetSubClause> = pair
                    .into_inner()
                    .map(|assignment| SetSubClause::parse(assignment))
                    .collect();

                // 解析 WHERE 子句（可选）
                let where_clause = inner.next().map(|p| Expr::parse(p));

                UpsertType::Update {
                    set_clause,
                    where_clause,
                }
            }
            rule => panic!("Unexpected rule: {:?}", rule),
        };

        Self {
            indexed_cols,
            where_clause,
            upsert_type,
        }
    }
}
