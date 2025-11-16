#[derive(Debug, Clone)]
pub enum Token {
    Let,
    Ident(String),
    Number(i64),
    Assign,
    Semicolon,
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.peek() {
        match c {
            ' ' | '\n' | '\t' => { chars.next(); }
            '=' => { tokens.push(Token::Assign); chars.next(); }
            ';' => { tokens.push(Token::Semicolon); chars.next(); }

            c if c.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(d) = chars.peek() {
                    if d.is_ascii_digit() {
                        num.push(*d);
                        chars.next();
                    } else { break; }
                }
                tokens.push(Token::Number(num.parse().unwrap()));
            }

            c if c.is_ascii_alphabetic() => {
                let mut id = String::new();
                while let Some(d) = chars.peek() {
                    if d.is_ascii_alphabetic() {
                        id.push(*d);
                        chars.next();
                    } else { break; }
                }

                if id == "let" {
                    tokens.push(Token::Let);
                } else {
                    tokens.push(Token::Ident(id));
                }
            }

            _ => {
                chars.next(); // skip unknown chars for now
            }
        }
    }

    tokens
}
