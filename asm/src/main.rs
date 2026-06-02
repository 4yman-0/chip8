use chip8_asm::{compiler, lexer, parser};
use clap::Parser as ClapParser;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

#[derive(ClapParser)]
struct Cli {
    input: PathBuf,

    #[arg(short, long)]
    output: PathBuf,

    #[arg(
    	short,
    	long,
    	default_value_t = false,
    	action = clap::ArgAction::SetTrue
    )]
    force: bool,

    #[arg(
    	short,
    	long,
    	default_value_t = false,
    	action = clap::ArgAction::SetTrue
    )]
    verbose: bool,
}

fn read_from_stdin() -> String {
    let mut buffer = String::new();
    let stdin = io::stdin();
    stdin.read_line(&mut buffer).unwrap();
    buffer
}

/*fn debug_print_array<T: core::fmt::Debug>(arr: &[T]) {
    println!("[");
    for i in arr {
        println!("\t{i:?}");
    }
    println!("]");
}*/

fn display_print_array<T: core::fmt::Display>(arr: &[T]) {
    println!("[");
    for i in arr {
        println!("\t{i}");
    }
    println!("]");
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    if fs::exists(&cli.output).unwrap() && !cli.force {
        print!("Output file already exists, overwrite? [y/N]: ");
        io::stdout().flush().unwrap();
        let input = read_from_stdin();
        if input.trim().to_lowercase() != "y" {
            println!();
            return;
        }
    }

    let input = fs::read_to_string(&cli.input).unwrap();

    let tokens = lexer::lex(&input);
    if cli.verbose {
        display_print_array(tokens.as_slice());
    }
    let _ = input;

    let ast = parser::parse(&tokens);
    if cli.verbose {
        display_print_array(ast.insts.as_slice());
    }
    let _ = tokens;

    let binary = compiler::compile(&ast);
    let _ = ast;

    fs::write(cli.output, &binary).unwrap();
    let _ = binary;
}
