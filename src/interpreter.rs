use crate::ast::{Expr, Stmt};
use crate::environment::Environment;
use crate::icps::Error;
use crate::token::{Bang, BangEqual, EqualEqual, Greater, GreaterEqual, Less, LessEqual, Minus, Plus, Slash, Star, Value};

pub struct Interpreter {
    pub env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter { env: Environment::new() }
    }

    pub fn interpret(&mut self, stmts: Vec<Stmt>) -> Result<(), Error> {
        for stmt in stmts {
            self.execute(&stmt)?;
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, Error> {
        match expr {
            Expr::Literal(token) => {
                if token.token.is_valid_value() {
                    Ok(Value::from(token.token.clone()))
                } else {
                    Err(Error::new(token.loc, "Runtime Error: Invalid literal value."))
                }
            },
            Expr::Grouping(e) => self.evaluate(&*e),
            Expr::Unary(op, re) => {
                let right;
                if let Ok(res) = self.evaluate(&*re) {
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
                match self.evaluate(&*le) {
                    Ok(l) => left = l,
                    Err(e) => return Err(e)
                }
                match self.evaluate(&*re) {
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
                        let comparison = match (&left, &right) {
                            (Value::Number(l), Value::Number(r)) => l == r,
                            (Value::String(l), Value::String(r)) => l == r,
                            (Value::Boolean(l), Value::Boolean(r)) => l == r,
                            (Value::Null, Value::Null) => true,
                            _ => false
                        };

                        Ok(Value::Boolean(if op.token == EqualEqual { comparison } else { !comparison }))
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
            },
            Expr::Variable(token) => self.env.get(token.clone()),
            Expr::Assign(token, value) => {
                let value = self.evaluate(&*value)?;
                self.env.assign(token.clone(), value.clone())?;
                Ok(value)
            },
            _ => panic!("{}", expr.to_string())
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<Value, Error> {
        match stmt {
            Stmt::Expression(e) => self.evaluate(&*e),
            Stmt::Log(e) => {
                match self.evaluate(&*e) {
                    Ok(v) => {
                        println!("{}", v);
                        Ok(v)
                    },
                    Err(e) => Err(e)
                }
            },
            Stmt::Variable(name, initializer) => {
                let value = match initializer {
                    Some(i) => self.evaluate(&*i)?,
                    None => Value::Null
                };
                self.env.define(name.clone(), value);
                Ok(Value::Null)
            },
            Stmt::Block(stmts) => {
                let mut new_env = Environment::new_local(self.env.clone());
                let previous = self.env.clone();
                self.env = new_env;
                let mut out = Ok(Value::Null);
                for stmt in stmts {
                    out = self.execute(&*stmt);
                }
                self.env = previous;
                out
            },
            _ => panic!()
        }
    }
}