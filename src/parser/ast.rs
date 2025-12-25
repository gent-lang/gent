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

/// A complete GENT program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub span: Span,
}

/// A statement in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    AgentDecl(AgentDecl),
    RunStmt(RunStmt),
}

/// An agent declaration: `agent Name { fields... }`
#[derive(Debug, Clone, PartialEq)]
pub struct AgentDecl {
    pub name: String,
    pub fields: Vec<AgentField>,
    pub tools: Vec<String>, // Tool names from `use` statements
    pub span: Span,
}

/// A field in an agent declaration: `name: value`
#[derive(Debug, Clone, PartialEq)]
pub struct AgentField {
    pub name: String,
    pub value: Expression,
    pub span: Span,
}

/// A run statement: `run AgentName` or `run AgentName with input`
#[derive(Debug, Clone, PartialEq)]
pub struct RunStmt {
    pub agent_name: String,
    pub input: Option<Expression>,
    pub span: Span,
}

/// An expression in GENT
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// String literal: `"hello"`
    String(String, Span),
    /// Number literal: `42` or `3.14`
    Number(f64, Span),
    /// Boolean literal: `true` or `false`
    Boolean(bool, Span),
    /// Identifier reference: `varName`
    Identifier(String, Span),
}

impl Expression {
    /// Get the span of this expression
    pub fn span(&self) -> &Span {
        match self {
            Expression::String(_, span) => span,
            Expression::Number(_, span) => span,
            Expression::Boolean(_, span) => span,
            Expression::Identifier(_, span) => span,
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
    Return(ReturnStmt),
    If(IfStmt),
    Expr(Expression),
}

/// Let statement: `let x = expr`
#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
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
