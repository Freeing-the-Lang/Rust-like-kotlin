mod lexer;
mod parser;
mod semantic;
mod codegen;

use lexer::*;
use parser::*;
use semantic::*;
use codegen::Codegen;

fn main() {
    let input = std::fs::read_to_string("input.sp")
        .expect("Failed to read input.sp");

    let tokens = lex(&input);

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program().expect("Parse error");

    let sem = SemanticAnalyzer::new(program);
    let ir = sem.analyze();

    let cg = Codegen;
    let asm = cg.generate(&ir);

    println!("{}", asm);
}
