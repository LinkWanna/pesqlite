# Pesqlite

Pesqlite is a lightweight SQL parser and SQLite-style statement interpreter front end written in Rust. It is designed to be portable, easy to embed into other Rust projects, and focused on turning SQL text into a structured Abstract Syntax Tree (AST).

It uses [pest](https://pest.rs/) and Parsing Expression Grammars (PEG) to parse SQL statements, then maps the parsed output into strongly typed Rust data structures.

## Project Structure

The codebase is intentionally small and easy to navigate.

- `src/lib.rs`: Public entry point. Exposes the AST types, parser trait, and helper parsing functions.
- `src/sql.pest`: The SQL grammar definition written with `pest`.
- `src/ast/`: AST definitions grouped by statement family (e.g., `ddl.rs`, `dml.rs`, etc.).
- `src/parser/`:Parse implementations that transform `pest` parse trees into AST nodes:

## Usage

[simple-dbms](https://github.com/LinkWanna/simple_dbms) is a toy database management system that uses `pesqlite` as its SQL parsing layer. You can see how `pesqlite` is integrated into a larger project and how the AST is used to execute SQL statements.
