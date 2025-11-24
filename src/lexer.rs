#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Func,
    Let,
    Return,
    If,
    Else,

    // Types
    IntType,
    StringType,

    // Identifiers & Literals
    Ident(String),
    Number(i64),
    StringLiteral(String),

    // Symbols
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    Colon,      // :
    Semicolon,  // ;
    Comma,      // ,

    // Operators
    Assign,     // =
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /

    Greater,    // >
    Less,       // <
    EqualEqual, // ==
    NotEqual,   // !=

    EOF,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // whitespace
            ' ' | '\n' | '\t' | '\r' => { chars.next(); }

            // punctuation
            '(' => { chars.next(); tokens.push(Token::LParen); }
            ')' => { chars.next(); tokens.push(Token::RParen); }
            '{' => { chars.next(); tokens.push(Token::LBrace); }
            '}' => { chars.next(); tokens.push(Token::RBrace); }
            ':' => { chars.next(); tokens.push(Token::Colon); }
            ';' => { chars.next(); tokens.push(Token::Semicolon); }
            ',' => { chars.next(); tokens.push(Token::Comma); }

            // operators
            '+' => { chars.next(); tokens.push(Token::Plus); }
            '-' => { chars.next(); tokens.push(Token::Minus); }
            '*' => { chars.next(); tokens.push(Token::Star); }
            '/' => { chars.next(); tokens.push(Token::Slash); }

            '=' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::EqualEqual);
                } else {
                    tokens.push(Token::Assign);
                }
            }

            '!' => {
                chars.next();
                if let Some('=') = chars.peek() {
                    chars.next();
                    tokens.push(Token::NotEqual);
                }
            }

            '>' => { chars.next(); tokens.push(Token::Greater); }
            '<' => { chars.next(); tokens.push(Token::Less); }

            // string literal
            '"' => {
                chars.next(); // skip "
                let mut s = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch == '"' {
                        chars.next();
                        break;
                    } else {
                        s.push(ch);
                        chars.next();
                    }
                }

                tokens.push(Token::StringLiteral(s));
            }

            // numbers
            c if c.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num.parse().unwrap()));
            }

            // identifiers (func, let, return, types…)
            c if c.is_ascii_alphabetic() => {
                let mut id = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_alphanumeric() || d == '_' {
                        id.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }

                match id.as_str() {
                    "func" => tokens.push(Token::Func),
                    "let" => tokens.push(Token::Let),
                    "return" => tokens.push(Token::Return),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "int" => tokens.push(Token::IntType),
                    "string" => tokens.push(Token::StringType),
                    _ => tokens.push(Token::Ident(id)),
                }
            }

            _ => {
                chars.next(); // skip unknown char
            }
        }
    }

    tokens.push(Token::EOF);
    tokens
}
