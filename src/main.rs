mod lexer;
mod parser;
mod semantic;
mod codegen;

use lexer::lex;
use parser::Parser;
use semantic::Semantic;
use codegen::Codegen;

use std::fs;

fn main() {
    // 1. read input.sp
    let input = match fs::read_to_string("input.sp") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("ERROR: input.sp not found");
            std::process::exit(1);
        }
    };

    // 2. lexing
    let tokens = lex(&input);

    // 3. parsing
    let mut parser = Parser::new(&tokens);
    let program = parser.parse_program();   // ✔ Program directly returned

    // 4. semantic analysis
    let mut semantic = Semantic::new();
    let ir = semantic.process(program);     // ✔ your semantic already returns Program/IR

    // 5. codegen
    let cg = Codegen::new();
    let asm = cg.generate(&ir);

    // 6. output ASM
    println!("{}", asm);
}
