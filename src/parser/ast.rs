//! Abstract Syntax Tree definitions for GENT

use crate::Span;

/// Binary operators
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not, // !
    Neg, // -
}

/// Part of an interpolated string
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    /// Literal text
    Literal(String),
    /// Interpolated expression
    Expr(Box<Expression>),
}

/// Lambda body - either a single expression or a block
#[derive(Debug, Clone, PartialEq)]
pub enum LambdaBody {
    /// Single expression: (x) => x * 2
    Expression(Box<Expression>),
    /// Block with statements: (x) => { return x * 2 }
    Block(Block),
}

/// Lambda expression: (params) => body
#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    pub params: Vec<String>,
    pub body: LambdaBody,
    pub span: Span,
}

/// Type names in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    String,
    Number,
    Boolean,
    Object,
    Array,
    Any,
}

/// Field types for structured output schemas
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array(Box<FieldType>),
    Object(Vec<StructField>),
    Named(String), // reference to a struct
}

/// A field in a struct or inline object type
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: FieldType,
    pub span: Span,
}

/// Struct declaration: `struct Name implements Interface1, Interface2 { fields... }`
#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub implements: Vec<String>, // interface names this struct implements
    pub fields: Vec<StructField>,
    pub span: Span,
}

/// Enum declaration: `enum Name { Variant1, Variant2(type) }`
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

/// A variant in an enum declaration
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<EnumField>,
    pub span: Span,
}

/// A field in an enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct EnumField {
    pub name: Option<String>,
    pub type_name: String,
    pub span: Span,
}

/// Interface declaration: `interface Name { methods/fields... }`
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDecl {
    pub name: String,
    pub members: Vec<InterfaceMember>,
    pub span: Span,
}

/// A member in an interface declaration
#[derive(Debug, Clone, PartialEq)]
pub enum InterfaceMember {
    Field(InterfaceField),
    Method(InterfaceMethod),
}

/// A field in an interface: `name: type`
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceField {
    pub name: String,
    pub type_name: TypeName,
    pub span: Span,
}

/// A method in an interface: `name(params) -> return_type`
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeName>,
    pub span: Span,
}

/// Duration unit for timeout specifications
#[derive(Debug, Clone, PartialEq)]
pub enum DurationUnit {
    Milliseconds,
    Seconds,
    Minutes,
}

/// Duration value with unit
#[derive(Debug, Clone, PartialEq)]
pub struct Duration {
    pub value: u64,
    pub unit: DurationUnit,
    pub span: Span,
}

impl Duration {
    pub fn to_millis(&self) -> u64 {
        match self.unit {
            DurationUnit::Milliseconds => self.value,
            DurationUnit::Seconds => self.value * 1000,
            DurationUnit::Minutes => self.value * 60 * 1000,
        }
    }
}

/// Parallel execution block declaration
#[derive(Debug, Clone, PartialEq)]
pub struct ParallelDecl {
    pub name: String,
    pub agents: Vec<Expression>,
    pub timeout: Duration,
    pub span: Span,
}

/// Output type specification (inline or named)
#[derive(Debug, Clone, PartialEq)]
pub enum OutputType {
    Inline(Vec<StructField>),
    Named(String),
}

/// Import statement: `import { Name1, Name2 } from "./path.gnt"`
#[derive(Debug, Clone, PartialEq)]
pub struct ImportStmt {
    pub names: Vec<String>,
    pub path: String,
    pub span: Span,
}

/// A complete GENT program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// A statement in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Import(ImportStmt),
    AgentDecl(AgentDecl),
    ToolDecl(ToolDecl),
    FnDecl(FnDecl),
    StructDecl(StructDecl),
    EnumDecl(EnumDecl),
    InterfaceDecl(InterfaceDecl),
    ParallelDecl(ParallelDecl),
    LetStmt(LetStmt),
    TopLevelCall(TopLevelCall),
}

/// A top-level function call: `funcName(args...)`
#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelCall {
    pub name: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

/// An agent declaration: `agent Name { fields... }`
#[derive(Debug, Clone, PartialEq)]
pub struct AgentDecl {
    pub name: String,
    pub fields: Vec<AgentField>,
    pub tools_expr: Option<Expression>, // From `tools:` field
    pub output: Option<OutputType>,
    pub span: Span,
}

/// A field in an agent declaration: `name: value`
#[derive(Debug, Clone, PartialEq)]
pub struct AgentField {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

/// Tool declaration: `tool name(params) -> return_type { body }`
#[derive(Debug, Clone, PartialEq)]
pub struct ToolDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeName>,
    pub body: Block,
    pub span: Span,
}

