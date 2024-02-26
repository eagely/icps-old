use std::iter::Peekable;
use std::str::Chars;
use crate::token::*;
use crate::icps;
use crate::icps::Error;

#[derive(Clone, Copy, Debug)]
pub struct Loc {
    pub line: usize,
    pub col: usize,
    pub idx: usize,
}

#[derive(Clone, Debug)]
pub struct LocToken {
    pub token: Token,
    pub loc: Loc,
}

pub struct Scanner<'a> {
    it: Peekable<Chars<'a>>,
    tokens: Vec<LocToken>,
    cur: Loc,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            it: source.chars().peekable(),
            tokens: Vec::new(),
            cur: Loc {
                line: 1,
                col: 1,
                idx: 0,
            },
        }
    }

    fn peek(&mut self) -> Option<&char> {
        self.it.peek()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.it.next();
        self.cur.idx += 1;
        match c {
            Some('\n') => {
                self.cur.line += 1;
                self.cur.col = 1;
            }
            Some(_) => {
                self.cur.col += 1;
            }
            None => {}
        };
        c
    }

    fn emit(&mut self, token: Token) {
        self.tokens.push(LocToken {
            token,
            loc: self.cur,
        });
    }

    pub fn scan(&mut self) -> Result<Vec<LocToken>, Error> {
        while let Some(c) = self.it.next() {
            match match c {
                ' ' => {
                    self.cur.col += 1;
                    self.cur.idx += 1;
                    continue
                }
                '\r' => continue,
                '\t' => {
                    self.cur.col += 1;
                    self.cur.idx += 1;
                    continue
                }
                '\n' => {
                    self.cur.line += 1;
                    self.cur.col = 1;
                    self.cur.idx += 1;
                    Ok(Newline)
                },
                '(' => Ok(LeftParen),
                ')' => Ok(RightParen),
                '{' => Ok(LeftBrace),
                '}' => Ok(RightBrace),
                '@' => Ok(At),
                ',' => Ok(Comma),
                '+' => Ok(Plus),
                '-' => Ok(Minus),
                '/' => match self.peek() {
                    Some('/') => {
                        while let Some(c) = self.next() {
                            if c == '\n' {
                                break;
                            }
                        }
                        Ok(Newline)
                    }
                    Some('*') => {
                        while let Some(c) = self.next() {
                            if c == '*' {
                                if let Some('/') = self.peek() {
                                    self.next();
                                    break;
                                }
                            }
                        }
                        continue;
                    }
                    _ => Ok(Slash)
                },
                '*' => Ok(Star),
                ';' => {
                    if self.peek() == Some(&'\n') {
                        icps::warn(self.cur.line, self.cur.col, "Redundant semicolon.");
                    }
                    Ok(Semicolon)
                },
                '?' => Ok(QuestionMark),
                ':' => Ok(Colon),
                '!' => match self.peek() {
                    Some('=') => {
                        self.next();
                        Ok(BangEqual)
                    }
                    _ => Ok(Bang)
                },
                '=' => match self.peek() {
                    Some('=') => {
                        self.next();
                        Ok(EqualEqual)
                    }
                    _ => Ok(Equal)
                },
                '>' => match self.peek() {
                    Some('=') => {
                        self.next();
                        Ok(GreaterEqual)
                    }
                    _ => Ok(Greater)
                },
                '<' => match self.peek() {
                    Some('=') => {
                        self.next();
                        Ok(LessEqual)
                    }
                    _ => Ok(Less)
                },
                '"' => {
                    self.string(c)
                }
                // ew
                '.' => {
                    match self.peek() {
                        Some(&'.') => {
                            self.next();
                            Ok(Range)
                        },
                        Some(c) if c.is_digit(10) => {
                            let decimal_part = self.collect_decimal_part();

                            if let Some(LocToken { token: Token::Number(number), .. }) = self.tokens.last_mut() {
                                let whole_number = *number;
                                let decimal_number = format!("{}{}", whole_number, decimal_part).parse().unwrap();
                                *number = decimal_number;
                                continue;
                            } else {
                                return Err(Error::new(self.cur, "Unexpected character '.'"));
                            }
                        },
                        _ => {
                            Err(Error::new(self.cur, "Unexpected character '.'"))
                        }
                    }
                },
                _ => {
                    if c.is_digit(10) {
                        self.number(c)
                    } else if c.is_alphanumeric() {
                        Ok(self.identifier(c))
                    } else {
                        Err(Error::new(self.cur, format!("Unexpected character {}", c).as_str()))
                    }
                }
            } {
                Ok(token) => self.emit(token),
                Err(e) => return Err(e)
            }
        }
        self.emit(Eof);
        Ok(self.tokens.to_owned())
    }

    pub fn string(&mut self, c: char) -> Result<Token, Error> {
        let mut s = String::new();
        while let Some(c) = self.next() {
            if c == '"' {
                break;
            }
            s.push(c);
        }

        if c == '"' {
            Ok(String(s))
        } else {
            Err(Error::new(self.cur, "Unterminated string"))
        }
    }

    pub fn number(&mut self, start_char: char) -> Result<Token, Error> {
        let mut number_string = String::new();
        number_string.push(start_char);

        while let Some(&next_char) = self.peek() {
            if next_char.is_digit(10) {
                number_string.push(self.next().unwrap());
            } else {
                break;
            }
        }

        let number = number_string.parse().map_err(|_| Error::new(self.cur, "Invalid number"))?;
        Ok(Number(number))
    }

    fn collect_decimal_part(&mut self) -> String {
        let mut decimal_part = String::new();
        decimal_part.push('.');
        while let Some(&next_char) = self.peek() {
            if next_char.is_digit(10) {
                decimal_part.push(self.next().unwrap());
            } else {
                break;
            }
        }
        decimal_part
    }

    pub fn identifier(&mut self, c: char) -> Token {
        let mut s = String::new();
        s.push(c);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() {
                s.push(self.next().unwrap());
            } else {
                break;
            }
        }
        match KEYWORDS.get(&s.as_str()) {
            Some(token) => token.to_owned(),
            None => Identifier(s)
        }
    }
}