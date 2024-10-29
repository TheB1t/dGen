mod parser;
mod optimizer;
mod semantic_analyzer;

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

    println!("{:#?}", validated_root);

    for error in semantic_analyzer.errors() {
        println!("Semantic error: {}", error);
    }

    Ok(())
}