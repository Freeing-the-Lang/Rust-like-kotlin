mod lexer;
mod parser;
mod transpiler;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rust-like-kotlin <file.rlk>");
        return;
    }

    let input_file = &args[1];
    let source = fs::read_to_string(input_file)
        .expect("Input file read error");

    let tokens = lexer::lex(&source);
    let ast = parser::parse(tokens);
    let kotlin_code = transpiler::to_kotlin(ast);

    println!("{}", kotlin_code);
}
