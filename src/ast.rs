use crate::token::{Token::{self, *}, Value};
use crate::scanner::LocToken;
use std::fmt::{Display, format, Formatter};
use crate::icps::Error;

pub enum Expr {
    Assign(LocToken, Box<Expr>),
    Unary(LocToken, Box<Expr>),
    Binary(Box<Expr>, LocToken, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    // function name, args
    Get(Box<Expr>, LocToken),
    Set(Box<Expr>, LocToken, Box<Expr>),
    // object, name, value
    Grouping(Box<Expr>),
    // expr in parentheses
    Literal(LocToken),
    // num or string
    Logical(Box<Expr>, LocToken, Box<Expr>),
    Super(LocToken),
    // super.name
    This(LocToken),
    // this.name
    Variable(LocToken), // variable name
}

pub enum Stmt {
    Block(Vec<Box<Stmt>>),
    Class(LocToken, Box<Expr>, Vec<Box<Stmt>>),
    // name, superclass, methods
    Expression(Box<Expr>),
    Function(LocToken, Vec<LocToken>, Vec<Box<Stmt>>),
    // name, params, body
    If(Box<Expr>, Box<Stmt>, Box<Stmt>),
    // condition, then, else
    Log(Box<Expr>),
    Return(Option<Expr>),
    Variable(LocToken, Option<Box<Expr>>),
    // name, initializer (icps is kotlin-like not nullable)
    While(Box<Expr>, Box<Stmt>), // condition, body
}

impl Display for Stmt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Stmt::Class(name, superclass, methods) => {
                let methods_str = methods.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
                format!("class {} {} {}", name.token, superclass, methods_str)
            }
            Stmt::Expression(expr) => format!("{}", expr),
            Stmt::If(condition, then, else_) => format!("if {} then {} else {}", condition, then, else_),
            Stmt::Log(expr) => format!("log {}", expr),
            Stmt::Return(expr) => format!("return {}", expr.as_ref().map_or("".to_string(), ToString::to_string)),
            Stmt::Variable(name, initializer) => format!("var {} = {}", name.token, initializer.as_ref().map_or("".to_string(), ToString::to_string)),
            Stmt::While(condition, body) => format!("while {} {}", condition, body),
            _ => panic!()
        })
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Expr::Assign(name, value) => format!("{} {}", name.token, value),
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
            Expr::Variable(name) => format!("Vvar {}", name.token),
        })
    }
}