mod lexer;
mod parser;
mod semantic;
mod codegen;

use lexer::lex;
use parser::Parser;
use semantic::SemanticAnalyzer;
use codegen::Codegen;

use std::fs;

fn main() {
    let input = fs::read_to_string("input.sp")
        .expect("Failed to read input.sp");

    let tokens = lex(&input);

    // Parser::new 는 Vec<Token> 소유권 받음
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();   // ✅ expect 제거

    let sem = SemanticAnalyzer::new(program);
    let ir  = sem.analyze();

    let cg  = Codegen::new();
    let asm = cg.generate(&ir);

    println!("{}", asm);
}
