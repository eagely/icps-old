use std::cell::RefCell;
use std::rc::Rc;
use crate::ast::{Expr, Stmt};
use crate::environment::Environment;
use crate::icps::Error;
use crate::scanner::{Loc, LocToken};
use crate::token::{Token::{self, *}, Value};

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
            }
            Expr::Grouping(e) => self.evaluate(e),
            Expr::Unary(op, re) => {
                let right = self.evaluate(re)?;
                match op.token {
                    Minus => {
                        if let Value::Number(n) = right {
                            Ok(Value::Number(-n))
                        } else {
                            Err(Error::new(op.loc, "Runtime Error: Cannot negate non 'Number' expression."))
                        }
                    }

                    Bang => {
                        if let Value::Boolean(b) = right {
                            Ok(Value::Boolean(!b))
                        } else {
                            Err(Error::new(op.loc, "Runtime Error: Cannot negate non 'Boolean' expression."))
                        }
                    }

                    _ => Err(Error::new(op.loc, "Runtime Error: Invalid unary operator"))
                }
            }
            Expr::Binary(le, op, re) => {
                let left = self.evaluate(le)?;
                let right = self.evaluate(re)?;
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
                    }

                    Star => {
                        match left {
                            Value::Number(l) => {
                                match right {
                                    Value::Number(r) => Ok(Value::Number(l * r)),
                                    Value::String(r) => Ok(Value::String(r.repeat(l.round() as usize))),
                                    _ => Err(Error::new(op.loc, "Runtime Error: Cannot multiply 'Number' with anything but 'Number' or 'String' or an expression evaluating to it"))
                                }
                            }
                            Value::String(l) => {
                                match right {
                                    Value::Number(r) => Ok(Value::String(l.repeat(r.round() as usize))),
                                    _ => Err(Error::new(op.loc, "Runtime Error: Cannot multiply 'String' with anything but 'Number' or an expression evaluating to it"))
                                }
                            }
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot multiply anything with a non 'Number' or 'String' expression."))
                        }
                    }

                    Slash => {
                        match right {
                            Value::Number(r) => {
                                if r == 0.0 {
                                    Err(Error::new(op.loc, "Runtime Error: Division by zero."))
                                } else {
                                    match left {
                                        Value::Number(l) => Ok(Value::Number(l / r)),
                                        _ => Err(Error::new(op.loc, "Runtime Error: Cannot divide anything but 'Number' or an expression evaluating to it"))
                                    }
                                }
                            }
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot divide by anything but 'Number' or an expression evaluating to it"))
                        }
                    }

                    Range => {
                        match left {
                            Value::Number(l) => {
                                match right {
                                    Value::Number(r) => Ok(Value::Range(l, r)),
                                    _ => Err(Error::new(op.loc, "Runtime Error: Cannot create a range with anything but 'Number' or an expression evaluating to it"))
                                }
                            }
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot create a range with anything but 'Number' or an expression evaluating to it"))
                        }
                    }

                    EqualEqual | BangEqual => {
                        let comparison = match (&left, &right) {
                            (Value::Number(l), Value::Number(r)) => l == r,
                            (Value::String(l), Value::String(r)) => l == r,
                            (Value::Boolean(l), Value::Boolean(r)) => l == r,
                            (Value::Null, Value::Null) => true,
                            _ => false
                        };

                        Ok(Value::Boolean(if op.token == EqualEqual { comparison } else { !comparison }))
                    }

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
                            }
                            _ => Err(Error::new(op.loc, "Runtime Error: Cannot compare anything with a non 'Number' expression."))
                        }
                    }

                    _ => Err(Error::new(op.loc, "Runtime Error: Invalid binary operator"))
                }
            }

            Expr::Variable(token) => {
                match self.env.get(token)? {
                    Value::Null => Err(Error::new(token.loc, format!("Runtime Error: Cannot use variable '{}' before assignment.", token.token).as_str())),
                    v => Ok(v)
                }
            }

            Expr::Assign(token, value) => {
                let value = self.evaluate(value)?;
                self.env.assign(token.clone(), value.clone())?;
                Ok(value)
            }

            Expr::Logical(le, op, re) => {
                let left = self.evaluate(le)?.is_truthy();
                if op.token == Or && left {
                    return Ok(Value::Boolean(true));
                }
                if op.token == And && !left {
                    return Ok(Value::Boolean(false));
                }
                let right = self.evaluate(re)?.is_truthy();
                match op.token {
                    Or => Ok(Value::Boolean(left || right)),
                    Xor => Ok(Value::Boolean(left ^ right)),
                    And => Ok(Value::Boolean(left && right)),
                    _ => Err(Error::new(op.loc, "Runtime Error: Invalid logical operator. How did you do that bro?"))
                }
            }

            Expr::Call(_, _) => {
                println!("Warning: Called evaluate expr on a non implemented operation!");
                Ok(Value::Null)
            }

            Expr::Get(_, _) => {
                println!("Warning: Called evaluate expr on a non implemented operation!");
                Ok(Value::Null)
            }

            Expr::Set(_, _, _) => {
                println!("Warning: Called evaluate expr on a non implemented operation!");
                Ok(Value::Null)
            }

            Expr::Super(_) => {
                println!("Warning: Called evaluate expr on a non implemented operation!");
                Ok(Value::Null)
            }

            Expr::This(_) => {
                println!("Warning: Called evaluate expr on a non implemented operation!");
                Ok(Value::Null)
            }
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<Value, Error> {
        match stmt {
            Stmt::Expression(e) => self.evaluate(e),
            Stmt::Log(e) => {
                match self.evaluate(e) {
                    Ok(v) => {
                        println!("{}", v);
                        Ok(v)
                    }
                    Err(e) => Err(e)
                }
            }

            Stmt::Declaration(name, initializer) => {
                let value = match initializer {
                    Some(i) => self.evaluate(i)?,
                    None => Value::Null
                };
                self.env.define(name, value);
                Ok(Value::Null)
            }

            Stmt::Block(stmts) => {
                let previous = Rc::new(RefCell::new(self.env.clone()));
                self.env = Environment::new_local(previous.clone());
                let mut out = Ok(Value::Null);
                for stmt in stmts {
                    out = self.execute(stmt);
                }
                self.env = previous.borrow().clone();
                out
            }

            Stmt::If(condition, then_branch, else_branch) => {
                match self.evaluate(condition)? {
                    Value::Boolean(b) => {
                        if b {
                            self.execute(then_branch)
                        } else {
                            match else_branch {
                                Some(e) => self.execute(e),
                                None => Ok(Value::Null)
                            }
                        }
                    }
                    _ => Err(Error::new(Self::get_loc_token_from_expr(condition).loc, "Runtime Error: Invalid condition."))
                }
            }

            Stmt::While(condition, body) => {
                while self.evaluate(condition)?.is_truthy() {
                    self.execute(body)?;
                }
                Ok(Value::Null)
            }

            Stmt::For(name, iterable, body) => {
                let range = match self.evaluate(iterable)? {
                    Value::Range(l, r) => l..r,
                    _ => return Err(Error::new(Self::get_loc_token_from_expr(iterable).loc, "Runtime Error: For loop iterable must be a range."))
                };
                let previous = Rc::new(RefCell::new(self.env.clone()));
                self.env = Environment::new_local(previous.clone());
                let mut actual = match name {
                    Some(n) => n.clone(),
                    None => LocToken {
                        token: Identifier("i".to_string()),
                        loc: Self::get_loc_token_from_expr(iterable).loc,
                    },
                };
                self.env.define(&actual, Value::Number(range.start));
                match self.env.get(&actual).unwrap() {
                    Value::Number(_) => {}
                    _ => return Err(Error::new(Self::get_loc_token_from_expr(iterable).loc, "Runtime Error: For loop variable must be a number.")),
                }
                /*
                Rn this iterates over the range no matter what but if the variable is modified within it to be larger than the range then it doesnt care and keeps going
                Either fix this or make i immutable
                 */
                while let Value::Number(i) = self.env.get(&actual).unwrap() {
                    self.execute(body)?;
                    if let Value::Number(i) = self.env.get(&actual).unwrap() {
                        self.env.assign(actual.clone(), Value::Number(i + 1.0))?;
                    } else {
                        return Err(Error::new(Self::get_loc_token_from_expr(iterable).loc, "Runtime Error: For loop variable must be a number."));
                    }
                    if let Value::Number(new_i) = self.env.get(&actual).unwrap() {
                        if new_i >= range.end {
                            break;
                        }
                    } else {
                        return Err(Error::new(Self::get_loc_token_from_expr(iterable).loc, "Runtime Error: For loop variable must be a number."));
                    }
                }
                self.execute(body)?;
                self.env = previous.borrow().clone();
                Ok(Value::Null)
            }

            _ => {
                Err(Error::new(Loc { line: 0, col: 0, idx: 0 }, "Runtime Error: Not Implemented."))
            }
        }
    }

    fn get_loc_token_from_expr(expr: &Expr) -> LocToken {
        match expr {
            Expr::Literal(token) => token.clone(),
            Expr::Grouping(e) => Self::get_loc_token_from_expr(e),
            Expr::Assign(token, _) => token.clone(),
            Expr::Variable(token) => token.clone(),
            Expr::Literal(token) => token.clone(),
            Expr::Unary(token, _) => token.clone(),
            Expr::Get(_, token) => token.clone(),
            Expr::Set(_, token, _) => token.clone(),
            Expr::Logical(_, token, _) => token.clone(),
            Expr::Super(token) => token.clone(),
            Expr::This(token) => token.clone(),
            Expr::Variable(token) => token.clone(),
            Expr::Binary(_, token, _) => token.clone(),
            Expr::Call(_, _) => panic!()
        }
    }
}