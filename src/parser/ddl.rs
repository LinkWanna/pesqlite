use crate::{Rule, ast::*, parser::Parser};
use pest::iterators::Pair;

impl Parser for CreateTable {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 是否为临时表（可选）
        let (temp, pair) = match pair.as_rule() {
            Rule::temp => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 是否存在则不创建（可选）
        let (if_not_exists, pair) = match pair.as_rule() {
            Rule::if_not_exists => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析模式名.表名
        let schema_table = SchemaObject::parse(pair);
        let pair = inner.next().unwrap();

        // 解析表体
        let body = CreateTableBody::parse(pair);

        Self {
            temp,
            if_not_exists,
            schema_table,
            body,
        }
    }
}

impl Parser for DropTable {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 是否存在则删除（可选）
        let (if_exists, pair) = match pair.as_rule() {
            Rule::if_exists => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析模式名.表名
        let schema_table = SchemaObject::parse(pair);

        Self {
            if_exists,
            schema_table,
        }
    }
}

impl Parser for AlterTable {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析模式名.表名
        let schema_table = SchemaObject::parse(pair);
        let pair = inner.next().unwrap();

        // 解析改表操作
        let action = match pair.as_rule() {
            Rule::alter_table_action1 => {
                let name = String::parse(pair.into_inner().next().unwrap());
                AlterTableAction::RenameTable(name)
            }
            Rule::alter_table_action2 => {
                let mut inner = pair.into_inner();
                let old_name = String::parse(inner.next().unwrap());
                let new_name = String::parse(inner.next().unwrap());
                AlterTableAction::RenameColumn(old_name, new_name)
            }
            Rule::alter_table_action3 => {
                let column_def = ColumnDef::parse(pair.into_inner().next().unwrap());
                AlterTableAction::AddColumn(column_def)
            }
            Rule::alter_table_action4 => {
                let name = String::parse(pair.into_inner().next().unwrap());
                AlterTableAction::DropColumn(name)
            }
            rule => panic!("Unexpected rule: {:?}", rule),
        };

        Self {
            schema_table,
            action,
        }
    }
}

impl Parser for ColumnConstraint {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析约束名称（可选）
        let (name, pair) = match pair.as_rule() {
            Rule::ident => (Some(String::parse(pair)), inner.next().unwrap()),
            _ => (None, pair),
        };

        let ty = match pair.as_rule() {
            Rule::column_constraint1 => {
                let mut inner = pair.into_inner();
                let pair = inner.next();

                // 解析排序方式（可选）
                let (order, pair) = match pair {
                    Some(pair) if matches!(pair.as_rule(), Rule::asc | Rule::desc) => {
                        (pair.as_rule() == Rule::asc, inner.next())
                    }
                    _ => (true, pair),
                };

                // 解析自动递增（可选）
                let auto_inc = pair.is_some();

                ColumnConstraintType::PrimaryKey {
                    asc: order,
                    auto_inc,
                }
            }
            Rule::column_constraint2 => ColumnConstraintType::NotNull,
            Rule::column_constraint3 => ColumnConstraintType::Unique,
            Rule::column_constraint4 => {
                let expr_pair = pair.into_inner().next().unwrap();
                ColumnConstraintType::Check(Expr::parse(expr_pair))
            }
            Rule::column_constraint5 => {
                let literal_pair = pair.into_inner().next().unwrap();
                ColumnConstraintType::Default(Literal::parse(literal_pair))
            }
            rule => panic!("Unexpected rule: {:?}", rule),
        };

        Self { name, ty }
    }
}

impl Parser for ColumnDef {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析列名
        let col_name = String::parse(pair);
        let pair = inner.next();

        // 解析列类型（可选）
        let (col_type, pair) = match pair {
            Some(p) if p.as_rule() == Rule::type_name => (Some(TypeName::parse(p)), inner.next()),
            _ => (None, pair),
        };

        // 解析列级约束
        let constraints = pair.map_or(vec![], |p| {
            p.into_inner().map(|p| ColumnConstraint::parse(p)).collect()
        });

        Self {
            col_name,
            col_type,
            constraints,
        }
    }
}

