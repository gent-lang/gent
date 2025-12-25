//! Parser module for GENT - transforms pest CST to AST

pub mod ast;

pub use ast::{
    AgentDecl, AgentField, BinaryOp, Block, BlockStmt, Expression, FieldType, IfStmt, LetStmt,
    OutputType, Param, Program, ReturnStmt, RunStmt, Statement, StructDecl, StructField, ToolDecl,
    TypeName, UnaryOp,
};

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
        Rule::struct_decl => Ok(Statement::StructDecl(parse_struct_decl(inner)?)),
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
        Rule::expression => {
            let inner = pair.into_inner().next().unwrap();
            parse_expression(inner)
        }
        Rule::logical_or => parse_binary_left(pair, &[BinaryOp::Or]),
        Rule::logical_and => parse_binary_left(pair, &[BinaryOp::And]),
        Rule::equality => parse_binary_left(pair, &[BinaryOp::Eq, BinaryOp::Ne]),
        Rule::comparison => parse_binary_left(
            pair,
            &[BinaryOp::Lt, BinaryOp::Le, BinaryOp::Gt, BinaryOp::Ge],
        ),
        Rule::additive => parse_binary_left(pair, &[BinaryOp::Add, BinaryOp::Sub]),
        Rule::multiplicative => {
            parse_binary_left(pair, &[BinaryOp::Mul, BinaryOp::Div, BinaryOp::Mod])
        }
        Rule::unary => parse_unary(pair),
        Rule::postfix => parse_postfix(pair),
        Rule::primary => {
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
        Rule::null_literal => Ok(Expression::Null(span)),
        Rule::identifier => Ok(Expression::Identifier(pair.as_str().to_string(), span)),
        Rule::array_literal => parse_array_literal(pair),
        Rule::object_literal => parse_object_literal(pair),
        _ => Err(GentError::SyntaxError {
            message: format!("Unexpected expression: {:?}", pair.as_rule()),
            span,
        }),
    }
}

/// Parse binary left-associative operators
/// The grammar is: level = { next_level ~ ((op1 | op2 | ...) ~ next_level)* }
/// Pest gives us all children at next_level, so we need to extract operators from source positions
fn parse_binary_left(
    pair: pest::iterators::Pair<Rule>,
    ops: &[BinaryOp],
) -> GentResult<Expression> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let source = pair.as_str();
    let base_pos = pair.as_span().start();

    let inner = pair.into_inner();
    let pairs: Vec<_> = inner.collect();

    // If there's only one element, it's not a binary operation
    if pairs.len() == 1 {
        return parse_expression(pairs[0].clone());
    }

    // Parse first operand
    let mut left = parse_expression(pairs[0].clone())?;
    let mut last_end = pairs[0].as_span().end() - base_pos;

    // Process remaining operands
    for right_pair in pairs.iter().skip(1) {
        let right_start = right_pair.as_span().start() - base_pos;

        // Extract operator from the text between last operand and next operand
        let between = &source[last_end..right_start];
        let op_str = between.trim();

        let op = match op_str {
            "||" => BinaryOp::Or,
            "&&" => BinaryOp::And,
            "==" => BinaryOp::Eq,
            "!=" => BinaryOp::Ne,
            "<" => BinaryOp::Lt,
            "<=" => BinaryOp::Le,
            ">" => BinaryOp::Gt,
            ">=" => BinaryOp::Ge,
            "+" => BinaryOp::Add,
            "-" => BinaryOp::Sub,
            "*" => BinaryOp::Mul,
            "/" => BinaryOp::Div,
            "%" => BinaryOp::Mod,
            _ => {
                return Err(GentError::SyntaxError {
                    message: format!("Unknown operator: {}", op_str),
                    span: span.clone(),
                })
            }
        };

        // Verify this operator is in the expected set
        if !ops.contains(&op) {
            return Err(GentError::SyntaxError {
                message: format!(
                    "Unexpected operator: {} (expected one of {:?})",
                    op_str, ops
                ),
                span: span.clone(),
            });
        }

        let right = parse_expression(right_pair.clone())?;
        left = Expression::Binary(op, Box::new(left), Box::new(right), span.clone());
        last_end = right_pair.as_span().end() - base_pos;
    }

    Ok(left)
}

/// Parse unary operators
fn parse_unary(pair: pest::iterators::Pair<Rule>) -> GentResult<Expression> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let source = pair.as_str();
    let base_pos = pair.as_span().start();

    // Special case: Check if this is a negative number literal
    let full_text = source.trim();
    if full_text.starts_with('-') && full_text.len() > 1 {
        let rest = full_text[1..].trim();
        // Check if it's purely a number (no operators, just digits and maybe a decimal point)
        if rest.chars().all(|c| c.is_ascii_digit() || c == '.') {
            if let Ok(num) = full_text.parse::<f64>() {
                return Ok(Expression::Number(num, span));
            }
        }
    }

    let inner = pair.into_inner();
    let pairs: Vec<_> = inner.collect();

    // If there's only one child, no unary operators
    if pairs.len() == 1 {
        return parse_expression(pairs[0].clone());
    }

    // Collect unary operators from the source text
    let mut ops = Vec::new();

    // The last pair is the operand, everything before are unary operators
    let operand_pair = &pairs[pairs.len() - 1];
    let operand_start = operand_pair.as_span().start() - base_pos;

    // Extract operators from the text before the operand
    let op_text = &source[..operand_start].trim();
    for ch in op_text.chars() {
        match ch {
            '!' => ops.push(UnaryOp::Not),
            '-' => ops.push(UnaryOp::Neg),
            _ => {}
        }
    }

    let mut expr = parse_expression(operand_pair.clone())?;

    // Apply operators right-to-left (from the innermost to outermost)
    for op in ops.into_iter().rev() {
        expr = Expression::Unary(op, Box::new(expr), span.clone());
    }

    Ok(expr)
}

