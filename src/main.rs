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
    // -------------------------------
    // 1. Read input.sp
    // -------------------------------
    let input = match fs::read_to_string("input.sp") {
        Ok(s) => s,
        Err(_) => {
            eprintln!("ERROR: Cannot read input.sp (file missing)");
            std::process::exit(1);
        }
    };

    // -------------------------------
    // 2. Lexing
    // -------------------------------
    let tokens = lex(&input);

    // -------------------------------
    // 3. Parsing
    // -------------------------------
    let mut parser = Parser::new(&tokens);
    let program = match parser.parse_program() {
        Ok(p) => p,
        Err(err) => {
            eprintln!("PARSER ERROR: {}", err);
            std::process::exit(1);
        }
    };

    // -------------------------------
    // 4. Semantic Analysis
    // -------------------------------
    let mut semantic = Semantic::new();
    let ir = match semantic.process(program) {
        Ok(ir) => ir,
        Err(err) => {
            eprintln!("SEMANTIC ERROR: {}", err);
            std::process::exit(1);
        }
    };

    // -------------------------------
    // 5. Codegen (ASM output)
    // -------------------------------
    let codegen = Codegen::new();
    let asm = codegen.generate(&ir);

    // -------------------------------
    // 6. Print ASM to stdout
    //    CI에서 (>) 리다이렉트하여 파일로 저장
    // -------------------------------
    println!("{}", asm);
}
