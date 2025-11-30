mod lexer;
mod parser;
mod semantic;
mod codegen;

use std::fs;
use std::env;

fn main() {
    let source = fs::read_to_string("input.rlk")
        .expect("input.rlk missing");

    let tokens = lexer::lex(&source);
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse_program();

    let semantic = semantic::SemanticAnalyzer::new(ast);
    let ir = semantic.analyze();

    // detect system architecture
    let arch = env::consts::ARCH;   // "x86_64" or "aarch64"

    let asm = if arch == "aarch64" {
        codegen::generate_arm64(&ir)
    } else {
        codegen::generate_x86_64(&ir)
    };

    println!("{}", asm);
}
