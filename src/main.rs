use clap::Parser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use ast::{Ast, Tokenizer};
use machine::Machine;

mod ast;
mod machine;
mod tape;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    // Brainfuck source code file path
    #[arg(short, long)]
    filepath: String,
}

fn read_program(filepath: &str) -> anyhow::Result<Vec<u8>> {
    let mut f = BufReader::new(File::open(filepath).expect("Unable to open program file"));

    let mut bytes = vec![];
    f.read_to_end(&mut bytes).expect("error reading all bytes");
    Ok(bytes)
}

fn main() {
    let cli = Cli::parse();
    let path = Path::new(&cli.filepath);
    println!("{:?}", &path);
    let bytes =
        read_program(path.display().to_string().as_str()).expect("error reading source code file");
    let tokenizer = Tokenizer::from(bytes);
    let ast = Ast::from(tokenizer);
    Machine::default().run(&ast);
}
