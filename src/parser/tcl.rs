use crate::{Rule, ast::*, parser::Parser};
use pest::iterators::Pair;

impl Parser for Begin {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let mode = inner
            .next()
            .map(|mode_pair| match mode_pair.as_rule() {
                Rule::deferred => TransactionMode::Deferred,
                Rule::immediate => TransactionMode::Immediate,
                Rule::exclusive => TransactionMode::Exclusive,
                _ => unreachable!("Unexpected transaction mode: {:?}", mode_pair),
            })
            .unwrap_or(TransactionMode::Deferred); // 默认模式

        Self(mode)
    }
}

impl Parser for Commit {
    fn parse(_: Pair<Rule>) -> Self {
        Self
    }
}

impl Parser for Rollback {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let savepoint_name = inner.next().map(|p| String::parse(p));
        Self(savepoint_name)
    }
}

impl Parser for Savepoint {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = String::parse(inner.next().unwrap());
        Self(name)
    }
}

impl Parser for Release {
    fn parse(pair: Pair<Rule>) -> Self {
        let mut inner = pair.into_inner();
        let name = String::parse(inner.next().unwrap());
        Self(name)
    }
}
