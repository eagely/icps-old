use crate::ast::*;
use crate::icps::Error;

#[macro_export]
macro_rules! parenthesize {
    ($name:expr, $( $expr:expr ),* ) => {{
        let mut s = String::new();
        s.push_str(&$name.to_string());
        $(
            s.push(' ');
            match $expr.accept() {
                Ok(r) => s.push_str(&r.to_string()),
                Err(e) => return Err(e),
            }
        )*
        Ok(s)
    }};
}

pub fn print(expr: Expr) -> Result<(), Error> {
    let result = parenthesize!("", expr);
    match result {
        Ok(s) => {
            println!("{}", s);
            Ok(())
        },
        Err(e) => Err(e),
    }
}