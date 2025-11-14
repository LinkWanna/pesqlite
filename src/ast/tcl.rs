//! 事务控制语言（Transaction Control Language, TCL）相关的抽象语法树定义

/// 开启事务语句
#[derive(Clone, Debug, PartialEq)]
pub struct Begin(pub TransactionMode);

/// 提交事务语句
#[derive(Clone, Debug, PartialEq)]
pub struct Commit;

/// 回滚事务语句
#[derive(Clone, Debug, PartialEq)]
pub struct Rollback(pub Option<String>);

/// 保存点语句
#[derive(Clone, Debug, PartialEq)]
pub struct Savepoint(pub String);

/// 释放保存点语句
#[derive(Clone, Debug, PartialEq)]
pub struct Release(pub String);

/// 事务类型
#[derive(Clone, Debug, PartialEq)]
pub enum TransactionMode {
    Deferred, // 默认
    Immediate,
    Exclusive,
}
