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

// ===========================
// keyword / type lookup table
// ===========================
fn lookup_keyword(word: &str) -> Token {
    match word.to_lowercase().as_str() {
        "func"   => Token::Func,
        "let"    => Token::Let,
        "return" => Token::Return,
        "if"     => Token::If,
        "else"   => Token::Else,

        // types
        "int"    => Token::IntType,
        "string" => Token::StringType,

        _ => Token::Ident(word.to_string()),
    }
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            // whitespace
            ' ' | '\n' | '\t' | '\r' => {
                chars.next();
            }

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
                    }
                    s.push(ch);
                    chars.next();
                }

                tokens.push(Token::StringLiteral(s));
            }

            // numbers
            d if d.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_digit() {
                        num.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num.parse().unwrap()));
            }

            // identifiers / keywords / types
            alpha if alpha.is_ascii_alphabetic() => {
                let mut id = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_alphanumeric() || d == '_' {
                        id.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }

                tokens.push(lookup_keyword(&id));
            }

            // unknown / skip
            _ => {
                chars.next();
            }
        }
    }

    tokens.push(Token::EOF);
    tokens
}
