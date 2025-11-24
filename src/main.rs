mod lexer;
mod parser;
mod semantic;
mod codegen;

use std::fs;

fn main() {
    let input = fs::read_to_string("input.sp")
        .expect("Failed to read input.sp");

    let tokens = lexer::lex(&input);
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program();

    let analyzer = semantic::SemanticAnalyzer::new(program);
    let ir = analyzer.analyze();

    let mut cg = codegen::Codegen::new();
    let asm = cg.gen_program(&ir);

    println!("{}", asm);
}
