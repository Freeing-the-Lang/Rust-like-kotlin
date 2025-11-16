use crate::parser::Ast;

pub fn to_kotlin(ast: Vec<Ast>) -> String {
    let mut out = String::new();
    out.push_str("fun main() {\n");

    for node in ast {
        match node {
            Ast::LetAssign { name, value } => {
                out.push_str(&format!("    var {} = {}\n", name, value));
            }
        }
    }

    out.push_str("}\n");
    out
}
