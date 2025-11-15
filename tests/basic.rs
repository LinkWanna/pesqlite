mod common;

use common::test_parse;
use pesqlite::*;

test_parse!(
    test_literal_double,
    Rule::literal,
    Literal::parse,
    [
        ("123e-1", Literal::Double("123e-1".to_owned())),
        ("123e+2", Literal::Double("123e+2".to_owned())),
        ("123.1e+2", Literal::Double("123.1e+2".to_owned())),
    ]
);

test_parse!(
    test_literal_decimal,
    Rule::literal,
    Literal::parse,
    [
        ("123.", Literal::Decimal("123.".to_owned())),
        ("123.12", Literal::Decimal("123.12".to_owned())),
        (".5", Literal::Decimal(".5".to_owned())),
    ]
);

test_parse!(
    test_literal_integer,
    Rule::literal,
    Literal::parse,
    [
        ("123", Literal::Integer("123".to_owned())),
        ("+123", Literal::Integer("+123".to_owned())),
        ("-123", Literal::Integer("-123".to_owned())),
    ]
);

test_parse!(
    test_literal_string,
    Rule::literal,
    Literal::parse,
    [
        ("'hello'", Literal::String("hello".to_owned())),
        ("''", Literal::String("".to_owned())),
    ]
);

test_parse!(
    test_literal_blob,
    Rule::literal,
    Literal::parse,
    [
        ("x'010D'", Literal::Blob("010D".to_owned())),
        ("X'010D'", Literal::Blob("010D".to_owned())),
        ("x''", Literal::Blob("".to_owned())),
    ]
);

test_parse!(
    test_expr_accuracy,
    Rule::expr,
    Expr::parse,
    [
        ("a", Expr::QualifiedColumn(None, None, "a".to_owned())),
        (
            "\"a\" and b",
            Expr::Binary(
                Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                BinaryOp::LogicalAnd,
                Box::new(Expr::QualifiedColumn(None, None, "b".to_owned())),
            )
        ),
        (
            "a >= b",
            Expr::Binary(
                Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                BinaryOp::Ge,
                Box::new(Expr::QualifiedColumn(None, None, "b".to_owned())),
            )
        ),
        (
            "1 + 2 * 3",
            Expr::Binary(
                Box::new(Expr::Literal(Literal::Integer("1".to_owned()))),
                BinaryOp::Plus,
                Box::new(Expr::Binary(
                    Box::new(Expr::Literal(Literal::Integer("2".to_owned()))),
                    BinaryOp::Mul,
                    Box::new(Expr::Literal(Literal::Integer("3".to_owned())))
                ))
            )
        ),
        (
            "a IS NOT NULL",
            Expr::Binary(
                Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                BinaryOp::IsNot,
                Box::new(Expr::Literal(Literal::Null)),
            )
        ),
        (
            "NOT a",
            Expr::Unary(
                UnaryOp::LogicalNot,
                Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
            )
        ),
        (
            "TRUE AND FALSE",
            Expr::Binary(
                Box::new(Expr::Literal(Literal::Bool(true))),
                BinaryOp::LogicalAnd,
                Box::new(Expr::Literal(Literal::Bool(false))),
            )
        ),
    ]
);

#[test]
fn test_expr_samples() {
    use pest::Parser;

    let samples = [
        "1 + 2",
        "1 - 2",
        "1 * 2",
        "1 / 2",
        "1 % 2",
        "1 + 2 * 3",
        "(1 + 2) * 3",
        "1 + (2 * 3)",
        "1 + 2 * 3 - 4 / 5",
        "a + b",
        "a * (b + c)",
        "a / b - c * d",
        "a + b + c + d",
        "a * b * c * d",
        "a + b * c - d / e",
        "a > b",
        "a < b",
        "a >= b",
        "a <= b",
        "a = b",
        "a != b",
        "a AND b",
        "a OR b",
        "NOT a",
        "a IS NULL",
        "a IS NOT NULL",
        "a || b",
        "a + -b",
        "-a + b",
        "a + (b * (c - d))",
        "((a + b) * c) / d",
        "1.23 + 4.56",
        "'hello' || 'world'",
        "TRUE AND FALSE",
        "a + b * c / d - e % f",
        "a IS TRUE",
        "a IS NOT FALSE",
    ];

    for sample in samples {
        let pairs = SqlParser::parse(Rule::expr, sample).unwrap();
        // 确保消费了所有的输入
        let span = pairs.peek().unwrap().as_span();
        assert_eq!(
            span.end(),
            sample.len(),
            "Input '{}' was not fully consumed during parsing",
            sample
        );
    }
}
