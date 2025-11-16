use crate::parser::Ast;

pub fn to_sp(ast: Vec<Ast>) -> String {
    let mut out = String::new();

    for node in ast {
        match node {
            Ast::LetAssign { name, value } => {
                out.push_str(&format!("let {} = {}\n", name, value));
            }
            Ast::BinaryExpr { left, op, right } => {
                out.push_str(&format!("let temp = {} {} {}\n", left, op, right));
            }
            Ast::PrintExpr { value } => {
                out.push_str(&format!("print({})\n", value));
            }
        }
    }

    out
}