impl Parser for TableConstraint {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析约束名称（可选）
        let (name, pair) = match pair.as_rule() {
            Rule::ident => (Some(String::parse(pair)), inner.next().unwrap()),
            _ => (None, pair),
        };

        // 解析约束类型和列
        let ty = match pair.as_rule() {
            Rule::table_constraint1 => TableConstraintType::PrimaryKey,
            Rule::table_constraint2 => TableConstraintType::Unique,
            rule => panic!("Unexpected rule: {:?}", rule),
        };

        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();
        let cols = pair.into_inner().map(|p| IndexedColumn::parse(p)).collect();

        Self { name, cols, ty }
    }
}

impl Parser for IndexedColumn {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = inner.next().unwrap().as_str().to_owned();
        let asc = inner.next().map_or(true, |p| p.as_rule() == Rule::asc);

        Self { name, asc }
    }
}

impl Parser for TypeName {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let type_part = inner.next().unwrap();

        let ty = match type_part.as_rule() {
            Rule::type_decimal => TypeDef::Decimal,
            Rule::type_double => TypeDef::Double,
            Rule::type_int => TypeDef::Integer,
            Rule::type_string => TypeDef::String,
            Rule::type_varchar => TypeDef::Varchar,
            rule => panic!("Unexpected rule: {:?}", rule),
        };

        let size = match (inner.next(), inner.next()) {
            (Some(first), Some(second)) => Some(TypeSize::TypeSize(
                first.as_str().to_owned(),
                second.as_str().to_owned(),
            )),
            (Some(first), None) => Some(TypeSize::MaxSize(first.as_str().to_owned())),
            _ => None,
        };
        Self { ty, size }
    }
}

