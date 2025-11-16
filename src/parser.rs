use crate::lexer::Token;

#[derive(Debug)]
pub enum Ast {
    LetAssign {
        name: String,
        value: i64,
    }
}

pub fn parse(tokens: Vec<Token>) -> Vec<Ast> {
    let mut ast = Vec::new();
    let mut iter = tokens.into_iter().peekable();

    while let Some(tok) = iter.next() {
        match tok {
            Token::Let => {
                if let Some(Token::Ident(name)) = iter.next() {
                    let _eq = iter.next(); // '='
                    if let Some(Token::Number(val)) = iter.next() {
                        let _semi = iter.next(); // ';'
                        ast.push(Ast::LetAssign { name, value: val });
                    }
                }
            }
            _ => {}
        }
    }

    ast
}
