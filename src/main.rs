mod parser;

use std::fs::File;
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut file = File::open("src.dgen")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let stmt = parser::parse(contents);
    println!("{:#?}", stmt);
    Ok(())
}