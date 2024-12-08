use clap::Parser;
use pest::Parser as PestParser;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

#[allow(unused_imports)]
use ast::{Ast, Tokenizer};
use codegen::IRCodegen;
#[allow(unused_imports)]
use machine::Machine;

mod ast;
mod codegen;
mod machine;
mod tape;

#[derive(clap::ValueEnum, Clone)]
enum ParserMode {
    Pest,
    Internal,
}

#[derive(clap::ValueEnum, Clone)]
enum Mode {
    Jit,     // Just in time Compilation using LLVM
    Compile, // Complies using llvm and clang
    Machine, // Custom interpreter much slower
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    // Brainfuck source code file path
    #[arg(short, long)]
    filepath: String,

    // Switches between pest and internal parser
    #[arg(short, long, value_enum, default_value_t = ParserMode::Pest)]
    parsemode: ParserMode,

    #[arg(short, long, value_enum, default_value_t = Mode::Jit)]
    mode: Mode,
}

#[allow(dead_code)]
fn read_program(filepath: &Path) -> anyhow::Result<Vec<u8>> {
    let mut f = BufReader::new(File::open(filepath).expect("Unable to open program file"));

    let mut bytes = vec![];
    f.read_to_end(&mut bytes).expect("error reading all bytes");
    Ok(bytes)
}

fn main() {
    let cli = Cli::parse();
    let path = Path::new(&cli.filepath);

    let ast = match cli.parsemode {
        ParserMode::Pest => {
            let source = std::fs::read_to_string(path).expect("error reading source code");

            let pairs = ast::BrainFuckParser::parse(ast::Rule::Program, &source)
                .expect("error tokenizing program");
            Ast::from(pairs)
        }
        ParserMode::Internal => {
            let bytes = read_program(path).expect("error reading source code file");
            let tokenizer = Tokenizer::from(bytes);
            Ast::from(tokenizer)
        }
    };

    match cli.mode {
        Mode::Jit => IRCodegen::from(&ast).jit(),
        Mode::Compile => IRCodegen::from(&ast).compile("brainfuck"),
        Mode::Machine => Machine::default().run(&ast),
    }
}
