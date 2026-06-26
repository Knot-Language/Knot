use crate::lexer::num::Num;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(Num),
    String(String),
    Identifier(String),
    Keyword(String),
    Operator(String),
    NewLine,
    Error(String),
    Eof,
}
