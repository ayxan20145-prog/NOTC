use crate::token::{Token, Type, Value};

#[derive(Debug)]
pub enum Statement {
    Decleration {
        typee: Type,
        name: String,
        value: Expression,
    },
    If {
        condition: Expression,
        body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    // only for testing i will remove this later
    Print {
        value: Expression,
    },
    Push {
        value: Expression,
    },
    Pop {
        typee: Type,
        name: String,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
}

#[derive(Debug)]
pub enum Expression {
    Value(Value),
    Operation(Box<Expression>, Token, Box<Expression>),
    Variable(String),
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}
