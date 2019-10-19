extern crate djvuxml;
extern crate quick_xml;

use std::fs::File;
use std::io::{self, BufReader};

fn process_xml(path: &str) -> io::Result<()> {
    let f = BufReader::new(File::open(path)?);
    djvuxml::parse_rich_xml(f, |item| println!("{:?}", item));
    Ok(())
}

fn main() {
    if let Err(e) = process_xml("resources/tragedyofhamletp00shak_djvu.xml") {
        println!("Error! {:?}", e)
    }
}
