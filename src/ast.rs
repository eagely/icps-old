use crate::token::{Token::{self, *}, Value};
use crate::scanner::LocToken;
use std::fmt::{Display, format, Formatter};
use crate::icps::Error;

pub enum Expr {
    Assign(LocToken, Box<Expr>),
    Unary(LocToken, Box<Expr>),
    Binary(Box<Expr>, LocToken, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Get(Box<Expr>, LocToken),
    Set(Box<Expr>, LocToken, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(LocToken),
    Logical(Box<Expr>, LocToken, Box<Expr>),
    Super(LocToken),
    This(LocToken),
    Variable(LocToken),
}

pub enum Stmt {
    Block(Vec<Stmt>),
    Class(LocToken, Box<Expr>, Vec<Box<Stmt>>),
    Expression(Box<Expr>),
    Function(LocToken, Vec<LocToken>, Vec<Box<Stmt>>),
    If(Box<Expr>, Box<Stmt>, Option<Box<Stmt>>),
    Log(Box<Expr>),
    Return(Option<Expr>),
    Declaration(LocToken, Option<Box<Expr>>),
    While(Box<Expr>, Box<Stmt>),
    For(Option<LocToken>, Box<Expr>, Box<Stmt>),
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Stmt::Class(name, superclass, methods) => {
                let methods_str = methods.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
                format!("class {} {} {}", name.token, superclass, methods_str)
            }
            Stmt::Expression(expr) => format!("{}", expr),
            Stmt::If(condition, then, else_) => format!("if {} then {} else {}", condition, then, match else_ { Some(else_) => format!("{}", else_), None => "Nothing".to_string() }),
            Stmt::Log(expr) => format!("log {}", expr),
            Stmt::Return(expr) => format!("return {}", expr.as_ref().map_or("".to_string(), ToString::to_string)),
            Stmt::Declaration(name, initializer) => format!("var {} = {}", name.token, initializer.as_ref().map_or("".to_string(), ToString::to_string)),
            Stmt::While(condition, body) => format!("while {} {}", condition, body),
            Stmt::Block(stmts) => {
                let stmts_str = stmts.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
                format!("block [ {} ]", stmts_str)
            }
            _ => panic!()
        })
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Expr::Assign(name, value) => format!("{} = {}", name.token, value),
            Expr::Unary(operator, right) => format!("{} {}", operator.token, right),
            Expr::Binary(left, operator, right) => format!("{} {} {}", operator.token, left, right),
            Expr::Call(callee, args) => {
                let args_str = args.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
                format!("Call {} ({})", callee, args_str)
            }
            Expr::Get(object, name) => format!("{}.{}", object, name.token),
            Expr::Set(object, name, value) => format!("{}.{} = {}", object, name.token, value),
            Expr::Grouping(expr) => format!("grouping {}", expr),
            Expr::Literal(value) => format!("{}", value.token),
            Expr::Logical(left, operator, right) => format!("{} {} {}", operator.token, left, right),
            Expr::Super(keyword) => format!("super.{}", keyword.token),
            Expr::This(keyword) => format!("this.{}", keyword.token),
            Expr::Variable(name) => format!("{}", name.token),
        })
    }
}