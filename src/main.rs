use std::env;
use std::fs;
use std::io;
use std::io::Write;
use std::*;

pub mod ast_nodes;
pub mod defaults;
pub mod interpreter;
pub mod lang_errors;
pub mod spans;
pub mod tests;
pub mod token_lexer;
pub mod token_parser;
pub mod tokens;
use colored::Colorize;
use interpreter::Interpreter;
use lang_errors::ErrorBuilder;
use lang_errors::*;
use token_parser::Parser;
fn input(message: &str) -> String {
    print!("{message} ");
    io::stdout().flush().unwrap();
    let mut result = String::new();
    io::stdin()
        .read_line(&mut result)
        .expect("didnt receive input:");
    return String::from(result.trim());
}
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => repl(),
        2 => len2(args),
        3 => len3(args),

        _ => panic!("invalid commands"),
    }
}

fn AST_from_file(file_path: String) {
    let source = fs::read_to_string(file_path).expect("Should have been able to read the file");
    let mut parser = Parser::new(source.as_str());
    let ast = parser.batch_parse();
    println!("{ast:#?}");
}
fn len2(args: Vec<String>) {
    match args[1].to_lowercase().as_str() {
        "ast" | "a" => test_repl(),
        "help" | "h" => help(),
        _ => execute_file(args),
    }
}
fn len3(args: Vec<String>) {
    match args[1].to_lowercase().as_str() {
        "ast" | "a" => AST_from_file(args[2].clone()),
        _ => panic!("Invalid"),
    }
}
fn execute_file(args: Vec<String>) {
    let file_path = &args[1];
    let source = fs::read_to_string(file_path).expect("Should have been able to read the file");
    let err_out = ErrorBuilder::new(source.clone());
    let mut parser = Parser::new(source.as_str());
    let ast_result = parser.batch_parse();
    let Ok(ast) = ast_result else {
        ast_result.unwrap_err().print_msg(err_out);
        return;
    };
    let mut interpreter = Interpreter::new(ast);
    interpreter.execute().map_err(|e| e.print_msg(err_out));
}
fn repl() {
    loop {
        let source = input(">: ");
        let err_out = ErrorBuilder::new(source.clone());
        let mut parser = Parser::new(source.as_str());
        let ast_result = parser.batch_parse_expr();
        let Ok(ast) = ast_result else {
            ast_result.unwrap_err().print_msg(err_out); continue;
        };
        let init_result = Interpreter::execute_node(ast);
        let Ok((result,_)) = init_result else{
            init_result.unwrap_err().print_msg(err_out);
            continue;
        };
        println!("{}", defaults::val_to_str(&result).bright_black());
    }
}

fn test_repl() {
    loop {
        let source = input(">: ");
        let err_out = ErrorBuilder::new(source.clone());
        let mut parser = Parser::new(source.as_str());

        let ast_result = parser.batch_parse_expr();
        let Ok(ast) = ast_result else {
            ast_result.unwrap_err().print_msg(err_out); continue;
        };
        println!("{ast:#?}");
    }
}
fn help() {
    println!(
        "Help

no args - starts the repl
<file path> - runs the file
<ast,a> <optional file path> - reads input either from the repl or from a file and outputs the AST as text
"
);
}
