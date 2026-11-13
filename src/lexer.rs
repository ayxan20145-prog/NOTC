use crate::token::{Token, Value, check_keyword};

pub struct Lexer {
    source: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
        }
    }
    fn current(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }
    fn advance(&mut self) {
        self.position += 1;
    }
    fn next_token(&mut self) -> Token {
        loop {
            match self.current() {
                Some(c) => {
                    if c.is_whitespace() {
                        self.advance();
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        match self.current() {
            None => Token::Eof,

            Some(';') => {
                self.advance();
                Token::Semicolon
            }

            Some('+') => {
                self.advance();
                Token::Plus
            }

            Some('-') => {
                self.advance();
                Token::Minus
            }

            Some('*') => {
                self.advance();
                Token::Star
            }

            Some('/') => {
                self.advance();
                Token::Slash
            }

            Some('=') => {
                self.advance();

                if self.current() == Some('=') {
                    self.advance();
                    Token::EqualEqual
                } else {
                    Token::Equals
                }
            }

            Some('!') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::NotEqual
                } else {
                    panic!("unexpected '!'");
                }
            }

            Some('<') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::LessEqual
                } else {
                    Token::Less
                }
            }

            Some('>') => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Token::GreaterEqual
                } else {
                    Token::Greater
                }
            }

            Some('{') => {
                self.advance();
                Token::LBrace
            }

            Some('}') => {
                self.advance();
                Token::RBrace
            }

            Some('\'') => {
                self.advance();

                let byte = match self.current() {
                    None => panic!("unterminated char"),
                    Some('\\') => {
                        self.advance();
                        let esc = self.current().expect("unterminated escape");
                        self.advance();
                        match esc {
                            'n' => b'\n',
                            't' => b'\t',
                            'r' => b'\r',
                            '0' => 0,
                            '\\' => b'\\',
                            '\'' => b'\'',
                            other => panic!("unknown escape: \\{}", other),
                        }
                    }
                    Some(c) => {
                        self.advance();
                        let cp = c as u32;
                        if cp > 255 {
                            panic!("error message");
                        }
                        cp as u8
                    }
                };

                match self.current() {
                    Some('\'') => {
                        self.advance();
                        Token::Value(Value::Char(byte))
                    }
                    _ => panic!("expected closing quote"),
                }
            }

            Some('"') => {
                self.advance();
                let mut bytes = Vec::new();
                loop {
                    match self.current() {
                        None => panic!("unterminated string"),
                        Some('"') => {
                            self.advance();
                            break;
                        }
                        Some('\\') => {
                            self.advance();
                            let esc = self.current().expect("unterminated escape in string");
                            self.advance();
                            bytes.push(match esc {
                                'n' => b'\n',
                                't' => b'\t',
                                'r' => b'\r',
                                '0' => 0,
                                '\\' => b'\\',
                                '"' => b'"',
                                '\'' => b'\'',
                                other => panic!("unknown escape: \\{}", other),
                            });
                        }
                        Some(c) => {
                            self.advance();
                            if c as u32 > 255 {
                                panic!("error message but string");
                            }
                            bytes.push(c as u8);
                        }
                    }
                }
                Token::Value(Value::Str(bytes))
            }

            Some('#') => {
                loop {
                    match self.current() {
                        Some(c) => {
                            if c == '\n' {
                                break;
                            }

                            self.advance();
                        }
                        None => break,
                    }
                }

                self.next_token()
            }
            Some(c) if c.is_ascii_digit() => {
                let mut number = String::new();

                loop {
                    match self.current() {
                        Some(c) => {
                            if c.is_ascii_digit() {
                                number.push(c);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        None => break,
                    }
                }

                Token::Value(Value::Int(number.parse().unwrap()))
            }

            Some(c) if c.is_alphabetic() => {
                let mut name = String::new();

                loop {
                    match self.current() {
                        Some(c) => {
                            if c.is_alphanumeric() {
                                name.push(c);
                                self.advance();
                            } else {
                                break;
                            }
                        }
                        None => break,
                    }
                }

                match check_keyword(&name) {
                    Some(tok) => tok.clone(),
                    None => Token::Name(name),
                }
            }

            Some(c) => {
                panic!("unexpected char: {}", c);
            }
        }
    }
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();

            if token == Token::Eof {
                tokens.push(Token::Eof);
                break;
            }

            tokens.push(token);
        }

        tokens
    }
}