impl Parser for CreateView {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析是否为临时视图（可选）
        let (temp, pair) = match pair.as_rule() {
            Rule::temp => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析 if not exists（可选）
        let (if_not_exists, pair) = match pair.as_rule() {
            Rule::if_not_exists => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析模式名.视图名
        let schema_view = SchemaObject::parse(pair);
        let pair = inner.next().unwrap();

        // 解析视图列（可选）
        let (columns, pair) = match pair.as_rule() {
            Rule::idents => {
                let cols: Vec<_> = pair.into_inner().map(|p| String::parse(p)).collect();
                (cols, inner.next().unwrap())
            }
            _ => (vec![], pair),
        };

        // 解析 SELECT 语句
        let select = Select::parse(pair);

        Self {
            temp,
            if_not_exists,
            schema_view,
            columns,
            select,
        }
    }
}

impl Parser for DropView {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析 if exists
        let (if_exists, pair) = match pair.as_rule() {
            Rule::if_exists => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析模式名.视图名
        let schema_view = SchemaObject::parse(pair);

        Self {
            if_exists,
            schema_view,
        }
    }
}

impl Parser for CreateIndex {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析是否为唯一索引
        let (unique, pair) = match pair.as_rule() {
            Rule::unique => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析 if not exists
        let (if_not_exists, pair) = match pair.as_rule() {
            Rule::if_not_exists => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析模式名.索引名
        let schema_index = SchemaObject::parse(pair);
        let pair = inner.next().unwrap();

        // 解析表名
        let table_name = pair.as_str().to_owned();
        let pair = inner.next().unwrap();

        // 解析索引列
        let indexed_cols: Vec<_> = pair.into_inner().map(|p| IndexedColumn::parse(p)).collect();
        let pair = inner.next();

        // 解析 WHERE 子句（可选）
        let where_cond = pair.map(|p| Expr::parse(p));

        Self {
            unique,
            if_not_exists,
            schema_index,
            table_name,
            indexed_cols,
            where_cond,
        }
    }
}

impl Parser for DropIndex {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析 if exists
        let (if_exists, pair) = match pair.as_rule() {
            Rule::if_exists => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析模式名.视图名
        let schema_index = SchemaObject::parse(pair);

        Self {
            if_exists,
            schema_index,
        }
    }
}

impl Parser for CreateTableBody {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        match pair.as_rule() {
            Rule::select => {
                let select = Select::parse(pair);
                Self::Select(select)
            }
            Rule::column_defs => {
                // 解析列定义
                let columns = pair
                    .into_inner()
                    .map(|p| ColumnDef::parse(p))
                    .collect::<Vec<_>>();
                let pair = inner.next().unwrap();

                // 解析表级约束（可选）
                let table_constraints = pair
                    .into_inner()
                    .map(|p| TableConstraint::parse(p))
                    .collect();

                // 解析表选项（可选）
                let table_options: Vec<_> = inner
                    .map(|p| match p.as_rule() {
                        Rule::without_rowid => TableOption::WithoutRowid,
                        Rule::strict => TableOption::Strict,
                        rule => unreachable!("Unexpected rule: {:?}", rule),
                    })
                    .collect();

                Self::Columns {
                    columns,
                    table_constraints,
                    table_options,
                }
            }
            rule => unreachable!("Unexpected rule: {:?}", rule),
        }
    }
}

impl Parser for CreateTrigger {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析是否为临时触发器（可选）
        let (temp, pair) = match pair.as_rule() {
            Rule::temp => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析是否存在则不创建（可选）
        let (if_not_exists, pair) = match pair.as_rule() {
            Rule::if_not_exists => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析模式名.触发器名
        let schema_trigger = SchemaObject::parse(pair);
        let pair = inner.next().unwrap();

        // 解析触发器时机
        let timing = match pair.as_rule() {
            Rule::before => TriggerTiming::Before,
            Rule::after => TriggerTiming::After,
            Rule::instead => TriggerTiming::InsteadOf,
            rule => panic!("Unexpected rule: {:?}", rule),
        };
        let pair = inner.next().unwrap();

        // 解析触发器事件
        let event = match pair.as_rule() {
            Rule::trigger_event1 => TriggerEvent::Delete,
            Rule::trigger_event2 => TriggerEvent::Insert,
            Rule::trigger_event3 => {
                let mut inner = pair.into_inner();
                let cols: Vec<_> = inner.next().map_or(vec![], |p| {
                    p.into_inner().map(|p| String::parse(p)).collect()
                });
                TriggerEvent::Update(cols)
            }
            rule => panic!("Unexpected rule: {:?}", rule),
        };
        let pair = inner.next().unwrap();

        // 解析表名
        let table_name = String::parse(pair);
        let pair = inner.next().unwrap();

        // 解析 WHEN 条件（可选）
        let (when_cond, pair) = match pair.as_rule() {
            Rule::when_clause => {
                let expr_pair = pair.into_inner().next().unwrap();
                (Some(Expr::parse(expr_pair)), inner.next().unwrap())
            }
            _ => (None, pair),
        };

        // 解析触发器语句
        let mut statements = vec![];
        let first = match pair.as_rule() {
            Rule::select => Dml::Select(Select::parse(pair)),
            Rule::insert => Dml::Insert(Insert::parse(pair)),
            Rule::update => Dml::Update(Update::parse(pair)),
            Rule::delete => Dml::Delete(Delete::parse(pair)),
            _ => panic!("Unexpected rule: {:?}", pair),
        };
        statements.push(first);

        for p in inner {
            let stmt = match p.as_rule() {
                Rule::select => Dml::Select(Select::parse(p)),
                Rule::insert => Dml::Insert(Insert::parse(p)),
                Rule::update => Dml::Update(Update::parse(p)),
                Rule::delete => Dml::Delete(Delete::parse(p)),
                _ => panic!("Unexpected rule: {:?}", p),
            };
            statements.push(stmt);
        }

        Self {
            temp,
            if_not_exists,
            schema_trigger,
            timing,
            event,
            table_name,
            when_cond,
            statements,
        }
    }
}

impl Parser for DropTrigger {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        // 解析 if exists
        let (if_exists, pair) = match pair.as_rule() {
            Rule::if_exists => (true, inner.next().unwrap()),
            _ => (false, pair),
        };

        // 解析模式名.触发器名
        let schema_trigger = SchemaObject::parse(pair);

        Self {
            if_exists,
            schema_trigger,
        }
    }
}
