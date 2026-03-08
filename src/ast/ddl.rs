//! 数据定义语言（Data Definition Language, DDL）相关的抽象语法树定义

use crate::{Dml, Expr, SchemaObject, Select};

/// 建表语句
#[derive(Clone, Debug, PartialEq)]
pub struct CreateTable {
    pub temp: bool,
    pub if_not_exists: bool,
    pub schema_table: SchemaObject,
    pub body: CreateTableBody,
}

/// 索引创建语句
#[derive(Clone, Debug, PartialEq)]
pub struct CreateIndex {
    pub unique: bool,
    pub if_not_exists: bool,
    pub schema_index: SchemaObject,
    pub table_name: String,
    pub indexed_cols: Vec<IndexedColumn>,
    pub where_cond: Option<Expr>,
}

/// 视图创建语句
#[derive(Clone, Debug, PartialEq)]
pub struct CreateView {
    pub temp: bool,
    pub if_not_exists: bool,
    pub schema_view: SchemaObject,
    pub cols: Vec<String>,
    pub select: Select,
}

/// 触发器创建语句
#[derive(Clone, Debug, PartialEq)]
pub struct CreateTrigger {
    pub temp: bool,
    pub if_not_exists: bool,
    pub schema_trigger: SchemaObject,
    pub timing: TriggerTiming,
    pub event: TriggerEvent,
    pub table_name: String,
    pub when_cond: Option<Expr>,
    pub statements: Vec<Dml>,
}

/// 改表语句
#[derive(Clone, Debug, PartialEq)]
pub struct AlterTable {
    pub schema_table: SchemaObject,
    pub action: AlterTableAction,
}

/// 删表语句
#[derive(Clone, Debug, PartialEq)]
pub struct DropTable {
    pub if_exists: bool,
    pub schema_table: SchemaObject,
}

/// 索引删除语句
#[derive(Clone, Debug, PartialEq)]
pub struct DropIndex {
    pub if_exists: bool,
    pub schema_index: SchemaObject,
}

/// 视图删除语句
#[derive(Clone, Debug, PartialEq)]
pub struct DropView {
    pub if_exists: bool,
    pub schema_view: SchemaObject,
}

/// 触发器删除语句
#[derive(Clone, Debug, PartialEq)]
pub struct DropTrigger {
    pub if_exists: bool,
    pub schema_trigger: SchemaObject,
}

/// 改表操作
#[derive(Clone, Debug, PartialEq)]
pub enum AlterTableAction {
    RenameTable(String),
    RenameColumn(String, String),
    AddColumn(ColumnDef),
    DropColumn(String),
}

/// 列定义
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: Option<TypeName>,
    pub constraints: Vec<ColumnConstraint>,
}

/// 字段类型
#[derive(Clone, Debug, PartialEq)]
pub struct TypeName {
    pub affinity: Affinity,
    pub size: Option<TypeSize>,
}

/// 类型亲和力
/// <https://www.sqlite.org/datatype3.html#type_affinity>
#[derive(Clone, Debug, PartialEq)]
pub enum Affinity {
    Integer,
    Real,
    Numeric,
    Text,
    Blob,
}

/// 字段类型大小
#[derive(Clone, Debug, PartialEq)]
pub enum TypeSize {
    MaxSize(String),
    TypeSize(String, String),
}

/// 列级约束
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnConstraint {
    pub name: Option<String>,
    pub ty: ColumnConstraintType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnConstraintType {
    PrimaryKey {
        asc: bool, // 默认升序
        auto_inc: bool,
    },
    NotNull,
    Unique,
    Check(Expr),
    Default(Expr),
    ForeignKey(ForeignKey),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForeignKey {
    pub schema_table: SchemaObject,
    pub cols: Vec<String>,
}

/// 表级约束
#[derive(Clone, Debug, PartialEq)]
pub struct TableConstraint {
    pub name: Option<String>,
    pub cols: Vec<IndexedColumn>,
    pub ty: TableConstraintType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TableConstraintType {
    PrimaryKey,
    Unique,
}

/// 被索引的列
#[derive(Clone, Debug, PartialEq)]
pub struct IndexedColumn {
    pub name: String,
    pub asc: bool, // 默认升序
}

#[derive(Clone, Debug, PartialEq)]
pub enum CreateTableBody {
    Select(Select),
    Columns {
        cols: Vec<ColumnDef>,
        constraints: Vec<TableConstraint>,
        options: Vec<TableOption>,
    },
}

/// 表选项
#[derive(Clone, Debug, PartialEq)]
pub enum TableOption {
    WithoutRowid,
    Strict,
}

/// 触发器时机
#[derive(Clone, Debug, PartialEq)]
pub enum TriggerTiming {
    Before,
    After,
    InsteadOf,
}

/// 触发器事件
#[derive(Clone, Debug, PartialEq)]
pub enum TriggerEvent {
    Delete,
    Insert,
    Update(Vec<String>), // 更新指定列
}
