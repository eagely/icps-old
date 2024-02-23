use crate::token::{Token::{self, *}, Value};
use crate::scanner::LocToken;
use std::fmt::{Display, format};
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
    Class(LocToken, Box<Expr>, Vec<Box<Stmt>>),
    // name, superclass, methods
    Expression(Box<Expr>),
    Function(LocToken, Vec<LocToken>, Vec<Box<Stmt>>),
    // name, params, body
    If(Box<Expr>, Box<Stmt>, Box<Stmt>),
    // condition, then, else
    Log(Box<Expr>),
    Return(Option<Expr>),
    Variable(LocToken, Box<Expr>),
    // name, initializer (icps is kotlin-like not nullable)
    While(Box<Expr>, Box<Stmt>), // condition, body
    Var(LocToken, Option<Expr>),
}

impl Expr {
    pub fn accept(&self) -> Result<Value, Error> {
        match self {
            Expr::Literal(token) => {
                if token.token.is_valid_value() {
                    Ok(Value::from(token.token.clone()))
                } else {
                    Err(Error::new(token.loc, "Runtime Error: Invalid literal value."))
                }
            },
            Expr::Grouping(e) => e.accept(),
            Expr::Unary(op, re) => {
                let right;
                if let Ok(res) = re.accept() {
                    right = res
                } else {
                    return Err(Error::new(op.loc, "Runtime Error: Invalid unary expression."));
                }
                match op.token {
                    Minus => {
                        if let Value::Number(n) = right {
                            Ok(Value::Number(-n))
                        } else {
                            Err(Error::new(op.loc, "Runtime Error: Cannot negate non 'Number' expression."))
                        }
                    },
                    Bang => {
                        if let Value::Boolean(b) = right {
                            Ok(Value::Boolean(!b))
                        } else {
                            Err(Error::new(op.loc, "Runtime Error: Cannot negate non 'Boolean' expression."))
                        }
                    },
                    _ => Err(Error::new(op.loc, "Runtime Error: Invalid unary operator"))
                }
            }
            Expr::Binary(le, op, re) => {
                let left;
                let right;
                match le.accept() {
                    Ok(l) => left = l,
                    Err(e) => return Err(e)
                }
                match re.accept() {
                    Ok(r) => right = r,
                    Err(e) => return Err(e)
                }
                match op.token {
                    Plus => {
                        match left {
                            Value::Number(l) => {
                                match right {
                                    Value::Number(r) => Ok(Value::Number(l + r)),
                                    Value::String(r) => Ok(Value::String(format!("{}{}", l, r))),
                                    _ => Err(Error::new(op.loc, "Runtime Error: Cannot add 'Number' with anything but 'Number' or 'String' or an expression evaluating to it"))
                                }
                            }
                            Value::String(l) => {
                                match right {
                                    Value::Number(r) => Ok(Value::String(format!("{}{}", l, r))),
                                    Value::String(r) => Ok(Value::String(format!("{}{}", l, r))),
                                    _ => Err(Error::new(op.loc, "Runtime Error: Cannot add 'String' with anything but 'String' or 'Number' or an expression evaluating to it"))
                                }
                            }
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot add anything to a non 'Number' or 'String' expression."))
                        }
                    }
                    Minus => {
                        match left {
                            Value::Number(l) => {
                                match right {
                                    Value::Number(r) => Ok(Value::Number(l - r)),
                                    _ => Err(Error::new(op.loc, "Runtime Error: Cannot subtract 'Number' with anything but 'Number' or an expression evaluating to it"))
                                }
                            }
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot subtract anything from a non 'Number' expression."))
                        }
                    },
                    Star => {
                        match left {
                            Value::Number(l) => {
                                match right {
                                    Value::Number(r) => Ok(Value::Number(l * r)),
                                    Value::String(r) => Ok(Value::String(r.repeat(l.round() as usize))),
                                    _ => Err(Error::new(op.loc, "Runtime Error: Cannot multiply 'Number' with anything but 'Number' or 'String' or an expression evaluating to it"))
                                }
                            },
                            Value::String(l) => {
                                match right {
                                    Value::Number(r) => Ok(Value::String(l.repeat(r.round() as usize))),
                                    _ => Err(Error::new(op.loc, "Runtime Error: Cannot multiply 'String' with anything but 'Number' or an expression evaluating to it"))
                                }
                            },
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot multiply anything with a non 'Number' or 'String' expression."))
                        }

                    },
                    Slash => {
                        match right {
                            Value::Number(r) => {
                                if r == 0.0 {
                                    Err(Error::new(op.loc, "Runtime Error: Division by zero."))
                                }
                                else {
                                    match left {
                                        Value::Number(l) => Ok(Value::Number(l / r)),
                                        _ => Err(Error::new(op.loc, "Runtime Error: Cannot divide anything but 'Number' or an expression evaluating to it"))
                                    }
                                }
                            },
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot divide by anything but 'Number' or an expression evaluating to it"))
                        }
                    },
                    EqualEqual | BangEqual => {
                        match right {
                            Value::Boolean(_) | Value::Number(_) | Value::String(_) | Value::Null => {
                                if left == right {
                                    Ok(match op.token {
                                        EqualEqual => Value::Boolean(true),
                                        BangEqual => Value::Boolean(false),
                                        _ => panic!()
                                    })
                                } else {
                                    Err(Error::new(op.loc, "Runtime Error: Cannot compare different types"))
                                }
                            },
                        }
                    },
                    Greater | GreaterEqual | Less | LessEqual => {
                        match left {
                            Value::Number(l) => {
                                if let Value::Number(r) = right {
                                    Ok(match op.token {
                                        Greater => Value::Boolean(l > r),
                                        GreaterEqual => Value::Boolean(l >= r),
                                        Less => Value::Boolean(l < r),
                                        LessEqual => Value::Boolean(l <= r),
                                        _ => panic!()
                                    })
                                } else {
                                    Err(Error::new(op.loc, "Runtime Error: Cannot compare 'Number' with anything but 'Number' or an expression evaluating to it"))
                                }
                            },
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot compare anything with a non 'Number' expression."))
                        }
                    },
                    _ => Err(Error::new(op.loc, "Runtime Error: Invalid binary operator"))
                }
            }
            _ => panic!()
        }
    }
}

impl Stmt {
    pub fn accept(&self) -> Result<Value, Error> {
        match self {
            Stmt::Expression(e) => e.accept(),
            Stmt::Log(e) => {
                match e.accept() {
                    Ok(v) => {
                        println!("{}", v);
                        Ok(v)
                    },
                    Err(e) => Err(e)
                }
            },
            _ => panic!()
        }
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
            Expr::Variable(name) => format!("var {}", name.token),
        })
    }
}