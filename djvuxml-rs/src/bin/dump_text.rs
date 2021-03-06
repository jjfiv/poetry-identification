extern crate clap;
extern crate djvuxml;
extern crate quick_xml;

use clap::{App, Arg};
use djvuxml::types::FastDjVu;
use std::fs::File;
use std::io::{self, BufReader};
use FastDjVu::*;

fn process_xml(path: &str) -> io::Result<()> {
    let f = BufReader::new(File::open(path)?);
    djvuxml::parse_fast_xml(f, |item| match item {
        StartPage | StartLine => {}
        Word(word) => print!("{} ", word),
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
