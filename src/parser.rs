use std::process;
use crate::scanner::LocToken;
use crate::ast::*;
use crate::icps;
use icps::*;
use crate::token::Token::{self, *};

macro_rules! cmp {
    ($self:expr, $($types:expr),+) => {{
        let mut matched = false;
        $(
            if $self.check($types) {
                $self.advance();
                matched = true;
            }
        )+
        matched
    }};
}

pub struct Parser<'a> {
    tokens: &'a Vec<LocToken>,
    cur: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a Vec<LocToken>) -> Parser<'a> {
        Parser {
            tokens,
            cur: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();
        while !self.is_at_end() {
            if cmp!(*self, Semicolon, Newline) {
                continue;
            }
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => return Err(e)
            }
        }
        Ok(statements)
    }

    fn advance(&mut self) -> LocToken {
        if !self.is_at_end() {
            self.cur += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token == Eof
    }

    fn peek(&self) -> LocToken {
        if self.cur >= self.tokens.len() {
            process::exit(0);
        }
        self.tokens[self.cur].clone()
    }

    fn previous(&self) -> LocToken {
        self.tokens[self.cur - 1].clone()
    }

    fn check(&self, token: Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token == token
        }
    }

    fn consume(&mut self, token: Token) -> Result<LocToken, Error> {
        if self.check(token.clone()) {
            Ok(self.advance())
        } else {
            Err(Error::new(self.peek().loc, format!("Expected closing '{}' after statement.", token).as_str()))
        }
    }

    fn end_statement_if_not_else(&mut self) -> Result<(), Error> {
        if cmp!(*self, Semicolon, Newline, Eof) || self.peek().token == Else {
            Ok(())
        } else {
            Err(Error::new(self.peek().loc, "Expected ';' or newline after statement."))
        }
    }

    fn end_statement(&mut self) -> Result<(), Error> {
        if cmp!(*self, Semicolon, Newline, Eof) {
            Ok(())
        } else {
            Err(Error::new(self.peek().loc, "Expected ';' or newline after statement."))
        }
    }

    fn declaration(&mut self) -> Result<Stmt, Error> {
        if cmp!(self, Var) {
            self.variable()
        } else {
            self.statement()
        }
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        if cmp!(*self, If) {
            self.if_statement()
        } else if cmp!(*self, Log) {
            self.log()
        } else if cmp!(self, While) {
            self.while_loop()
        } else if cmp!(*self, For) {
            self.for_loop()
        } else if cmp!(*self, LeftBrace) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        let condition = self.expression()?;
        cmp!(*self, Newline);
        let then_branch = self.statement()?;
        cmp!(*self, Newline);
        let else_branch = if cmp!(*self, Else) {
            cmp!(*self, Newline);
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If(Box::new(condition), Box::new(then_branch), else_branch))
    }

    fn while_loop(&mut self) -> Result<Stmt, Error> {
        let condition = self.expression()?;
        cmp!(*self, Newline);
        let body = Box::new(self.statement()?);
        Ok(Stmt::While(Box::new(condition), body))
    }

    fn for_loop(&mut self) -> Result<Stmt, Error> {
        let var = match self.peek().token {
            Identifier(_) => {
                self.advance();
                Some(self.previous())
            }
            _ => None
        };
        let iterable = Box::new(self.range()?);
        cmp!(*self, Newline);
        let body = Box::new(self.statement()?);
        Ok(Stmt::For(var, iterable, body))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements: Vec<Stmt> = Vec::new();

        while self.check(Newline) {
            self.advance();
        }

        while !self.check(RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?)
        }

        self.consume(RightBrace)?;
        Ok(statements)
    }

    fn variable(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(Identifier("".to_string()))?;
        let mut initializer = None;

        if cmp!(*self, Equal) {
            initializer = Some(Box::new(self.expression()?));
        }

        self.end_statement()?;
        Ok(Stmt::Declaration(name, initializer))
    }

    fn log(&mut self) -> Result<Stmt, Error> {
        let out = Ok(Stmt::Log(Box::new(self.expression()?)));
        self.end_statement_if_not_else()?;
        out
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let out = Ok(Stmt::Expression(Box::new(self.expression()?)));
        self.end_statement_if_not_else();
        out
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.or()?;
        if cmp!(*self, Equal) {
            let equals = self.previous();
            let value = self.assignment()?;
            match expr {
                Expr::Variable(name) => Ok(Expr::Assign(name, Box::new(value))),
                _ => Err(Error::new(equals.loc, "Invalid assignment target."))
            }
        } else {
            Ok(expr)
        }
    }

    fn or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.and()?;

        while cmp!(*self, Or, Xor) {
            let op = self.previous();
            let right = self.and()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while cmp!(*self, And) {
            let op = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr;
        match self.comparison() {
            Ok(left) => {
                expr = left;
                while cmp!(*self, BangEqual, EqualEqual) {
                    let op = self.previous();
                    expr = Expr::Binary(Box::new(expr), op, Box::new(self.comparison()?));
                }
                Ok(expr)
            }
            Err(e) => Err(e)
        }
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr;
        match self.term() {
            Ok(left) => {
                expr = left;
                while cmp!(*self, Greater, GreaterEqual, Less, LessEqual) {
                    let op = self.previous();
                    match self.term() {
                        Ok(right) => expr = Expr::Binary(Box::new(expr), op, Box::new(right)),
                        Err(e) => return Err(e)
                    }
                }
                Ok(expr)
            }
            Err(e) => Err(e)
        }
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr;
        match self.factor() {
            Ok(left) => {
                expr = left;
                while cmp!(*self, Minus, Plus) {
                    let op = self.previous();
                    match self.factor() {
                        Ok(right) => expr = Expr::Binary(Box::new(expr), op, Box::new(right)),
                        Err(e) => return Err(e)
                    }
                }
                Ok(expr)
            }
            Err(e) => Err(e)
        }
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr;
        match self.range() {
            Ok(left) => {
                expr = left;
                while cmp!(*self, Slash, Star) {
                    let op = self.previous();
                    match self.unary() {
                        Ok(right) => expr = Expr::Binary(Box::new(expr), op, Box::new(right)),
                        Err(e) => return Err(e)
                    }
                }
                Ok(expr)
            }
            Err(e) => Err(e)
        }
    }

    fn range(&mut self) -> Result<Expr, Error> {
        let mut expr;
        match self.unary() {
            Ok(left) => {
                expr = left;
                while cmp!(*self, Range) {
                    let op = self.previous();
                    match self.unary() {
                        Ok(right) => expr = Expr::Binary(Box::new(expr), op, Box::new(right)),
                        Err(e) => return Err(e)
                    }
                }
                Ok(expr)
            }
            Err(e) => Err(e)
        }
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if cmp!(*self, Bang, Minus) {
            let op = self.previous();
            match self.unary() {
                Ok(right) => Ok(Expr::Unary(op, Box::new(right))),
                Err(e) => Err(e)
            }
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        let token = self.peek().clone();
        match token.token {
            False | True | Null | Number(_) | String(_) => {
                self.advance();
                Ok(Expr::Literal(token))
            }
            Identifier(_) => {
                self.advance();
                Ok(Expr::Variable(token))
            }
            LeftParen => {
                self.advance();
                match self.expression() {
                    Ok(expr) => {
                        match self.consume(RightParen) {
                            Ok(_) => Ok(Expr::Grouping(Box::new(expr))),
                            Err(e) => Err(e)
                        }
                    }
                    Err(e) => Err(e)
                }
            }
            _ => {
                Err(Error::new(token.loc, format!("Expected expression, but found '{}'.", token.token).as_str()))
            }
        }
    }


// fn synchronize(&mut self) {
//     self.advance();
//     while !self.is_at_end() {
//         if self.previous().token.kind() == Semicolon {
//             return;
//         }
//         match self.peek().token.kind() {
//             Class | Fn | While | Return => return,
//             _ => self.advance()
//         };
//     }
// }
}