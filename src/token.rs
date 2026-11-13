use std::fmt;

static KEYWORDS: &[(&str, Token)] = &[
    ("int", Token::Type(Type::Int)),
    ("char", Token::Type(Type::Char)),
    ("str", Token::Type(Type::Str)),
    ("bool", Token::Type(Type::Bool)),
    ("true", Token::Value(Value::Bool(true))),
    ("false", Token::Value(Value::Bool(false))),
    ("if", Token::If),
    ("else", Token::Else),
    ("print", Token::Print),
    ("push", Token::Push),
    ("pop", Token::Pop),
];

pub fn check_keyword(name: &str) -> Option<Token> {
    KEYWORDS
        .iter()
        .find(|(kw, _)| *kw == name)
        .map(|(_, tok)| tok.clone())
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Type(Type),
    Name(String),
    Equals,
    Value(Value),

    Plus,
    Minus,
    Star,
    Slash,

    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    EqualEqual,
    NotEqual,

    If,
    Else,

    LBrace,
    RBrace,

    Print,
    Push,
    Pop,

    Semicolon,
    Eof,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Int,
    Char,
    Str,
    Bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Int(i32),
    Char(u8),
    Str(Vec<u8>),
    Bool(bool),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(value) => write!(f, "{}", value),
            Value::Char(value) => write!(f, "{}", value),
            Value::Str(_) => unreachable!(),
            Value::Bool(value) => write!(f, "{}", value),
        }
    }
}
