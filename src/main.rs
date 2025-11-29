mod lexer;
mod parser;
mod semantic;
mod codegen;

use std::fs;

fn main() {
    let source = fs::read_to_string("input.rlk")
        .expect("input.rlk missing");

    let tokens = lexer::lex(&source);
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse_program();

    let semantic = semantic::SemanticAnalyzer::new(ast);
    let ir = semantic.analyze();

    let codegen = codegen::Codegen;
    let asm = codegen.generate(&ir);

    println!("{}", asm);
}
