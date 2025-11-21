mod ddl;
mod dml;
mod tcl;

pub use crate::ast::ddl::*;
pub use crate::ast::dml::*;
pub use crate::ast::tcl::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    // 数据操作语言（DML）
    Select(Select),
    Insert(Insert),
    Update(Update),
    Delete(Delete),

    // 数据定义语言（DDL）
    CreateTable(CreateTable),
    CreateIndex(CreateIndex),
    CreateView(CreateView),
    CreateTrigger(CreateTrigger),
    AlterTable(AlterTable),
    DropTable(DropTable),
    DropIndex(DropIndex),
    DropView(DropView),
    DropTrigger(DropTrigger),
}

/// 字面量
#[derive(Clone, Debug, PartialEq)]
pub enum Literal {
    Integer(u64),     // 整数字面量
    Float(f64),       // 浮点数字面量
    String(String),   // 字符串字面量
    Blob(String),     // 二进制字面量
    Null,             // NULL
    Bool(bool),       // 布尔值
    CurrentTime,      // CURRENT_TIME
    CurrentDate,      // CURRENT_DATE
    CurrentTimestamp, // CURRENT_TIMESTAMP
}

/// 表达式
#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Literal(Literal),                                        // 字面量
    ExprList(Vec<Expr>),                                     // 表达式列表
    Unary(UnaryOp, Box<Expr>),                               // 一元运算
    Binary(Box<Expr>, BinaryOp, Box<Expr>),                  // 二元运算
    QualifiedColumn(Option<String>, Option<String>, String), // 限定名称
    NullJudge(Box<Expr>, bool),
    Between {
        expr: Box<Expr>,
        not: bool,
        low: Box<Expr>,
        high: Box<Expr>,
    }, // BETWEEN 表达式
    In {
        expr: Box<Expr>,
        not: bool,
        list: Vec<Expr>,
    }, // IN 列表表达式
    Match {
        expr: Box<Expr>,
        not: bool,
        pattern: Box<Expr>,
    }, // MATCH 表达式
    Like {
        expr: Box<Expr>,
        not: bool,
        pattern: Box<Expr>,
        escape: Option<Box<Expr>>,
    }, // LIKE 表达式
    Regexp {
        expr: Box<Expr>,
        not: bool,
        pattern: Box<Expr>,
    }, // REGEXP 表达式
    Glob {
        expr: Box<Expr>,
        not: bool,
        pattern: Box<Expr>,
    }, // GLOB 表达式}
}

/// 二元运算符
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BinaryOp {
    // level 0
    Concat, // ||

    // level 1
    Mul, // *
    Div, // /
    Mod, // %

    // level 2
    Plus,  // +
    Minus, // -

    // level 3
    BitwiseAnd, // &
    BitwiseOr,  // |
    RightShift, // >>
    LeftShift,  // <<

    // level 4
    Lt, // <
    Le, // <=
    Gt, // >
    Ge, // >=

    // level 5
    Eq,    // = or ==
    Ne,    // != or <>
    Is,    // IS
    IsNot, // IS NOT

    // level 6
    LogicalAnd, // AND
    LogicalOr,  // OR
}

/// 一元运算符
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum UnaryOp {
    BitwiseNot, // ~
    Positive,   // +
    Negative,   // -
    LogicalNot, // NOT
}

/// 模式名.对象名
#[derive(Clone, Debug, PartialEq)]
pub struct SchemaObject {
    pub schema_name: Option<String>,
    pub name: String,
}

/// 冲突解决策略（默认 Abort）
#[derive(Clone, Debug, PartialEq)]
pub enum ConflictResolution {
    Abort,
    Fail,
    Ignore,
    Replace,
    Rollback,
}
