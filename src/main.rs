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

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();

    let sem = SemanticAnalyzer::new(program);
    let ir  = sem.analyze();

    let cg  = Codegen;          // 여기! new() 필요 없음
    let asm = cg.generate(&ir); // 그대로 호출 가능

    println!("{}", asm);
}
