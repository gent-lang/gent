//! Parser module for GENT - transforms pest CST to AST

pub mod ast;

pub use ast::{BinaryOp, UnaryOp, TypeName, Program, Statement, AgentDecl, AgentField, ToolDecl, Param, RunStmt, Expression, Block, BlockStmt, LetStmt, ReturnStmt, IfStmt};

use crate::errors::{GentError, GentResult, Span};
use crate::lexer::{GentParser, Rule};
use pest::Parser;

/// Parse GENT source code into an AST
pub fn parse(source: &str) -> GentResult<Program> {
    let pairs = GentParser::parse(Rule::program, source).map_err(|e| GentError::SyntaxError {
        message: e.to_string(),
        span: Span::new(0, 0),
    })?;

    let mut statements = Vec::new();
    let program_span = Span::new(0, source.len());

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::statement => {
                        statements.push(parse_statement(inner)?);
                    }
                    Rule::EOI => {}
                    _ => {}
                }
            }
        }
    }

    Ok(Program {
        statements,
        span: program_span,
    })
}

fn parse_statement(pair: pest::iterators::Pair<Rule>) -> GentResult<Statement> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::agent_decl => Ok(Statement::AgentDecl(parse_agent_decl(inner)?)),
        Rule::run_stmt => Ok(Statement::RunStmt(parse_run_stmt(inner)?)),
        _ => Err(GentError::SyntaxError {
            message: format!("Unexpected rule: {:?}", inner.as_rule()),
            span: Span::new(0, 0),
        }),
    }
}

fn parse_agent_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<AgentDecl> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let mut fields = Vec::new();
    let mut tools = Vec::new();

    if let Some(body) = inner.next() {
        for item_pair in body.into_inner() {
            // item_pair is agent_item which contains either use_stmt or agent_field
            let item_inner = item_pair.into_inner().next().unwrap();
            match item_inner.as_rule() {
                Rule::use_stmt => {
                    // Extract tool names from use statement
                    for ident in item_inner.into_inner() {
                        if ident.as_rule() == Rule::identifier {
                            tools.push(ident.as_str().to_string());
                        }
                    }
                }
                Rule::agent_field => {
                    fields.push(parse_agent_field(item_inner)?);
                }
                _ => {}
            }
        }
    }

    Ok(AgentDecl {
        name,
        fields,
        tools,
        span,
    })
}

fn parse_agent_field(pair: pest::iterators::Pair<Rule>) -> GentResult<AgentField> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expression(inner.next().unwrap())?;

    Ok(AgentField { name, value, span })
}

fn parse_run_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<RunStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let agent_name = inner.next().unwrap().as_str().to_string();
    let input = if let Some(args) = inner.next() {
        // run_args contains the "with" expression
        let expr_pair = args.into_inner().next().unwrap();
        Some(parse_expression(expr_pair)?)
    } else {
        None
    };

    Ok(RunStmt {
        agent_name,
        input,
        span,
    })
}

fn parse_expression(pair: pest::iterators::Pair<Rule>) -> GentResult<Expression> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());

    match pair.as_rule() {
        // For most intermediate expression rules, descend to the inner pair
        Rule::expression | Rule::logical_or | Rule::logical_and | Rule::equality |
        Rule::comparison | Rule::additive | Rule::multiplicative |
        Rule::postfix | Rule::primary => {
            let inner = pair.into_inner().next().unwrap();
            parse_expression(inner)
        }
        // Special handling for unary to support negative numbers
        Rule::unary => {
            let full_text = pair.as_str().trim();
            // Check if it starts with a negation and looks like a number
            if full_text.starts_with('-') && full_text.len() > 1 {
                let rest = &full_text[1..].trim_start();
                // Check if the rest is a valid number
                if rest.chars().next().map_or(false, |c| c.is_ascii_digit()) {
                    // Try to parse as a negative number
                    if let Ok(num) = full_text.parse::<f64>() {
                        return Ok(Expression::Number(num, span));
                    }
                }
            }
            // Otherwise, descend normally (will handle ! and other cases in future)
            let inner = pair.into_inner().next().unwrap();
            parse_expression(inner)
        }
        Rule::string_literal => {
            let raw = pair.as_str();
            // Remove quotes and handle escapes
            let content = &raw[1..raw.len() - 1];
            let unescaped = unescape_string(content);
            Ok(Expression::String(unescaped, span))
        }
        Rule::number_literal => {
            let num: f64 = pair.as_str().parse().map_err(|_| GentError::SyntaxError {
                message: format!("Invalid number: {}", pair.as_str()),
                span: span.clone(),
            })?;
            Ok(Expression::Number(num, span))
        }
        Rule::boolean_literal => {
            let val = pair.as_str() == "true";
            Ok(Expression::Boolean(val, span))
        }
        Rule::identifier => Ok(Expression::Identifier(pair.as_str().to_string(), span)),
        Rule::null_literal => Ok(Expression::Identifier("null".to_string(), span)), // Temporary: treat null as identifier
        _ => Err(GentError::SyntaxError {
            message: format!("Unexpected expression: {:?}", pair.as_rule()),
            span,
        }),
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                match next {
                    '"' => {
                        result.push('"');
                        chars.next();
                    }
                    '\\' => {
                        result.push('\\');
                        chars.next();
                    }
                    'n' => {
                        result.push('\n');
                        chars.next();
                    }
                    'r' => {
                        result.push('\r');
                        chars.next();
                    }
                    't' => {
                        result.push('\t');
                        chars.next();
                    }
                    _ => result.push(c),
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result
}