/// Parse postfix expressions (call, member, index)
fn parse_postfix(pair: pest::iterators::Pair<Rule>) -> GentResult<Expression> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let mut expr = parse_expression(inner.next().unwrap())?;

    for postfix_pair in inner {
        match postfix_pair.as_rule() {
            Rule::call_expr => {
                let args = parse_arg_list(postfix_pair)?;
                expr = Expression::Call(Box::new(expr), args, span.clone());
            }
            Rule::member_expr => {
                let member_inner = postfix_pair.into_inner().next().unwrap();
                let member_name = member_inner.as_str().to_string();
                expr = Expression::Member(Box::new(expr), member_name, span.clone());
            }
            Rule::index_expr => {
                let index_inner = postfix_pair.into_inner().next().unwrap();
                let index = parse_expression(index_inner)?;
                expr = Expression::Index(Box::new(expr), Box::new(index), span.clone());
            }
            _ => {}
        }
    }

    Ok(expr)
}

/// Parse argument list for function calls
fn parse_arg_list(pair: pest::iterators::Pair<Rule>) -> GentResult<Vec<Expression>> {
    let mut args = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::arg_list {
            for arg_pair in inner.into_inner() {
                args.push(parse_expression(arg_pair)?);
            }
        }
    }

    Ok(args)
}

/// Parse array literal [1, 2, 3]
fn parse_array_literal(pair: pest::iterators::Pair<Rule>) -> GentResult<Expression> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut elements = Vec::new();

    for inner in pair.into_inner() {
        elements.push(parse_expression(inner)?);
    }

    Ok(Expression::Array(elements, span))
}

/// Parse object literal {key: value, ...}
fn parse_object_literal(pair: pest::iterators::Pair<Rule>) -> GentResult<Expression> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut fields = Vec::new();

    for field_pair in pair.into_inner() {
        if field_pair.as_rule() == Rule::object_field {
            let mut field_inner = field_pair.into_inner();

            let key_pair = field_inner.next().unwrap();
            let key = match key_pair.as_rule() {
                Rule::identifier => key_pair.as_str().to_string(),
                Rule::string_literal => {
                    let raw = key_pair.as_str();
                    let content = &raw[1..raw.len() - 1];
                    unescape_string(content)
                }
                _ => {
                    return Err(GentError::SyntaxError {
                        message: "Expected identifier or string for object key".to_string(),
                        span: Span::new(key_pair.as_span().start(), key_pair.as_span().end()),
                    })
                }
            };

            let value = parse_expression(field_inner.next().unwrap())?;
            fields.push((key, value));
        }
    }

    Ok(Expression::Object(fields, span))
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

    Ok(Param {
        name,
        type_name,
        span,
    })
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
    let value = pair
        .into_inner()
        .next()
        .map(|p| parse_expression(p))
        .transpose()?;

    Ok(ReturnStmt { value, span })
}

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<IfStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let condition = parse_expression(inner.next().unwrap())?;
    let then_block = parse_block(inner.next().unwrap())?;
    let else_block = inner.next().map(|p| parse_block(p)).transpose()?;

    Ok(IfStmt {
        condition,
        then_block,
        else_block,
        span,
    })
}

fn parse_struct_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<StructDecl> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let body = inner.next().unwrap();
    let fields = parse_struct_body(body)?;

    Ok(StructDecl { name, fields, span })
}

fn parse_struct_body(pair: pest::iterators::Pair<Rule>) -> GentResult<Vec<StructField>> {
    let mut fields = Vec::new();
    for field_pair in pair.into_inner() {
        if field_pair.as_rule() == Rule::struct_field {
            fields.push(parse_struct_field(field_pair)?);
        }
    }
    Ok(fields)
}

fn parse_struct_field(pair: pest::iterators::Pair<Rule>) -> GentResult<StructField> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let field_type = parse_field_type(inner.next().unwrap())?;

    Ok(StructField { name, field_type, span })
}

fn parse_field_type(pair: pest::iterators::Pair<Rule>) -> GentResult<FieldType> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::field_type_array => {
            let base = inner.into_inner().next().unwrap();
            let base_type = parse_field_type_base(base)?;
            Ok(FieldType::Array(Box::new(base_type)))
        }
        Rule::field_type_object => {
            let body = inner.into_inner().next().unwrap();
            let fields = parse_struct_body(body)?;
            Ok(FieldType::Object(fields))
        }
        Rule::field_type_named => {
            let name = inner.into_inner().next().unwrap().as_str();
            match name {
                "string" => Ok(FieldType::String),
                "number" => Ok(FieldType::Number),
                "boolean" => Ok(FieldType::Boolean),
                _ => Ok(FieldType::Named(name.to_string())),
            }
        }
        _ => Err(GentError::SyntaxError {
            message: format!("Unexpected field type rule: {:?}", inner.as_rule()),
            span: Span::new(inner.as_span().start(), inner.as_span().end()),
        }),
    }
}

fn parse_field_type_base(pair: pest::iterators::Pair<Rule>) -> GentResult<FieldType> {
    let name = pair.as_str();
    match name {
        "string" => Ok(FieldType::String),
        "number" => Ok(FieldType::Number),
        "boolean" => Ok(FieldType::Boolean),
        _ => Ok(FieldType::Named(name.to_string())),
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
