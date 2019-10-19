extern crate djvuxml;

extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate zip;

use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use djvuxml::text::is_numeric;
use djvuxml::types::FastDjVu;
use FastDjVu::*;

const WINDOW_SIZE: usize = 32;
const CAPACITY: usize = WINDOW_SIZE * 2 + 1;
type CircBuf = VecDeque<PositionedEvent>;

#[derive(Debug, Fail)]
pub enum Problem {
    #[fail(display = "Error: {}", msg)]
    Msg { msg: String },
}
impl From<zip::result::ZipError> for Problem {
    fn from(err: zip::result::ZipError) -> Problem {
        Problem::Msg {
            msg: format!("{:?}", err),
        }
    }
}
impl From<std::io::Error> for Problem {
    fn from(err: std::io::Error) -> Problem {
        Problem::Msg {
            msg: format!("{:?}", err),
        }
    }
}

struct PositionedEvent {
    page: u32,
    word: u32,
    event: FastDjVu,
}
impl PositionedEvent {
    fn new(page: u32, word: u32, event: FastDjVu) -> PositionedEvent {
        PositionedEvent { page, word, event }
    }
}

fn main() {
    if let Err(e) = run("/mnt/net/roaming/jfoley/web-docs/inex500.zip") {
        eprintln!("Error! {:?}", e)
    }
}

fn as_str(x: &FastDjVu) -> &str {
    match *x {
        Word(ref w) => w.trim(),
        StartPage => "<P>",
        StartLine => "<L>",
        EndLine => "</L>",
        EndPage => "</P>",
        Error(_) => "",
    }
}

fn consider<W: Write>(buffer: &CircBuf, book: &str, dest: &mut W) -> Result<(), Problem> {
    assert_eq!(buffer.len(), CAPACITY);
    let center = &buffer[WINDOW_SIZE];
    if let Word(ref center_word) = center.event {
        if is_numeric(center_word) {
            write!(dest, "{}\t{}\t{}", book, center.page, center.word)?;
            for elem in buffer.iter() {
                write!(dest, "\t{}", as_str(&elem.event))?;
            }
            write!(dest, "\n")?;
        }
    }

    Ok(())
}

fn run(archive_path: &str) -> Result<(), Problem> {
    let f = File::open(archive_path)?;
    eprintln!("Opened file.");
    let mut zip = zip::ZipArchive::new(f)?;
    eprintln!("Opened ZipArchive.");
    let n = zip.len();
    eprintln!("Got ZipArchive length={}", n);

    for i in 0..n {
        let mut file = zip.by_index(i)?;
        let name = file.name().to_owned();
        eprintln!("{}/{}: {}", i, n, name);
        process_book(&name, io::BufReader::new(file))?;
    }

    Ok(())
}

fn process_book<R: BufRead>(path: &str, reader: R) -> Result<(), Problem> {
    let mut circ: CircBuf = CircBuf::with_capacity(CAPACITY);
    for _ in 0..CAPACITY {
        circ.push_back(PositionedEvent::new(0, 0, StartPage));
    }
    assert_eq!(circ.len(), CAPACITY);

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let mut page = 0;
    let mut word_index = 0;
    djvuxml::parse_fast_xml(reader, |item| {
        match item {
            Word(ref word) => {
                let _ = circ.pop_front();
                circ.push_back(PositionedEvent::new(
                    page,
                    word_index,
                    Word(word.to_owned()),
                ));
                assert_eq!(circ.len(), CAPACITY);
                word_index += 1;
            }
            StartPage | StartLine | EndLine | EndPage => {
                let _ = circ.pop_front();
                circ.push_back(PositionedEvent::new(page, word_index, item.clone()));
                assert_eq!(circ.len(), CAPACITY);
            }
            Error(ref msg) => eprintln!("{:?}", msg),
        }
        consider(&circ, path, &mut out).unwrap();
        if item == EndPage {
            page += 1;
            word_index = 0;
        }
    });

    for _ in 0..CAPACITY {
        let _ = circ.pop_front();
        circ.push_back(PositionedEvent::new(page + 1, 0, EndPage));
        consider(&circ, path, &mut out)?;
    }
    Ok(())
}
