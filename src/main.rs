mod parser;
mod optimizer;

use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut file = File::open("src.dgen")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let root = parser::parse(contents);
    let optimized_root = optimizer::optimize(root);

    println!("{:#?}", optimized_root);
    Ok(())
}