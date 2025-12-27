//! Parser module for GENT - transforms pest CST to AST

pub mod ast;

pub use ast::{
    AgentDecl, AgentField, AssignmentStmt, BinaryOp, Block, BlockStmt, Duration, DurationUnit,
    EnumDecl, EnumField, EnumVariant, Expression, FieldType, FnDecl, ForStmt, IfStmt, ImportStmt,
    InterfaceDecl, InterfaceField, InterfaceMember, InterfaceMethod, Lambda, LambdaBody, LetStmt,
    MatchArm, MatchBody, MatchExpr, MatchPattern, OutputType, ParallelDecl, Param, Program,
    ReturnStmt, Statement, StringPart, StructDecl, StructField, ToolDecl, TopLevelCall, TryStmt,
    TypeName, UnaryOp, WhileStmt,
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
        Rule::import_stmt => Ok(Statement::Import(parse_import_stmt(inner)?)),
        Rule::struct_decl => Ok(Statement::StructDecl(parse_struct_decl(inner)?)),
        Rule::enum_decl => Ok(Statement::EnumDecl(parse_enum_decl(inner)?)),
        Rule::interface_decl => Ok(Statement::InterfaceDecl(parse_interface_decl(inner)?)),
        Rule::agent_decl => Ok(Statement::AgentDecl(parse_agent_decl(inner)?)),
        Rule::tool_decl => Ok(Statement::ToolDecl(parse_tool_decl(inner)?)),
        Rule::fn_decl => Ok(Statement::FnDecl(parse_fn_decl(inner)?)),
        Rule::parallel_decl => Ok(Statement::ParallelDecl(parse_parallel_decl(inner)?)),
        Rule::top_level_let => Ok(Statement::LetStmt(parse_top_level_let(inner)?)),
        Rule::top_level_call => Ok(Statement::TopLevelCall(parse_top_level_call(inner)?)),
        _ => Err(GentError::SyntaxError {
            message: format!("Unexpected rule: {:?}", inner.as_rule()),
            span: Span::new(0, 0),
        }),
    }
}

fn parse_import_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<ImportStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let import_list = inner.next().unwrap();
    let mut names = Vec::new();
    for ident in import_list.into_inner() {
        if ident.as_rule() == Rule::identifier {
            names.push(ident.as_str().to_string());
        }
    }

    let path_pair = inner.next().unwrap();
    let raw_path = path_pair.as_str();
    // Remove the quotes from the string literal
    let path = raw_path[1..raw_path.len() - 1].to_string();

    Ok(ImportStmt { names, path, span })
}

fn parse_top_level_let(pair: pest::iterators::Pair<Rule>) -> GentResult<LetStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expression(inner.next().unwrap())?;

    Ok(LetStmt { name, value, span })
}

