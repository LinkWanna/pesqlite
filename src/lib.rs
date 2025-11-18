mod ast;
mod parser;

pub use crate::ast::*;
pub use crate::parser::Parser;
use pest::Parser as PestParser;

#[derive(pest_derive::Parser)]
#[grammar = "sql.pest"]
pub struct SqlParser;

/// Parse a SQL statement into an AST.
pub fn parse_stmts(input: &str) -> Result<Vec<Stmt>, pest::error::Error<Rule>> {
    let pairs = SqlParser::parse(Rule::stmts_end, input)?;
    Ok(pairs.map(|p| Stmt::parse(p)).collect())
}

pub fn parse_stmt(input: &str) -> Result<Stmt, pest::error::Error<Rule>> {
    let pairs = SqlParser::parse(Rule::stmt_end, input)?;
    let pair = pairs.into_iter().next().unwrap();
    Ok(Stmt::parse(pair))
}

pub fn parse_create_table(input: &str) -> Result<CreateTable, pest::error::Error<Rule>> {
    let pairs = SqlParser::parse(Rule::create_table_end, input)?;
    let pair = pairs.into_iter().next().unwrap();
    Ok(CreateTable::parse(pair))
}

pub fn parse_create_index(input: &str) -> Result<CreateIndex, pest::error::Error<Rule>> {
    let pairs = SqlParser::parse(Rule::create_index_end, input)?;
    let pair = pairs.into_iter().next().unwrap();
    Ok(CreateIndex::parse(pair))
}

pub fn parse_create_view(input: &str) -> Result<CreateView, pest::error::Error<Rule>> {
    let pairs = SqlParser::parse(Rule::create_view_end, input)?;
    let pair = pairs.into_iter().next().unwrap();
    Ok(CreateView::parse(pair))
}

pub fn parse_create_trigger(input: &str) -> Result<CreateView, pest::error::Error<Rule>> {
    let pairs = SqlParser::parse(Rule::create_trigger_end, input)?;
    let pair = pairs.into_iter().next().unwrap();
    Ok(CreateView::parse(pair))
}
