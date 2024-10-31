mod parser;
mod optimizer;
mod semantic_analyzer;
mod generic;
mod dgen_ast;
mod sqf_ast;
mod dgen2sqf_ast;
mod sqf_generator;
mod transform;
mod boxable;

use std::io;

use crate::transform::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "dGen to SQF transpiler", version = "v0.1", author = "Bit")]
#[command(about = "Transpiles dGen to SQF", long_about = None)]

struct Cli {
    #[arg(short, long)]
    input: String,
    #[arg(short, long, default_value = "out.sqf")]
    output: String,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();

    let mut semantic_analyzer   = semantic_analyzer::SemanticAnalyzer::new();

    let raw_root                = parser::parse_file(args.input)?;
    let optimized_root          = optimizer::optimize(raw_root);
    let validated_root          = semantic_analyzer.analyze(optimized_root);
    let errors                  = semantic_analyzer.errors();

    if errors.len() > 0 {
        for error in errors {
            println!("Semantic error: {}", error);
        }

        Err(io::Error::new(io::ErrorKind::InvalidData, "Semantic errors found"))
    } else {
        let sqf_ast : sqf_ast::Stmt = validated_root.transform();
        let code                    = sqf_ast.generate_sqf(0);
        std::fs::write(args.output, code)?;

        Ok(())
    }
}