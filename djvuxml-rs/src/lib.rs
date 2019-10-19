extern crate quick_xml;
extern crate regex;

#[macro_use]
extern crate lazy_static;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod coords;
mod params;
pub mod text;
pub mod types;

use quick_xml::events::*;
use quick_xml::reader::Reader;
use std::io::BufRead;
use types::FastDjVu;
use types::RichDjVu;
use types::WordCoords;
use types::Book;
use types::BookWord;

const WORD: &[u8] = b"WORD";
const LINE: &[u8] = b"LINE";
const PAGE: &[u8] = b"OBJECT";
const PARAM: &[u8] = b"PARAM";
const COORDS_ATTR: &[u8] = b"coords";

/// Parse rich events from a DJVU XML document.
///
/// Call ``callback`` whenever we encounter a ``RichDjVu`` element in a DJVU XML file.
pub fn parse_rich_xml<R, F>(rdr: R, mut callback: F)
where
    R: BufRead,
    F: FnMut(RichDjVu) -> (),
{
    let mut xml = Reader::from_reader(rdr);
    xml.check_end_names(false);
    xml.expand_empty_elements(true);
    let mut buf = Vec::new();
    let mut in_word = false;
    let mut recent_coords: Option<WordCoords> = None;
    loop {
        match xml.read_event(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => match e.name() {
                WORD => {
                    in_word = true;
                    recent_coords = e
                        .attributes()
                        .flat_map(|r| r)
                        .find(|attr| attr.key == COORDS_ATTR)
                        .and_then(|attr| WordCoords::parse_opt(&attr.value))
                }
                PARAM => match params::process(e) {
                    Err(e) => callback(RichDjVu::Error(format!("{:?}", e))),
                    Ok(Some(evt)) => callback(evt),
                    _ => {}
                },
                PAGE => match params::process_page(e) {
                    Err(e) => callback(RichDjVu::Error(format!("{:?}", e))),
                    Ok(Some(evt)) => callback(evt),
                    _ => {}
                },
                LINE => callback(RichDjVu::StartLine),
                _ => {} //println!("{:?}", std::str::from_utf8(e.name()))
            },
            Ok(Event::End(ref e)) => match e.name() {
                WORD => in_word = false,
                LINE => callback(RichDjVu::EndLine),
                PAGE => callback(RichDjVu::EndPage),
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if in_word {
                    let txt = e.unescape_and_decode(&xml).unwrap();
                    callback(RichDjVu::Word(recent_coords, txt));
                } else {
                }
            }
            _ => {}
        }
        buf.clear();
    }
}

pub fn process_book<R: BufRead>(
    reader: R
) -> Result<Book, String> {
    let mut book = Book::new();
    let mut errors = Vec::new();
    parse_rich_xml(reader, |item| match item {
        RichDjVu::PageDimensions(w, h) => {
            let mut page = book.current_page();
            page.width = w;
            page.height = h;
        }
        RichDjVu::PageDPI(dpi) => book.current_page().dpi = dpi,
        RichDjVu::StartLine => book.current_page().lines.push(Vec::new()),
        RichDjVu::Word(owc, ref text) => {
            // Ditch words where location could not be parsed!
            if let Some(coords) = owc {
                book.current_line()
                .push(BookWord::new(coords, text.clone()))
            }
        },
        RichDjVu::EndLine => {}
        RichDjVu::EndPage => book.end_page(),
        RichDjVu::Error(msg) => errors.push(msg),
    });

    // Remove last page if invalid:
    if !book.current_page().valid() {
        let _ = book.pages.pop();
    }
    Ok(book)
}

/// Parse simple events from a DJVU XML document.
///
/// Call ``callback`` whenever we encounter a ``FastDjVu`` element in a DJVU XML file.
pub fn parse_fast_xml<R, F>(rdr: R, mut callback: F)
where
    R: BufRead,
    F: FnMut(FastDjVu) -> (),
{
    let mut xml = Reader::from_reader(rdr);
    xml.check_end_names(false);
    let mut buf = Vec::new();
    let mut in_word = false;
    loop {
        match xml.read_event(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(ref e)) => match e.name() {
                WORD => in_word = true,
                PAGE => callback(FastDjVu::StartPage),
                LINE => callback(FastDjVu::StartLine),
                _ => {}
            },
            Ok(Event::End(ref e)) => match e.name() {
                WORD => in_word = false,
                LINE => callback(FastDjVu::EndLine),
                PAGE => callback(FastDjVu::EndPage),
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if in_word {
                    let txt = e.unescape_and_decode(&xml).unwrap();
                    callback(FastDjVu::Word(txt));
                } else {
                }
            }
            _ => {}
        }
        buf.clear();
    }
}

/// `trim_book` returns the internet archive id from a path in a zip file.
///
/// ```
/// use djvuxml::trim_book;
/// assert_eq!("abelincjohn02morsrich", trim_book("inex500/abelincjohn02morsrich_djvu.xml"));
/// assert_eq!("abelincjohn02morsrich", trim_book("inex500/abelincjohn02morsrich.xml"));
/// assert_eq!("abelincjohn02morsrich", trim_book("abelincjohn02morsrich_djvu.xml"));
/// assert_eq!("abelincjohn02morsrich", trim_book("abelincjohn02morsrich.xml"));
/// ```
pub fn trim_book(input: &str) -> &str {
    let stripped = input
        .trim_right_matches("_djvu.xml")
        .trim_right_matches(".xml");
    if let Some(pt) = stripped.rfind('/') {
        &stripped[(pt + 1)..]
    } else {
        stripped
    }
}
