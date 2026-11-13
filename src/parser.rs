use crate::{
    ast::{Expression, Program, Statement},
    token::Token,
};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }
    fn current(&self) -> Token {
        self.tokens[self.position].clone()
    }
    fn advance(&mut self) {
        self.position += 1;
    }
    fn parse_statement(&mut self) -> Statement {
        match self.current() {
            Token::Type(_) => self.parse_decleration(),
            Token::If => self.parse_if(),
            Token::Print => self.parse_print(),
            Token::Push => self.parse_push(),
            Token::Pop => self.parse_pop(),
            other => panic!("expected statement found {:?}", other),
        }
    }
    fn parse_decleration(&mut self) -> Statement {
        let typee = match self.current() {
            Token::Type(typee) => {
                self.advance();
                typee
            }

            _ => panic!("expected type"),
        };

        let name = match self.current() {
            Token::Name(name) => {
                self.advance();
                name
            }

            _ => panic!("expected name"),
        };

        match self.current() {
            Token::Equals => self.advance(),
            _ => panic!("expected '='"),
        }

        let value = self.parse_expression();

        match self.current() {
            Token::Semicolon => self.advance(),
            _ => panic!("expected ';'"),
        }

        Statement::Decleration { typee, name, value }
    }
    fn parse_if(&mut self) -> Statement {
        self.advance();

        let condition = self.parse_expression();

        match self.current() {
            Token::LBrace => self.advance(),
            _ => panic!("expected '{{'"),
        }

        let mut body = Vec::new();

        while self.current() != Token::RBrace {
            body.push(self.parse_statement());
        }

        self.advance();

        let else_body = if self.current() == Token::Else {
            self.advance();

            match self.current() {
                Token::LBrace => self.advance(),
                _ => panic!("expected '{{'"),
            }
            let mut body = Vec::new();

            while self.current() != Token::RBrace {
                body.push(self.parse_statement());
            }

            self.advance();

            Some(body)
        } else {
            None
        };

        Statement::If {
            condition,
            body,
            else_body,
        }
    }
    fn parse_print(&mut self) -> Statement {
        self.advance();

        let value = self.parse_expression();

        match self.current() {
            Token::Semicolon => self.advance(),
            _ => panic!("expected ';'"),
        }

        Statement::Print { value }
    }
    fn parse_push(&mut self) -> Statement {
        self.advance();

        let value = self.parse_expression();

        match self.current() {
            Token::Semicolon => self.advance(),
            _ => panic!("expected ';'"),
        }

        Statement::Push { value }
    }
    fn parse_pop(&mut self) -> Statement {
        self.advance();

        let typee = match self.current() {
            Token::Type(t) => {
                self.advance();
                t
            }
            _ => panic!("expected type"),
        };

        let name = match self.current() {
            Token::Name(name) => {
                self.advance();
                name
            }
            _ => panic!("expected name"),
        };

        match self.current() {
            Token::Semicolon => self.advance(),
            _ => panic!("expected ';'"),
        }

        Statement::Pop { typee, name }
    }
    fn parse_expression(&mut self) -> Expression {
        let mut left = match self.current() {
            Token::Value(value) => {
                self.advance();
                Expression::Value(value)
            }

            Token::Name(name) => {
                self.advance();
                Expression::Variable(name)
            }

            _ => panic!("expected value"),
        };

        loop {
            let op = match self.current() {
                Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Less
                | Token::LessEqual
                | Token::Greater
                | Token::GreaterEqual
                | Token::EqualEqual
                | Token::NotEqual => {
                    let op = self.current();
                    self.advance();
                    op
                }

                _ => break,
            };

            let right = match self.current() {
                Token::Value(value) => {
                    self.advance();
                    Expression::Value(value)
                }

                Token::Name(name) => {
                    self.advance();
                    Expression::Variable(name)
                }

                _ => panic!("expected value"),
            };

            left = Expression::Operation(Box::new(left), op, Box::new(right));
        }

        left
    }
    pub fn parse_program(&mut self) -> Program {
        let mut statements = Vec::new();

        while self.current() != Token::Eof {
            statements.push(self.parse_statement());
        }

        Program { statements }
    }
}
