//! Abstract Syntax Tree definitions for GENT

use crate::Span;

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