fn parse_agent_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<AgentDecl> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let mut fields = Vec::new();
    let mut tools_expr = None;
    let mut output = None;

    if let Some(body) = inner.next() {
        for item_pair in body.into_inner() {
            // item_pair is agent_item which contains tools_field, output_field, or agent_field
            let item_inner = item_pair.into_inner().next().unwrap();
            match item_inner.as_rule() {
                Rule::tools_field => {
                    // Parse tools field: tools: expression
                    let expr_pair = item_inner.into_inner().next().unwrap();
                    tools_expr = Some(parse_expression(expr_pair)?);
                }
                Rule::output_field => {
                    // Parse output field directly from grammar rule
                    output = Some(parse_output_field(item_inner)?);
                }
                Rule::agent_field => {
                    let field = parse_agent_field(item_inner)?;
                    // Legacy support: Check if this is the output field (shouldn't happen with new grammar)
                    if field.name == "output" {
                        output = Some(parse_output_type_from_expr(&field.value)?);
                    } else {
                        fields.push(field);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(AgentDecl {
        name,
        fields,
        tools_expr,
        output,
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

/// Parse output field: "output" ~ ":" ~ output_type
fn parse_output_field(pair: pest::iterators::Pair<Rule>) -> GentResult<OutputType> {
    // output_field = { "output" ~ ":" ~ output_type }
    // We only get the output_type part since "output" and ":" are consumed by grammar
    let output_type_pair = pair.into_inner().next().unwrap();
    parse_output_type(output_type_pair)
}

/// Parse output_type = { field_type_object | identifier }
fn parse_output_type(pair: pest::iterators::Pair<Rule>) -> GentResult<OutputType> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::field_type_object => {
            let fields = parse_struct_body_from_object(inner)?;
            Ok(OutputType::Inline(fields))
        }
        Rule::identifier => Ok(OutputType::Named(inner.as_str().to_string())),
        _ => Err(GentError::SyntaxError {
            message: format!("Expected output type, got {:?}", inner.as_rule()),
            span: Span::new(inner.as_span().start(), inner.as_span().end()),
        }),
    }
}

/// Parse struct_body from field_type_object: "{" ~ struct_body ~ "}"
fn parse_struct_body_from_object(
    pair: pest::iterators::Pair<Rule>,
) -> GentResult<Vec<StructField>> {
    let mut fields = Vec::new();
    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::struct_body {
            for field_pair in inner.into_inner() {
                if field_pair.as_rule() == Rule::struct_field {
                    fields.push(parse_struct_field(field_pair)?);
                }
            }
        }
    }
    Ok(fields)
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
            let mut parts = Vec::new();

            for inner in pair.into_inner() {
                match inner.as_rule() {
                    Rule::string_chars => {
                        let text = unescape_string(inner.as_str());
                        parts.push(StringPart::Literal(text));
                    }
                    Rule::interpolation => {
                        let expr_pair = inner.into_inner().next().unwrap();
                        let expr = parse_expression(expr_pair)?;
                        parts.push(StringPart::Expr(Box::new(expr)));
                    }
                    Rule::string_part => {
                        // Handle nested string_part if needed
                        for sub in inner.into_inner() {
                            match sub.as_rule() {
                                Rule::string_chars => {
                                    let text = unescape_string(sub.as_str());
                                    parts.push(StringPart::Literal(text));
                                }
                                Rule::interpolation => {
                                    let expr_pair = sub.into_inner().next().unwrap();
                                    let expr = parse_expression(expr_pair)?;
                                    parts.push(StringPart::Expr(Box::new(expr)));
                                }
                                _ => {}
                            }
                        }
                    }
                    Rule::multiline_string => {
                        // Handle multiline string (triple quotes)
                        for sub in inner.into_inner() {
                            match sub.as_rule() {
                                Rule::multiline_chars => {
                                    // No unescape needed for multiline - preserve content as-is
                                    parts.push(StringPart::Literal(sub.as_str().to_string()));
                                }
                                Rule::multiline_interpolation => {
                                    let expr_pair = sub.into_inner().next().unwrap();
                                    let expr = parse_expression(expr_pair)?;
                                    parts.push(StringPart::Expr(Box::new(expr)));
                                }
                                Rule::multiline_part => {
                                    for subsub in sub.into_inner() {
                                        match subsub.as_rule() {
                                            Rule::multiline_chars => {
                                                parts.push(StringPart::Literal(subsub.as_str().to_string()));
                                            }
                                            Rule::multiline_interpolation => {
                                                let expr_pair = subsub.into_inner().next().unwrap();
                                                let expr = parse_expression(expr_pair)?;
                                                parts.push(StringPart::Expr(Box::new(expr)));
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            // If no parts, it's an empty string
            if parts.is_empty() {
                parts.push(StringPart::Literal(String::new()));
            }

            Ok(Expression::String(parts, span))
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
        Rule::range_expr => parse_range_expr(pair),
        Rule::lambda => Ok(Expression::Lambda(parse_lambda(pair)?)),
        Rule::match_expr => Ok(Expression::Match(parse_match_expr(pair)?)),
        _ => Err(GentError::SyntaxError {
            message: format!("Unexpected expression: {:?}", pair.as_rule()),
            span,
        }),
    }
}

fn parse_lambda(pair: pest::iterators::Pair<Rule>) -> GentResult<Lambda> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    // Parse parameters (optional)
    let mut params = Vec::new();
    if let Some(params_pair) = inner.peek() {
        if params_pair.as_rule() == Rule::lambda_params {
            let params_pair = inner.next().unwrap();
            for param in params_pair.into_inner() {
                params.push(param.as_str().to_string());
            }
        }
    }

    // Parse body (lambda_body wraps either block or expression)
    let body_pair = inner.next().unwrap();
    let body = if body_pair.as_rule() == Rule::lambda_body {
        // Unwrap lambda_body to get the actual block or expression
        let inner_body = body_pair.into_inner().next().unwrap();
        match inner_body.as_rule() {
            Rule::block => LambdaBody::Block(parse_block(inner_body)?),
            _ => LambdaBody::Expression(Box::new(parse_expression(inner_body)?)),
        }
    } else {
        // Direct handling (shouldn't happen but handle gracefully)
        match body_pair.as_rule() {
            Rule::block => LambdaBody::Block(parse_block(body_pair)?),
            _ => LambdaBody::Expression(Box::new(parse_expression(body_pair)?)),
        }
    };

    Ok(Lambda { params, body, span })
}

fn parse_match_expr(pair: pest::iterators::Pair<Rule>) -> GentResult<MatchExpr> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let subject = Box::new(parse_expression(inner.next().unwrap())?);
    let mut arms = Vec::new();

    for arm_pair in inner {
        arms.push(parse_match_arm(arm_pair)?);
    }

    Ok(MatchExpr { subject, arms, span })
}

fn parse_match_arm(pair: pest::iterators::Pair<Rule>) -> GentResult<MatchArm> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let pattern_pair = inner.next().unwrap();
    let pattern = parse_match_pattern(pattern_pair)?;

    let body_pair = inner.next().unwrap();
    let body = parse_match_body(body_pair)?;

    Ok(MatchArm { pattern, body, span })
}

fn parse_match_pattern(pair: pest::iterators::Pair<Rule>) -> GentResult<MatchPattern> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::wildcard_pattern => Ok(MatchPattern::Wildcard),
        Rule::enum_pattern => {
            let mut parts = inner.into_inner();
            let enum_name = parts.next().unwrap().as_str().to_string();
            let variant_name = parts.next().unwrap().as_str().to_string();

            let mut bindings = Vec::new();
            if let Some(bindings_pair) = parts.next() {
                for binding in bindings_pair.into_inner() {
                    bindings.push(binding.as_str().to_string());
                }
            }

            Ok(MatchPattern::EnumVariant {
                enum_name,
                variant_name,
                bindings,
            })
        }
        _ => unreachable!(),
    }
}

fn parse_match_body(pair: pest::iterators::Pair<Rule>) -> GentResult<MatchBody> {
    let inner = pair.into_inner().next().unwrap();

    match inner.as_rule() {
        Rule::block => Ok(MatchBody::Block(parse_block(inner)?)),
        _ => Ok(MatchBody::Expression(Box::new(parse_expression(inner)?))),
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

fn parse_fn_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<FnDecl> {
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

    Ok(FnDecl {
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

fn parse_top_level_call(pair: pest::iterators::Pair<Rule>) -> GentResult<TopLevelCall> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut args = Vec::new();
    if let Some(arg_list) = inner.next() {
        for arg_pair in arg_list.into_inner() {
            args.push(parse_expression(arg_pair)?);
        }
    }

    Ok(TopLevelCall { name, args, span })
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
        Rule::assignment_stmt => Ok(BlockStmt::Assignment(parse_assignment_stmt(inner)?)),
        Rule::return_stmt => Ok(BlockStmt::Return(parse_return_stmt(inner)?)),
        Rule::if_stmt => Ok(BlockStmt::If(parse_if_stmt(inner)?)),
        Rule::for_stmt => Ok(BlockStmt::For(parse_for_stmt(inner)?)),
        Rule::while_stmt => Ok(BlockStmt::While(parse_while_stmt(inner)?)),
        Rule::try_stmt => Ok(BlockStmt::Try(parse_try_stmt(inner)?)),
        Rule::break_stmt => {
            let span = Span::new(inner.as_span().start(), inner.as_span().end());
            Ok(BlockStmt::Break(span))
        }
        Rule::continue_stmt => {
            let span = Span::new(inner.as_span().start(), inner.as_span().end());
            Ok(BlockStmt::Continue(span))
        }
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

fn parse_assignment_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<AssignmentStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let value = parse_expression(inner.next().unwrap())?;

    Ok(AssignmentStmt { name, value, span })
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

fn parse_for_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<ForStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let variable = inner.next().unwrap().as_str().to_string();
    let iterable = parse_expression(inner.next().unwrap())?;
    let body = parse_block(inner.next().unwrap())?;

    Ok(ForStmt {
        variable,
        iterable,
        body,
        span,
    })
}

fn parse_while_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<WhileStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let condition = parse_expression(inner.next().unwrap())?;
    let body = parse_block(inner.next().unwrap())?;

    Ok(WhileStmt {
        condition,
        body,
        span,
    })
}

fn parse_try_stmt(pair: pest::iterators::Pair<Rule>) -> GentResult<TryStmt> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let try_block = parse_block(inner.next().unwrap())?;
    let error_var = inner.next().unwrap().as_str().to_string();
    let catch_block = parse_block(inner.next().unwrap())?;

    Ok(TryStmt {
        try_block,
        error_var,
        catch_block,
        span,
    })
}

fn parse_range_expr(pair: pest::iterators::Pair<Rule>) -> GentResult<Expression> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let start = parse_expression(inner.next().unwrap())?;
    let end = parse_expression(inner.next().unwrap())?;

    Ok(Expression::Range(Box::new(start), Box::new(end), span))
}

fn parse_struct_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<StructDecl> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    // Check if next is implements_clause or struct_body
    let mut implements = Vec::new();
    let next = inner.next().unwrap();

    let body = if next.as_rule() == Rule::implements_clause {
        // Parse the implements clause - collect all interface names
        for ident in next.into_inner() {
            if ident.as_rule() == Rule::identifier {
                implements.push(ident.as_str().to_string());
            }
        }
        // struct_body comes after implements_clause
        inner.next().unwrap()
    } else {
        // No implements clause, next is the struct_body
        next
    };

    let fields = parse_struct_body(body)?;

    Ok(StructDecl {
        name,
        implements,
        fields,
        span,
    })
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

    Ok(StructField {
        name,
        field_type,
        span,
    })
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
                    '{' => {
                        result.push('{');
                        chars.next();
                    }
                    '}' => {
                        result.push('}');
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

fn parse_output_type_from_expr(expr: &Expression) -> GentResult<OutputType> {
    match expr {
        Expression::Identifier(name, _) => Ok(OutputType::Named(name.clone())),
        Expression::Object(fields, _) => {
            let struct_fields = fields
                .iter()
                .map(|(name, value)| {
                    let field_type = infer_field_type_from_expr(value)?;
                    Ok(StructField {
                        name: name.clone(),
                        field_type,
                        span: value.span().clone(),
                    })
                })
                .collect::<GentResult<Vec<_>>>()?;
            Ok(OutputType::Inline(struct_fields))
        }
        _ => Err(GentError::SyntaxError {
            message: "output must be an identifier or object type".to_string(),
            span: expr.span().clone(),
        }),
    }
}

fn infer_field_type_from_expr(expr: &Expression) -> GentResult<FieldType> {
    match expr {
        Expression::Identifier(name, _) => match name.as_str() {
            "string" => Ok(FieldType::String),
            "number" => Ok(FieldType::Number),
            "boolean" => Ok(FieldType::Boolean),
            _ => Ok(FieldType::Named(name.clone())),
        },
        _ => Err(GentError::SyntaxError {
            message: "Invalid type expression".to_string(),
            span: expr.span().clone(),
        }),
    }
}

fn parse_enum_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<EnumDecl> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let mut variants = Vec::new();

    if let Some(body) = inner.next() {
        for variant_pair in body.into_inner() {
            variants.push(parse_enum_variant(variant_pair)?);
        }
    }

    Ok(EnumDecl { name, variants, span })
}

fn parse_enum_variant(pair: pest::iterators::Pair<Rule>) -> GentResult<EnumVariant> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let mut fields = Vec::new();

    // Check for variant data
    if let Some(data_pair) = inner.next() {
        let field_list = data_pair.into_inner().next().unwrap();
        for field_pair in field_list.into_inner() {
            fields.push(parse_enum_field(field_pair)?);
        }
    }

    Ok(EnumVariant { name, fields, span })
}

fn parse_enum_field(pair: pest::iterators::Pair<Rule>) -> GentResult<EnumField> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let first = inner.next().unwrap();
    let (name, type_name) = if let Some(second) = inner.next() {
        // Named field: name: type
        (Some(first.as_str().to_string()), second.as_str().to_string())
    } else {
        // Unnamed field: just type
        (None, first.as_str().to_string())
    };

    Ok(EnumField { name, type_name, span })
}

fn parse_interface_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<InterfaceDecl> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let mut members = Vec::new();

    if let Some(body) = inner.next() {
        for member_pair in body.into_inner() {
            members.push(parse_interface_member(member_pair)?);
        }
    }

    Ok(InterfaceDecl {
        name,
        members,
        span,
    })
}

fn parse_interface_member(pair: pest::iterators::Pair<Rule>) -> GentResult<InterfaceMember> {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::interface_method => Ok(InterfaceMember::Method(parse_interface_method(inner)?)),
        Rule::interface_field => Ok(InterfaceMember::Field(parse_interface_field(inner)?)),
        _ => Err(GentError::SyntaxError {
            message: format!("Unexpected interface member rule: {:?}", inner.as_rule()),
            span: Span::new(inner.as_span().start(), inner.as_span().end()),
        }),
    }
}

fn parse_interface_field(pair: pest::iterators::Pair<Rule>) -> GentResult<InterfaceField> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let type_pair = inner.next().unwrap();
    let type_name = parse_type_name(type_pair)?;

    Ok(InterfaceField {
        name,
        type_name,
        span,
    })
}

fn parse_interface_method(pair: pest::iterators::Pair<Rule>) -> GentResult<InterfaceMethod> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();
    let mut params = Vec::new();
    let mut return_type = None;

    for item in inner {
        match item.as_rule() {
            Rule::param_list => {
                params = parse_param_list(item)?;
            }
            Rule::return_type => {
                let type_pair = item.into_inner().next().unwrap();
                return_type = Some(parse_type_name(type_pair)?);
            }
            _ => {}
        }
    }

    Ok(InterfaceMethod {
        name,
        params,
        return_type,
        span,
    })
}

fn parse_parallel_decl(pair: pest::iterators::Pair<Rule>) -> GentResult<ParallelDecl> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let mut inner = pair.into_inner();

    let name = inner.next().unwrap().as_str().to_string();

    let mut agents = Vec::new();
    let mut timeout = None;

    // Parse parallel_body -> parallel_field*
    for field_pair in inner {
        match field_pair.as_rule() {
            Rule::parallel_body => {
                for item in field_pair.into_inner() {
                    match item.as_rule() {
                        Rule::parallel_field => {
                            let field_inner = item.into_inner().next().unwrap();
                            match field_inner.as_rule() {
                                Rule::agents_field => {
                                    for expr_pair in field_inner.into_inner() {
                                        agents.push(parse_expression(expr_pair)?);
                                    }
                                }
                                Rule::timeout_field => {
                                    let duration_pair = field_inner.into_inner().next().unwrap();
                                    timeout = Some(parse_duration(duration_pair)?);
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    let timeout = timeout.ok_or_else(|| GentError::SyntaxError {
        message: "parallel block requires 'timeout' field".to_string(),
        span: span.clone(),
    })?;

    if agents.is_empty() {
        return Err(GentError::SyntaxError {
            message: "parallel block requires at least one agent in 'agents' field".to_string(),
            span: span.clone(),
        });
    }

    Ok(ParallelDecl {
        name,
        agents,
        timeout,
        span,
    })
}

fn parse_duration(pair: pest::iterators::Pair<Rule>) -> GentResult<Duration> {
    let span = Span::new(pair.as_span().start(), pair.as_span().end());
    let text = pair.as_str();

    // Parse "30s", "2m", "500ms"
    let (value_str, unit) = if text.ends_with("ms") {
        (&text[..text.len() - 2], DurationUnit::Milliseconds)
    } else if text.ends_with('s') {
        (&text[..text.len() - 1], DurationUnit::Seconds)
    } else if text.ends_with('m') {
        (&text[..text.len() - 1], DurationUnit::Minutes)
    } else {
        return Err(GentError::SyntaxError {
            message: format!("Invalid duration: {}", text),
            span,
        });
    };

    let value = value_str.parse::<u64>().map_err(|_| GentError::SyntaxError {
        message: format!("Invalid duration value: {}", value_str),
        span: span.clone(),
    })?;

    Ok(Duration { value, unit, span })
}
