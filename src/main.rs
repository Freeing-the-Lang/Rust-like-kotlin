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
    // ---------------------
    // 1. Read input.sp file
    // ---------------------
    let input = match fs::read_to_string("input.sp") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("ERROR: input.sp not found");
            std::process::exit(1);
        }
    };

    // ---------------------
    // 2. Lexing
    // ---------------------
    let tokens = lex(&input);

    // ---------------------
    // 3. Parsing
    // ---------------------
    let mut parser = Parser::new(tokens);   // MUST PASS Vec<Token>
    let program = parser.parse_program();   // returns Program (not Result)

    // ---------------------
    // 4. Semantic Analysis
    // ---------------------
    let semantic = SemanticAnalyzer::new(program);
    let ir = semantic.analyze();   // returns IRProgram

    // ---------------------
    // 5. Codegen: IR → NASM ASM
    // ---------------------
    let cg = Codegen::new();
    let asm = cg.generate(&ir);

    // ---------------------
    // 6. Write ASM to stdout
    // ---------------------
    println!("{}", asm);
}
