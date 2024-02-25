use std::collections::HashMap;
use std::fmt::{Display, format};
use std::string::String;
use lazy_static::lazy_static;

#[derive(Clone, Debug)]
pub enum Token {
    // Single-character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    At,
    Comma,
    Plus,
    Minus,
    Slash,
    Star,
    Semicolon,
    Newline,
    QuestionMark,
    Colon,

    // Comparisons
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords
    And,
    Or,
    Xor,
    Not,
    True,
    False,
    Class,
    Else,
    While,
    Null,
    Log,
    Return,
    Super,
    This,
    Fn,
    Use,
    Var,

    // Special
    EOF,
    UnterminatedString,
    Unknown(char),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            LeftParen => "(".to_string(),
            RightParen => ")".to_string(),
            LeftBrace => "{".to_string(),
            RightBrace => "}".to_string(),
            At => "@".to_string(),
            Comma => ",".to_string(),
            Plus => "+".to_string(),
            Minus => "-".to_string(),
            Slash => "/".to_string(),
            Star => "*".to_string(),
            Semicolon => ";".to_string(),
            Newline => "\\n".to_string(),
            QuestionMark => "?".to_string(),
            Colon => ":".to_string(),
            Bang => "!".to_string(),
            BangEqual => "!=".to_string(),
            Equal => "=".to_string(),
            EqualEqual => "==".to_string(),
            Greater => ">".to_string(),
            GreaterEqual => ">=".to_string(),
            Less => "<".to_string(),
            LessEqual => "<=".to_string(),
            Identifier(s) => s.to_string(),
            String(s) => s.to_string(),
            Number(n) => n.to_string(),
            And => "and".to_string(),
            Or => "or".to_string(),
            Xor => "xor".to_string(),
            Not => "not".to_string(),
            True => "true".to_string(),
            False => "false".to_string(),
            Class => "class".to_string(),
            Else => "else".to_string(),
            While => "while".to_string(),
            Null => "null".to_string(),
            Log => "log".to_string(),
            Return => "return".to_string(),
            Super => "super".to_string(),
            This => "this".to_string(),
            Fn => "fn".to_string(),
            Use => "use".to_string(),
            Var => "var".to_string(),
            EOF => "EOF".to_string(),
            UnterminatedString => "unterminated string".to_string(),
            Unknown(c) => "unknown".to_string()
        })
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        matches!((self, other),
        (Identifier(_), Identifier(_)) |
        (String(_), String(_)) |
        (Number(_), Number(_)) |
        (Unknown(_), Unknown(_)) |
        (LeftParen, LeftParen) |
        (RightParen, RightParen) |
        (LeftBrace, LeftBrace) |
        (RightBrace, RightBrace) |
        (At, At) |
        (Comma, Comma) |
        (Plus, Plus) |
        (Minus, Minus) |
        (Slash, Slash) |
        (Star, Star) |
        (Semicolon, Semicolon) |
        (Newline, Newline) |
        (QuestionMark, QuestionMark) |
        (Colon, Colon) |
        (Bang, Bang) |
        (BangEqual, BangEqual) |
        (Equal, Equal) |
        (EqualEqual, EqualEqual) |
        (Greater, Greater) |
        (GreaterEqual, GreaterEqual) |
        (Less, Less) |
        (LessEqual, LessEqual) |
        (And, And) |
        (Or, Or) |
        (Xor, Xor) |
        (Not, Not) |
        (True, True) |
        (False, False) |
        (Class, Class) |
        (Else, Else) |
        (While, While) |
        (Null, Null) |
        (Log, Log) |
        (Return, Return) |
        (Super, Super) |
        (This, This) |
        (Fn, Fn) |
        (Use, Use) |
        (Var, Var) |
        (EOF, EOF) |
        (UnterminatedString, UnterminatedString)
    )
    }


    fn ne(&self, other: &Self) -> bool {
        !(self == other)
    }
}

impl Token {
    pub fn is_valid_value(&self) -> bool {
        match self {
            Number(_) | String(_) | True | False | Null => true,
            _ => false
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null")
        }
    }
}

impl From<Token> for Value {
    fn from(token: Token) -> Self {
        match token {
            Number(n) => Value::Number(n),
            String(s) => Value::String(s),
            True => Value::Boolean(true),
            False => Value::Boolean(false),
            Null => Value::Null,
            _ => {
                icps::panic("token.rs, in the function 'impl From<Token> for Value' -> Attempted to convert a non-value 'Token' into a 'Value'.");
                panic!()
            }
        }
    }
}

impl Value {
    pub fn as_str(&self) -> String {
        match self {
            Value::String(s) => s.to_string(),
            _ => panic!()
        }
    }

    pub fn as_num(&self) -> f64 {
        match self {
            Value::Number(n) => *n,
            _ => panic!()
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            _ => panic!()
        }
    }

    pub fn is_truthy(&self) -> bool {
        match *self {
            Value::Boolean(b) => b,
            Value::Null => false,
            _ => panic!("Attempted to call is_truthy on non-(boolean/null) 'Value'. Please report this, this wasn't supposed to happen.")
        }
    }
}


lazy_static! {
    pub static ref KEYWORDS: HashMap<&'static str, Token> = {
        let mut m = HashMap::new();
        m.insert("and", And);
        m.insert("or", Or);
        m.insert("xor", Xor);
        m.insert("not", Not);
        m.insert("true", True);
        m.insert("false", False);
        m.insert("class", Class);
        m.insert("else", Else);
        m.insert("while", While);
        m.insert("null", Null);
        m.insert("log", Log);
        m.insert("return", Return);
        m.insert("super", Super);
        m.insert("this", This);
        m.insert("fn", Fn);
        m.insert("use", Use);
        m.insert("var", Var);
        m
    };
}

pub use Token::*;
use crate::icps;