mod common;

use common::test_parse;
use pesqlite::*;

test_parse!(
    test_literal_float,
    Rule::literal,
    Literal::parse,
    [
        ("123e-1", Literal::Float("123e-1".to_owned())),
        ("123e+2", Literal::Float("123e+2".to_owned())),
        ("123.1e+2", Literal::Float("123.1e+2".to_owned())),
        ("123.", Literal::Float("123.".to_owned())),
        ("123.12", Literal::Float("123.12".to_owned())),
        (".5", Literal::Float(".5".to_owned())),
    ]
);

test_parse!(
    test_literal_integer,
    Rule::literal,
    Literal::parse,
    [
        ("123", Literal::Integer("123".to_owned())),
        ("0x123", Literal::Integer("0x123".to_owned())),
        ("0X123", Literal::Integer("0X123".to_owned())),
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
    test_literal_time,
    Rule::literal,
    Literal::parse,
    [
        ("Current_time", Literal::CurrentTime),
        ("Current_date", Literal::CurrentDate),
        ("Current_timestamp", Literal::CurrentTimestamp),
    ]
);

test_parse!(
    test_expr_accuracy,
    Rule::expr,
    Expr::parse,
    [
        ("a", Expr::QualifiedColumn(None, None, "a".to_owned())),
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
            "\"a\" and b",
            Expr::Binary(
                Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                BinaryOp::LogicalAnd,
                Box::new(Expr::QualifiedColumn(None, None, "b".to_owned())),
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
        (
            "a between 1 AND 10",
            Expr::Between {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: false,
                low: Box::new(Expr::Literal(Literal::Integer("1".to_owned()))),
                high: Box::new(Expr::Literal(Literal::Integer("10".to_owned()))),
            }
        ),
        // In 表达式
        (
            "a IN (1, 2, 3)",
            Expr::In {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: false,
                list: vec![
                    Expr::Literal(Literal::Integer("1".to_owned())),
                    Expr::Literal(Literal::Integer("2".to_owned())),
                    Expr::Literal(Literal::Integer("3".to_owned())),
                ],
            }
        ),
        (
            "a NOT IN (1, 2, 3)",
            Expr::In {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: true,
                list: vec![
                    Expr::Literal(Literal::Integer("1".to_owned())),
                    Expr::Literal(Literal::Integer("2".to_owned())),
                    Expr::Literal(Literal::Integer("3".to_owned())),
                ],
            }
        ),
        // Match 表达式
        (
            "a MATCH 'abc'",
            Expr::Match {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: false,
                pattern: Box::new(Expr::Literal(Literal::String("abc".to_owned()))),
            }
        ),
        (
            "a NOT MATCH 'abc'",
            Expr::Match {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: true,
                pattern: Box::new(Expr::Literal(Literal::String("abc".to_owned()))),
            }
        ),
        // Like 表达式
        (
            "a LIKE 'abc%'",
            Expr::Like {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: false,
                pattern: Box::new(Expr::Literal(Literal::String("abc%".to_owned()))),
                escape: None,
            }
        ),
        (
            "a NOT LIKE 'abc%'",
            Expr::Like {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: true,
                pattern: Box::new(Expr::Literal(Literal::String("abc%".to_owned()))),
                escape: None,
            }
        ),
        (
            "a LIKE 'abc%' ESCAPE '\\'",
            Expr::Like {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: false,
                pattern: Box::new(Expr::Literal(Literal::String("abc%".to_owned()))),
                escape: Some(Box::new(Expr::Literal(Literal::String("\\".to_owned())))),
            }
        ),
        // Regexp 表达式
        (
            "a REGEXP 'abc.*'",
            Expr::Regexp {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: false,
                pattern: Box::new(Expr::Literal(Literal::String("abc.*".to_owned()))),
            }
        ),
        (
            "a NOT REGEXP 'abc.*'",
            Expr::Regexp {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: true,
                pattern: Box::new(Expr::Literal(Literal::String("abc.*".to_owned()))),
            }
        ),
        // Glob 表达式
        (
            "a GLOB 'abc*'",
            Expr::Glob {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: false,
                pattern: Box::new(Expr::Literal(Literal::String("abc*".to_owned()))),
            }
        ),
        (
            "a NOT GLOB 'abc*'",
            Expr::Glob {
                expr: Box::new(Expr::QualifiedColumn(None, None, "a".to_owned())),
                not: true,
                pattern: Box::new(Expr::Literal(Literal::String("abc*".to_owned()))),
            }
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
        "a BETWEEN 1 AND 10",
        "a NOT BETWEEN 1 AND 10",
        "a IN (1, 2, 3)",
        "a NOT IN (1, 2, 3)",
        "a LIKE 'abc%'",
        "a NOT LIKE 'abc%'",
        "a LIKE 'abc%' ESCAPE '\\'",
        "a REGEXP 'abc.*'",
        "a NOT REGEXP 'abc.*'",
        "a GLOB 'abc*'",
        "a NOT GLOB 'abc*'",
        "a MATCH 'abc'",
        "a NOT MATCH 'abc'",
        "x'010D' || x'FF'",
        "123e-1 + 456.78",
        "a IS UNKNOWN",
        "a IS NOT UNKNOWN",
        "a IS FALSE",
        "a IS NOT TRUE",
        "a IS NOT NULL AND b IS NULL",
        "a AND b OR c",
        "NOT (a OR b)",
        "((a + b) * (c - d)) / (e % f)",
        "a + b * (c - d / e)",
        "a + (b * c) - (d / e)",
        "a + b * c - d / e + f % g",
        "a || b || c",
        "'foo' || 'bar' || 'baz'",
        "1 + 2 + 3 * 4 / 5 - 6 % 7",
        "((1 + 2) * (3 - 4)) / (5 % 6)",
        "a = b AND c != d OR e < f",
        "a IS NOT NULL OR b IS NULL AND c IS TRUE",
        "a + -b * +c",
        "-(a + b)",
        "NOT NOT a",
        "a + (b * (c - (d / e)))",
        "a BETWEEN b AND c AND d",
        "a LIKE 'abc%' AND b GLOB 'def*'",
        "a IS NULL OR b IS NOT NULL",
        "a IS TRUE AND b IS FALSE",
        "a = b OR c = d AND e = f",
        "a + (b * c) / (d - e)",
        "a / (b + c * (d - e))",
        "a + b * (c - d / (e + f))",
        "a + b * c / d - e % f + g",
        "a IS NOT NULL AND (b IS NULL OR c IS TRUE)",
        "a IN (1, 2, 3) AND b NOT IN (4, 5, 6)",
        "a LIKE 'abc%' ESCAPE '_'",
        "a REGEXP '^abc$' OR b GLOB '*def*'",
        "a MATCH 'pattern' AND b NOT MATCH 'other'",
        "a || b || c || d",
        "TRUE OR FALSE AND TRUE",
        "NOT TRUE OR FALSE",
        "a + b * c / d - e % f + g - h",
        "a IS NULL AND b IS NOT NULL OR c IS TRUE",
        "a BETWEEN 1 AND 10 OR b IN (2, 3, 4)",
        "a LIKE 'abc%' AND b LIKE 'def%'",
        "a REGEXP 'abc.*' AND b REGEXP 'def.*'",
        "a GLOB 'abc*' OR b GLOB 'def*'",
        "a MATCH 'abc' AND b MATCH 'def'",
        "a IS TRUE OR b IS FALSE AND c IS UNKNOWN",
        "a + (b * (c - (d / (e + f))))",
        "a IN (1, 2, 3, 4, 5)",
        "a NOT IN (6, 7, 8, 9, 10)",
        "a LIKE 'a%' ESCAPE '!'",
        "a REGEXP '[a-z]+'",
        "a GLOB '?abc*'",
        "a MATCH 'abcdef'",
        "a IS NULL AND b IS NOT NULL AND c IS TRUE",
        "a + b * c / d - e % f + g - h * i / j",
        "a = b AND (c != d OR e < f)",
        "a IS NOT NULL OR (b IS NULL AND c IS TRUE)",
        "a + -b * +c - -d / +e",
        "-(a + b * (c - d / e))",
        "NOT (a OR b AND c)",
        "((a + b) * (c - d)) / ((e % f) + g)",
        "a + (b * (c - d / (e + f * g)))",
        "a + b * (c - d / (e + f))",
        "a + (b * c) - (d / e) + (f % g)",
        "a + b * c - d / e + f % g - h",
        "a || b || c || d || e",
        "'foo' || 'bar' || 'baz' || 'qux'",
        "1 + 2 + 3 * 4 / 5 - 6 % 7 + 8",
        "((1 + 2) * (3 - 4)) / ((5 % 6) + 7)",
        "a = b AND c != d OR e < f AND g > h",
        "a IS NOT NULL OR b IS NULL AND c IS TRUE OR d IS FALSE",
        "a + -b * +c - -d / +e + f",
        "-(a + b * (c - d / e) + f)",
        "NOT (a OR b AND c OR d)",
        "((a + b) * (c - d)) / ((e % f) + g - h)",
        "a + (b * (c - d / (e + f * g) - h))",
        "a + b * (c - d / (e + f) + g)",
        "a + (b * c) - (d / e) + (f % g) - h",
        "a + b * c - d / e + f % g - h + i",
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
