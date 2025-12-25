//! Lexer module for GENT - uses pest for parsing

use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lexer/grammar.pest"]
pub struct GentParser;