/// Function declaration (pure, no agent access): `fn name(params) -> return_type { body }`
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeName>,
    pub body: Block,
    pub span: Span,
}

/// Parameter in a tool or function declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_name: TypeName,
    pub span: Span,
}

/// An expression in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// String literal: `"hello"` or interpolated string: `"Hello, ${name}!"`
    String(Vec<StringPart>, Span),
    /// Number literal: `42` or `3.14`
    Number(f64, Span),
    /// Boolean literal: `true` or `false`
    Boolean(bool, Span),
    /// Identifier reference: `varName`
    Identifier(String, Span),
    /// Null literal: `null`
    Null(Span),
    /// Array literal: `[1, 2, 3]`
    Array(Vec<Expression>, Span),
    /// Object literal: `{key: value}`
    Object(Vec<(String, Expression)>, Span),
    /// Binary operation: `a + b`
    Binary(BinaryOp, Box<Expression>, Box<Expression>, Span),
    /// Unary operation: `-x`, `!x`
    Unary(UnaryOp, Box<Expression>, Span),
    /// Function call: `foo(args)`
    Call(Box<Expression>, Vec<Expression>, Span),
    /// Member access: `obj.prop`
    Member(Box<Expression>, String, Span),
    /// Index access: `arr[0]`
    Index(Box<Expression>, Box<Expression>, Span),
    /// Range expression (start..end)
    Range(Box<Expression>, Box<Expression>, Span),
    /// Lambda expression: (x) => x * 2
    Lambda(Lambda),
    /// Match expression: match value { Pattern => result }
    Match(MatchExpr),
}

impl Expression {
    /// Get the span of this expression
    pub fn span(&self) -> &Span {
        match self {
            Expression::String(_, span) => span,
            Expression::Number(_, span) => span,
            Expression::Boolean(_, span) => span,
            Expression::Identifier(_, span) => span,
            Expression::Null(span) => span,
            Expression::Array(_, span) => span,
            Expression::Object(_, span) => span,
            Expression::Binary(_, _, _, span) => span,
            Expression::Unary(_, _, span) => span,
            Expression::Call(_, _, span) => span,
            Expression::Member(_, _, span) => span,
            Expression::Index(_, _, span) => span,
            Expression::Range(_, _, span) => span,
            Expression::Lambda(lambda) => &lambda.span,
            Expression::Match(m) => &m.span,
        }
    }
}

/// A block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<BlockStmt>,
    pub span: Span,
}

/// A statement within a block
#[derive(Debug, Clone, PartialEq)]
pub enum BlockStmt {
    Let(LetStmt),
    Assignment(AssignmentStmt),
    Return(ReturnStmt),
    If(IfStmt),
    For(ForStmt),
    While(WhileStmt),
    Try(TryStmt),
    Break(Span),
    Continue(Span),
    Expr(Expression),
}

/// Let statement: `let x = expr`
#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

/// Assignment statement: `x = expr`
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentStmt {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

/// Return statement: `return expr?`
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Expression>,
    pub span: Span,
}

/// If statement: `if cond { ... } else { ... }`
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expression,
    pub then_block: Block,
    pub else_block: Option<Block>,
    pub span: Span,
}

/// For loop statement
#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    /// Loop variable name
    pub variable: String,
    /// Expression to iterate over
    pub iterable: Expression,
    /// Loop body
    pub body: Block,
    /// Source location
    pub span: Span,
}

/// While loop statement
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    /// Loop condition
    pub condition: Expression,
    /// Loop body
    pub body: Block,
    /// Source location
    pub span: Span,
}

/// Try/catch statement
#[derive(Debug, Clone, PartialEq)]
pub struct TryStmt {
    /// The try block
    pub try_block: Block,
    /// The error variable name in catch
    pub error_var: String,
    /// The catch block
    pub catch_block: Block,
    /// Source location
    pub span: Span,
}

/// Match expression: `match value { Pattern => result }`
#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub subject: Box<Expression>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// A single arm in a match expression
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: MatchBody,
    pub span: Span,
}

/// Pattern in a match arm
#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    /// Wildcard: `_`
    Wildcard,
    /// Enum variant: `Status.Pending` or `Result.Ok(value)`
    EnumVariant {
        enum_name: String,
        variant_name: String,
        bindings: Vec<String>,
    },
}

/// Body of a match arm
#[derive(Debug, Clone, PartialEq)]
pub enum MatchBody {
    Expression(Box<Expression>),
    Block(Block),
}
