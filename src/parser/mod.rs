mod ddl;
mod dml;
mod tcl;

use crate::{Rule, ast::*};
use pest::{
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
};

pub trait Parser {
    fn parse(pair: Pair<Rule>) -> Self;
}

impl Parser for Stmt {
    fn parse(pair: Pair<Rule>) -> Self {
        let pair = pair.into_inner().next().unwrap();

        match pair.as_rule() {
            Rule::select => Stmt::Select(Select::parse(pair)),
            Rule::insert => Stmt::Insert(Insert::parse(pair)),
            Rule::update => Stmt::Update(Update::parse(pair)),
            Rule::delete => Stmt::Delete(Delete::parse(pair)),
            Rule::create_table => Stmt::CreateTable(CreateTable::parse(pair)),
            Rule::create_index => Stmt::CreateIndex(CreateIndex::parse(pair)),
            Rule::create_view => Stmt::CreateView(CreateView::parse(pair)),
            Rule::create_trigger => Stmt::CreateTrigger(CreateTrigger::parse(pair)),
            Rule::alter_table => Stmt::AlterTable(AlterTable::parse(pair)),
            Rule::drop_table => Stmt::DropTable(DropTable::parse(pair)),
            Rule::drop_index => Stmt::DropIndex(DropIndex::parse(pair)),
            Rule::drop_view => Stmt::DropView(DropView::parse(pair)),
            Rule::drop_trigger => Stmt::DropTrigger(DropTrigger::parse(pair)),
            Rule::begin => Stmt::Begin(Begin::parse(pair)),
            Rule::commit => Stmt::Commit(Commit::parse(pair)),
            Rule::rollback => Stmt::Rollback(Rollback::parse(pair)),
            Rule::savepoint => Stmt::Savepoint(Savepoint::parse(pair)),
            Rule::release => Stmt::Release(Release::parse(pair)),
            _ => unreachable!("Unexpected statement rule: {:?}", pair.as_rule()),
        }
    }
}

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use Rule::*;
        use Assoc::*;

        // 运算符优先级从低到高定义
        PrattParser::new()
            // OR
            .op(Op::infix(logical_or, Left))
            // AND
            .op(Op::infix(logical_and, Left))
            // NOT
            .op(Op::prefix(logical_not))
            // IS, IS NOT
            .op(
                Op::infix(is_not, Left) | Op::infix(is, Left) |
                Op::postfix(between) | Op::postfix(in_op) | Op::postfix(match_op) | Op::postfix(like) | Op::postfix(regexp) | Op::postfix(glob) |
                Op::postfix(is_null) |  Op::postfix(not_null)
            )
            // =, !=
            .op(Op::infix(eq, Left) | Op::infix(ne, Left))
            // <, <=, >, >=
            .op(Op::infix(lt, Left) | Op::infix(le, Left) | Op::infix(gt, Left) | Op::infix(ge, Left))
            // &, |, >>, <<
            .op(Op::infix(bitwise_and, Left)| Op::infix(bitwise_or, Left) | Op::infix(right_shift, Left) | Op::infix(left_shift, Left))
            // +, -
            .op(Op::infix(plus, Left) | Op::infix(minus, Left))
            // *, /, %
            .op(Op::infix(mul, Left) | Op::infix(div, Left) | Op::infix(r#mod, Left))
            // ||
            .op(Op::infix(concat, Left))
            // +, -, ~
            .op(Op::prefix(bitwise_not) | Op::prefix(positive) | Op::prefix(negative))
    };
}

