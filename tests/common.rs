#![allow(unused_macros)]
macro_rules! test_parse {
    ($name:ident, $rule:expr, $parser:path, $cases: expr) => {
        #[test]
        fn $name() {
            use pest::Parser;
            for (input, expected) in $cases {
                let pair = match SqlParser::parse($rule, input) {
                    Ok(mut pairs) => pairs.next().unwrap(),
                    Err(e) => panic!("Failed to parse input '{}': {}", input, e),
                };
                let ast = $parser(pair);
                assert_eq!(ast, expected);
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use test_parse;
