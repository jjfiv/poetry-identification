extern crate djvuxml;

use std::fs::File;
use std::io::{self, BufReader};

use djvuxml::text::clean_word;
use djvuxml::types::FastDjVu;
use FastDjVu::*;

fn process_xml(path: &str) -> io::Result<()> {
    let f = BufReader::new(File::open(path)?);
    djvuxml::parse_fast_xml(f, |item| match item {
        StartPage | StartLine => {}
        Word(word) => {
            if let Some(ref w) = clean_word(&word) {
                println!("{}", w)
            }
        }
        EndLine | EndPage => println!(),
        Error(msg) => eprintln!("{:?}", msg),
    });
    Ok(())
}

fn main() {
    if let Err(e) = process_xml("resources/tragedyofhamletp00shak_djvu.xml") {
        println!("Error! {:?}", e)
    }
}
