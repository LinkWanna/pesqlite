mod common;
use pesqlite::*;

#[test]
fn test_sql_samples() {
    use pest::Parser;

    let samples = [
        "CREATE TABLE users (id INTEGER PRIMARY KEY, name VARCHAR(255));",
        "INSERT INTO users (id, name) VALUES (1, 'Alice');",
        "SELECT * FROM users WHERE id = 1;",
        "UPDATE users SET name = 'Bob' WHERE id = 1;",
        "DELETE FROM users WHERE id = 1;",
        "CREATE INDEX idx_name ON users (name);",
        "DROP TABLE users;",
    ];

    for sample in samples {
        let pairs = SqlParser::parse(Rule::stmt, sample).unwrap();
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
