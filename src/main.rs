mod parser;
mod optimizer;
mod semantic_analyzer;
mod dgen_ast;
mod sqf_ast;
mod dgen2sqf_ast;
mod transform;
mod boxable;

use crate::transform::*;

use std::fs::File;
use std::io::{self, Read};


fn main() -> io::Result<()> {
    let mut file = File::open("src.dgen")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let root = parser::parse(contents);
    let optimized_root = optimizer::optimize(root);
    let mut semantic_analyzer = semantic_analyzer::SemanticAnalyzer::new();
    let validated_root = semantic_analyzer.analyze(optimized_root);
    let sqf_ast : sqf_ast::Stmt = validated_root.transform();

    println!("{:#?}", sqf_ast);

    for error in semantic_analyzer.errors() {
        println!("Semantic error: {}", error);
    }

    Ok(())
}