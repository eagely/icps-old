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

    // fn peek_ahead(&mut self, ahead: usize) -> Option<char> {
    //     let mut iter_clone = self.it.clone();
    //     for _ in 0..ahead {
    //         iter_clone.next();
    //     }
    //     iter_clone.next()
    // }

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
                        continue;
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
                _ => {
                    if c.is_digit(10) || c == '.' {
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
        self.emit(EOF);
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

    pub fn number(&mut self, c: char) -> Result<Token, Error> {
        let mut s = String::new();
        s.push(c);
        while let Some(c) = self.peek() {
            if c.is_digit(10) || (c == &'.' && s.chars().filter(|&c| c == '.').count() == 0) {
                s.push(self.next().unwrap());
            } else {
                break;
            }
        }
        if s.chars().filter(|&c| c == '.').count() == s.len() {
            Err(Error::new(self.cur, "Unexpected character '.'"))
        }
        else {
            Ok(Number(s.parse().unwrap()))
        }
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