impl Parser for Expr {
    fn parse(pair: Pair<Rule>) -> Self {
        let pairs = pair.into_inner();

        PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::exprs => Self::ExprList(primary.into_inner().map(Self::parse).collect()),
                Rule::literal => Self::Literal(Literal::parse(primary)),
                Rule::qualified_column => {
                    let mut inner = primary.into_inner();

                    // 解析模式名和表名
                    let (schema_name, table_name) = match inner.len() {
                        3 => {
                            let schema_name = String::parse(inner.next().unwrap());
                            let table_name = String::parse(inner.next().unwrap());
                            (Some(schema_name), Some(table_name))
                        }
                        2 => {
                            let table_name = String::parse(inner.next().unwrap());
                            (None, Some(table_name))
                        }
                        _ => (None, None),
                    };

                    // 解析列名
                    let column_name = String::parse(inner.next().unwrap());

                    Self::QualifiedColumn(schema_name, table_name, column_name)
                }
                _ => unreachable!("Unexpected Rule: {:?}", primary),
            })
            .map_infix(|lhs, op, rhs| {
                let op = match op.as_rule() {
                    Rule::concat => BinaryOp::Concat,
                    Rule::mul => BinaryOp::Mul,
                    Rule::div => BinaryOp::Div,
                    Rule::r#mod => BinaryOp::Mod,
                    Rule::plus => BinaryOp::Plus,
                    Rule::minus => BinaryOp::Minus,
                    Rule::bitwise_and => BinaryOp::BitwiseAnd,
                    Rule::bitwise_or => BinaryOp::BitwiseOr,
                    Rule::left_shift => BinaryOp::LeftShift,
                    Rule::right_shift => BinaryOp::RightShift,
                    Rule::lt => BinaryOp::Lt,
                    Rule::le => BinaryOp::Le,
                    Rule::gt => BinaryOp::Gt,
                    Rule::ge => BinaryOp::Ge,
                    Rule::eq => BinaryOp::Eq,
                    Rule::ne => BinaryOp::Ne,
                    Rule::is => BinaryOp::Is,
                    Rule::is_not => BinaryOp::IsNot,
                    Rule::logical_and => BinaryOp::LogicalAnd,
                    Rule::logical_or => BinaryOp::LogicalOr,
                    rule => unreachable!("Expr::parse expected infix oprator, found {:?}", rule),
                };

                Self::Binary(Box::new(lhs), op, Box::new(rhs))
            })
            .map_prefix(|op, rhs| {
                let op = match op.as_rule() {
                    Rule::negative => UnaryOp::Negative,
                    Rule::positive => UnaryOp::Positive,
                    Rule::bitwise_not => UnaryOp::BitwiseNot,
                    Rule::logical_not => UnaryOp::LogicalNot,
                    rule => unreachable!("Expr::parse expected prefix oprator, found {:?}", rule),
                };
                Self::Unary(op, Box::new(rhs))
            })
            .map_postfix(|lhs, op| {
                let rule = match op.as_rule() {
                    Rule::is_null => return Self::NullJudge(Box::new(lhs), true),
                    Rule::not_null => return Self::NullJudge(Box::new(lhs), false),
                    rule => rule,
                };

                let mut inner = op.into_inner();
                let pair = inner.next().unwrap();

                // 解析 not（可选）
                let (not, pair) = match pair.as_rule() {
                    Rule::logical_not => (true, inner.next().unwrap()),
                    _ => (false, pair),
                };

                match rule {
                    Rule::between => {
                        let low = Expr::parse(pair);
                        let high = Expr::parse(inner.next().unwrap());
                        Self::Between {
                            expr: Box::new(lhs),
                            not,
                            low: Box::new(low),
                            high: Box::new(high),
                        }
                    }
                    Rule::in_op => {
                        // 解析表达式列表
                        let list = pair.into_inner().map(Expr::parse).collect::<Vec<Expr>>();
                        Self::In {
                            expr: Box::new(lhs),
                            not,
                            list,
                        }
                    }
                    Rule::match_op => {
                        let pattern = Expr::parse(pair);
                        Self::Match {
                            expr: Box::new(lhs),
                            not,
                            pattern: Box::new(pattern),
                        }
                    }
                    Rule::like => {
                        let pattern = Expr::parse(pair);

                        // 解析 escape（可选）
                        let escape = inner.next().map(Expr::parse).map(Box::new);
                        Self::Like {
                            expr: Box::new(lhs),
                            not,
                            pattern: Box::new(pattern),
                            escape,
                        }
                    }
                    Rule::regexp => {
                        let pattern = Expr::parse(pair);
                        Self::Regexp {
                            expr: Box::new(lhs),
                            not,
                            pattern: Box::new(pattern),
                        }
                    }
                    Rule::glob => {
                        let pattern = Expr::parse(pair);
                        Self::Glob {
                            expr: Box::new(lhs),
                            not,
                            pattern: Box::new(pattern),
                        }
                    }
                    rule => unreachable!("Expr::parse expected postfix oprator, found {:?}", rule),
                }
            })
            .parse(pairs)
    }
}

impl Parser for Literal {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        match pair.as_rule() {
            Rule::numeric => {
                // 检查是否有 '.' 或 'e'/'E' 来判断是整数还是实数
                let str = pair.as_str();
                let bytes = str.as_bytes();

                if bytes.len() >= 2 && matches!(bytes[1], b'x' | b'X') {
                    Self::Integer(u64::from_str_radix(&str[2..], 16).unwrap())
                } else if bytes.iter().any(|b| matches!(b, b'.' | b'e' | b'E')) {
                    Self::Float(str.parse::<f64>().unwrap())
                } else {
                    Self::Integer(u64::from_str_radix(str, 10).unwrap())
                }
            }
            Rule::string => {
                let str = pair.as_str();
                Self::String(str[1..str.len() - 1].to_owned())
            }
            Rule::blob => {
                let str = pair.as_str();
                Self::Blob(str[2..str.len() - 1].to_owned())
            }
            Rule::null => Self::Null,
            Rule::r#true => Self::Bool(true),
            Rule::r#false => Self::Bool(false),
            Rule::current_time => Self::CurrentTime,
            Rule::current_date => Self::CurrentDate,
            Rule::current_timestamp => Self::CurrentTimestamp,
            rule => panic!("Unexpected rule: {:?}", rule),
        }
    }
}

impl Parser for SchemaObject {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();

        // 解析模式名（可选）
        let (schema_name, name) = match (inner.next(), inner.next()) {
            (Some(schema_pair), Some(name_pair)) => {
                let schema_name = String::parse(schema_pair);
                let name = String::parse(name_pair);
                (Some(schema_name), name)
            }
            (Some(name_pair), None) => {
                let name = String::parse(name_pair);
                (None, name)
            }
            _ => unreachable!("Unexpected schema object format"),
        };

        Self { schema_name, name }
    }
}

impl Parser for String {
    fn parse(pair: Pair<Rule>) -> Self {
        let pair = pair.into_inner().next().unwrap();

        match pair.as_rule() {
            Rule::ident_bare => pair.as_str().to_lowercase(),
            Rule::quoted_ident => {
                let str = pair.as_str();
                str[1..str.len() - 1].to_owned()
            }
            _ => unreachable!("Unexpected {:?}", pair),
        }
    }
}

impl Parser for ConflictResolution {
    fn parse(pair: Pair<Rule>) -> Self {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::abort => ConflictResolution::Abort,
            Rule::fail => ConflictResolution::Fail,
            Rule::ignore => ConflictResolution::Ignore,
            Rule::replace => ConflictResolution::Replace,
            Rule::rollback_kw => ConflictResolution::Rollback,
            rule => unreachable!("Unexpected rule: {:?}", rule),
        }
    }
}
