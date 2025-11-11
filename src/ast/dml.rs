//! 数据操作语言（Data Manipulation Language, DML）相关的抽象语法树定义
use crate::{ConflictResolution, Expr, IndexedColumn, SchemaObject};

// DML 语句枚举
#[derive(Clone, Debug, PartialEq)]
pub enum Dml {
    Select(Select),
    Insert(Insert),
    Update(Update),
    Delete(Delete),
}

/// Select 语句
#[derive(Clone, Debug, PartialEq)]
pub struct Select {
    pub core: SelectCore,
    pub compound: Vec<(CompoundOperator, SelectCore)>,
    pub order_by: Vec<OrderingTerm>,
    pub limit: Option<Expr>,
    pub offset: Option<Expr>,
}

/// Insert 语句
#[derive(Clone, Debug, PartialEq)]
pub struct Insert {
    pub header: InsertHeader,
    pub schema_table: SchemaObject,
    pub alias: Option<String>,
    pub columns: Vec<String>,
    pub values: InsertValues,
    pub return_clause: Vec<ReturnSubClause>,
}

/// Update 语句
#[derive(Clone, Debug, PartialEq)]
pub struct Update {
    pub conflict: ConflictResolution,
    pub qualified_table: QualifiedTable,
    pub set_clause: Vec<SetSubClause>,
    pub from_clause: Option<FromClause>,
    pub where_clause: Option<Expr>,
    pub return_clause: Vec<ReturnSubClause>,
}

/// Delete 语句
#[derive(Clone, Debug, PartialEq)]
pub struct Delete {
    pub qualified_table: QualifiedTable,
    pub where_clause: Option<Expr>,
    pub return_clause: Vec<ReturnSubClause>,
}

/// Select 语句的核心部分
#[derive(Clone, Debug, PartialEq)]
pub enum SelectCore {
    Query {
        is_distinct: bool,
        columns: Vec<ResultColumn>,
        from_clause: Option<FromClause>,
        where_clause: Option<Expr>,
        group_by: Vec<Expr>,
        having: Option<Expr>,
    },
    Values(Vec<Vec<Expr>>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResultColumn {
    Expr(Expr, Option<String>),
    Star,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InsertValues {
    Values {
        values: Vec<Vec<Expr>>,
        upsert: Option<UpsertSubClause>,
    },
    Select {
        select: Box<Select>,
        upsert: Option<UpsertSubClause>,
    },
    Default,
}

/// 赋值语句
#[derive(Clone, Debug, PartialEq)]
pub struct SetSubClause {
    pub columns: Vec<String>,
    pub value: Expr,
}

/// Join 子句
#[derive(Clone, Debug, PartialEq)]
pub struct JoinClause {
    pub table_or_subquery: QualifiedTable,
    pub joins: Vec<JoinSubClause>,
}

/// Join 子句的子部分
#[derive(Clone, Debug, PartialEq)]
pub struct JoinSubClause {
    pub operator: JoinOperator,
    pub table_or_subquery: QualifiedTable,
    pub constraint: Option<JoinConstraint>,
}

/// 连接操作符
#[derive(Clone, Debug, PartialEq)]
pub enum JoinOperator {
    Comma,
    Cross,
    Inner(bool),
    Outer(bool, OuterJoinType),
}

/// 外连接类型
#[derive(Clone, Debug, PartialEq)]
pub enum OuterJoinType {
    Left,
    Right,
    Full,
}

#[derive(Clone, Debug, PartialEq)]
pub enum JoinConstraint {
    Expr(Expr),         // 连接条件表达式
    Using(Vec<String>), // 列名
}

/// 完整的表标识，包括模式名、表名、别名和索引信息
#[derive(Clone, Debug, PartialEq)]
pub struct QualifiedTable {
    pub schema_table: SchemaObject,
    pub alias: Option<String>,
    pub indexed: Option<Indexed>,
}

/// 索引信息
#[derive(Clone, Debug, PartialEq)]
pub enum Indexed {
    By(String),
    NotIndexed,
}

/// From 子句
#[derive(Clone, Debug, PartialEq)]
pub enum FromClause {
    TableOrQuerys(Vec<QualifiedTable>),
    Join(JoinClause),
}

/// 排序方式
#[derive(Clone, Debug, PartialEq)]
pub struct OrderingTerm {
    pub expr: Expr,
    pub asc: bool,
    pub nulls_first: bool, // 默认 NULLS FIRST
}

/// Return 子句
#[derive(Clone, Debug, PartialEq)]
pub enum ReturnSubClause {
    Star,
    Expr(Expr, Option<String>),
}

/// Insert header 信息
#[derive(Clone, Debug, PartialEq)]
pub enum InsertHeader {
    Insert(ConflictResolution),
    Replace,
}

/// upsert 子句
#[derive(Clone, Debug, PartialEq)]
pub struct UpsertSubClause {
    pub indexed_cols: Vec<IndexedColumn>,
    pub where_clause: Option<Expr>,
    pub upsert_type: UpsertType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpsertType {
    Nothing,
    Update {
        set_clause: Vec<SetSubClause>,
        where_clause: Option<Expr>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompoundOperator {
    Union(bool), // bool 表示是否为 ALL
    Intersect,
    Except,
}
