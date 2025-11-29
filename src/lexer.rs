#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Func,
    Let,
    Return,
    If,
    Else,

    IntType,
    StringType,

    Ident(String),
    Number(i64),
    StringLiteral(String),

    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Semicolon,
    Assign,

    Plus,
    Minus,
    Star,
    Slash,
    Greater,
    Less,
    EqualEqual,
    NotEqual,

    EOF,
}

pub fn lex(input: &str) -> Vec<Token> {
    use Token::*;

    let mut chars = input.chars().peekable();
    let mut tokens = Vec::new();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' | '\r' | '\n' => { chars.next(); }

            '(' => { chars.next(); tokens.push(LParen); }
            ')' => { chars.next(); tokens.push(RParen); }
            '{' => { chars.next(); tokens.push(LBrace); }
            '}' => { chars.next(); tokens.push(RBrace); }
            ',' => { chars.next(); tokens.push(Comma); }
            ':' => { chars.next(); tokens.push(Colon); }
            ';' => { chars.next(); tokens.push(Semicolon); }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(EqualEqual);
                } else {
                    tokens.push(Assign);
                }
            }
            '+' => { chars.next(); tokens.push(Plus); }
            '-' => { chars.next(); tokens.push(Minus); }
            '*' => { chars.next(); tokens.push(Star); }
            '/' => { chars.next(); tokens.push(Slash); }
            '>' => { chars.next(); tokens.push(Greater); }
            '<' => { chars.next(); tokens.push(Less); }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(NotEqual);
                } else {
                    panic!("Unexpected '!'");
                }
            }

            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '"' { break; }
                    s.push(ch);
                }
                tokens.push(StringLiteral(s));
            }

            d if d.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_ascii_digit() {
                        num.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Number(num.parse().unwrap()));
            }

            a if a.is_ascii_alphabetic() || a == '_' => {
                let mut ident = String::new();
                while let Some(&c2) = chars.peek() {
                    if c2.is_ascii_alphanumeric() || c2 == '_' {
                        ident.push(c2);
                        chars.next();
                    } else {
                        break;
                    }
                }

                match ident.as_str() {
                    "func" => tokens.push(Func),
                    "let" => tokens.push(Let),
                    "return" => tokens.push(Return),
                    "if" => tokens.push(If),
                    "else" => tokens.push(Else),
                    "Int" => tokens.push(IntType),
                    "String" => tokens.push(StringType),
                    _ => tokens.push(Ident(ident)),
                }
            }

            _ => panic!("Unexpected char: {}", c),
        }
    }

    tokens.push(EOF);
    tokens
}
