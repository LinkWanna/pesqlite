mod common;

use common::test_parse;
use pesqlite::{Parser, *};

test_parse!(
    test_begin,
    Rule::begin,
    Begin::parse,
    [
        ("BEGIN IMMEDIATE", Begin(TransactionMode::Immediate)),
        ("BEGIN EXCLUSIVE", Begin(TransactionMode::Exclusive)),
        ("BEGIN", Begin(TransactionMode::Deferred)),
    ]
);

test_parse!(
    test_commit,
    Rule::commit,
    Commit::parse,
    [
        ("Commit", Commit),
        ("End", Commit),
        ("Commit Transaction", Commit),
    ]
);

test_parse!(
    test_rollback,
    Rule::rollback,
    Rollback::parse,
    [
        (
            "ROLLBACK TO savepoint1",
            Rollback(Some("savepoint1".to_owned()))
        ),
        ("ROLLBACK", Rollback(None)),
        (
            "ROLLBACK TRANSACTION TO \"Savepoint2\"",
            Rollback(Some("Savepoint2".to_owned()))
        ),
    ]
);

test_parse!(
    test_savepoint,
    Rule::savepoint,
    Savepoint::parse,
    [
        ("SAVEPOINT sp1", Savepoint("sp1".to_owned())),
        (
            "SAVEPOINT \"MySavepoint\"",
            Savepoint("MySavepoint".to_owned())
        ),
    ]
);

test_parse!(
    test_release,
    Rule::release,
    Release::parse,
    [
        ("RELEASE sp1", Release("sp1".to_owned())),
        ("RELEASE \"MySavepoint\"", Release("MySavepoint".to_owned())),
    ]
);
