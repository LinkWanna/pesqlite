mod ast;
mod parser;

pub use crate::ast::*;
pub use crate::parser::Parser;
use pest::Parser as PestParser;

#[derive(pest_derive::Parser)]
#[grammar = "sql.pest"]
pub struct SqlParser;

/// Parse a SQL statement into an AST.
pub fn parse_stmt(input: &str) -> Result<Vec<Stmt>, pest::error::Error<Rule>> {
    let pairs = SqlParser::parse(Rule::stmt, input)?;
    Ok(pairs.map(|p| Stmt::parse(p)).collect())
}
