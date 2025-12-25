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
        Rule::tool_decl => Ok(Statement::ToolDecl(parse_tool_decl(inner)?)),
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

fn parse_tool_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<ToolDecl> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let mut params = Vec::new();
    let mut return_type = None;
    let mut body = None;

    for item in inner {
        match item.as_rule() {
            Rule::param_list => {
                params = parse_param_list(item)?;
            }
            Rule::return_type => {
                let type_pair = item.into_inner().next().unwrap();
                return_type = Some(parse_type_name(type_pair)?);
            }
            Rule::block => {
                body = Some(parse_block(item)?);
            }
            _ => {}
        }
    }

    Ok(ToolDecl {
        name,
        params,
        return_type,
        body: body.unwrap_or_else(|| Block {
            statements: vec![],
            span: span.clone(),
        }),
        span,
    })
}

fn parse_param_list(pair: pest::iterators::Pair<Rule>) -> GentResult<Vec<Param>> {
    let mut params = Vec::new();
    for param_pair in pair.into_inner() {
        if param_pair.as_rule() == Rule::param {
            params.push(parse_param(param_pair)?);
        }
    }
    Ok(params)
}

fn parse_param(pair: pest::iterators::Pair<Rule>) -> GentResult<Param> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let type_pair = inner.next().unwrap();
    let type_name = parse_type_name(type_pair)?;

    Ok(Param { name, type_name, span })
}

fn parse_type_name(pair: pest::iterators::Pair<Rule>) -> GentResult<TypeName> {
    match pair.as_str() {
        "string" => Ok(TypeName::String),
        "number" => Ok(TypeName::Number),
        "boolean" => Ok(TypeName::Boolean),
        "object" => Ok(TypeName::Object),
        "array" => Ok(TypeName::Array),
        "any" => Ok(TypeName::Any),
        other => Err(GentError::SyntaxError {
            message: format!("Unknown type: {}", other),
            span: Span::new(pair.as_span().start(), pair.as_span().end()),
        }),
    }
}

fn parse_block(pair: pest::iterators::Pair<Rule>) -> GentResult<Block> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut statements = Vec::new();

    for stmt_pair in pair.into_inner() {
        if stmt_pair.as_rule() == Rule::block_stmt {
            statements.push(parse_block_stmt(stmt_pair)?);
        }
    }

    Ok(Block { statements, span })
}

fn parse_block_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<BlockStmt> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::let_stmt => Ok(BlockStmt::Let(parse_let_stmt(inner)?)),
        Rule::return_stmt => Ok(BlockStmt::Return(parse_return_stmt(inner)?)),
        Rule::if_stmt => Ok(BlockStmt::If(parse_if_stmt(inner)?)),
        Rule::expr_stmt => {
            let expr_pair = inner.into_inner().next().unwrap();
            Ok(BlockStmt::Expr(parse_expression(expr_pair)?))
        }
        _ => Err(GentError::SyntaxError {
            message: format!("Unexpected block statement: {:?}", inner.as_rule()),
            span: Span::new(0, 0),
        }),
    }
}

fn parse_let_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<LetStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expression(inner.next().unwrap())?;

    Ok(LetStmt { name, value, span })
}

fn parse_return_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<ReturnStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let value = pair.into_inner().next().map(|p| parse_expression(p)).transpose()?;

    Ok(ReturnStmt { value, span })
}

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<IfStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let condition = parse_expression(inner.next().unwrap())?;
    let then_block = parse_block(inner.next().unwrap())?;
    let else_block = inner.next().map(|p| parse_block(p)).transpose()?;

    Ok(IfStmt { condition, then_block, else_block, span })
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
