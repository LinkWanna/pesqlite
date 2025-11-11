mod ddl;
mod dml;

use crate::{BinaryOp, ConflictResolution, Expr, Literal, Rule, SchemaObject, UnaryOp};
use pest::{
    iterators::Pair,
    pratt_parser::{Assoc, Op, PrattParser},
};

pub trait Parser {
    fn parse(pair: Pair<Rule>) -> Self;
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
            .op(Op::infix(is_not, Left) | Op::infix(is, Left))
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
                    rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
                };

                Self::Binary(Box::new(lhs), op, Box::new(rhs))
            })
            .map_prefix(|op, rhs| {
                let op = match op.as_rule() {
                    Rule::negative => UnaryOp::Negative,
                    Rule::positive => UnaryOp::Positive,
                    Rule::bitwise_not => UnaryOp::BitwiseNot,
                    Rule::logical_not => UnaryOp::LogicalNot,
                    rule => unreachable!("Expr::parse expected prefix operation, found {:?}", rule),
                };
                Self::Unary(op, Box::new(rhs))
            })
            .parse(pairs)
    }
}

impl Parser for Literal {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let pair = inner.next().unwrap();

        match pair.as_rule() {
            Rule::double => Self::Double(pair.as_str().to_owned()),
            Rule::decimal => Self::Decimal(pair.as_str().to_owned()),
            Rule::integer => Self::Integer(pair.as_str().to_owned()),
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

/// 解析标识符
impl Parser for String {
    fn parse(pair: Pair<Rule>) -> Self {
        let pair = pair.into_inner().next().unwrap();

        match pair.as_rule() {
            Rule::ident_inner => pair.as_str().to_owned(),
            Rule::quoted_ident => {
                let str = pair.as_str();
                str[1..str.len() - 1].to_owned()
            }
            _ => unreachable!("Unexpected {:?}", pair),
        }
    }
}

/// 解析冲突解决策略
impl Parser for ConflictResolution {
    fn parse(pair: Pair<Rule>) -> Self {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::abort => ConflictResolution::Abort,
            Rule::fail => ConflictResolution::Fail,
            Rule::ignore => ConflictResolution::Ignore,
            Rule::replace => ConflictResolution::Replace,
            Rule::rollback => ConflictResolution::Rollback,
            rule => unreachable!("Unexpected rule: {:?}", rule),
        }
    }
}
