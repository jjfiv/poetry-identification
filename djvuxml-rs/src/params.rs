extern crate quick_xml;

use quick_xml::events::BytesStart;
use std;
use types::RichDjVu;

const NAME_KEY: &[u8] = b"name";
const VALUE_KEY: &[u8] = b"value";
const WIDTH_KEY: &[u8] = b"width";
const HEIGHT_KEY: &[u8] = b"height";

#[derive(Clone, Debug)]
pub(crate) enum ParamError {
    BadUnicode(std::str::Utf8Error),
    BadNumber(std::num::ParseIntError),
    Missing(&'static [u8]),
}

impl std::fmt::Display for ParamError {
    fn fmt(&self, _output: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}
impl std::error::Error for ParamError {
    fn description(&self) -> &str {
        ""
    }
}
impl From<std::str::Utf8Error> for ParamError {
    fn from(err: std::str::Utf8Error) -> Self {
        ParamError::BadUnicode(err)
    }
}
impl From<std::num::ParseIntError> for ParamError {
    fn from(err: std::num::ParseIntError) -> Self {
        ParamError::BadNumber(err)
    }
}

fn get_attribute(e: &BytesStart, query: &'static [u8]) -> Result<String, ParamError> {
    let attr = e
        .attributes()
        .flat_map(|r| r)
        .find(|attr| attr.key == query)
        .ok_or_else(|| ParamError::Missing(query))?;
    Ok(String::from(std::str::from_utf8(&attr.value)?))
}

pub(crate) fn process(e: &BytesStart) -> Result<Option<RichDjVu>, ParamError> {
    let name = get_attribute(e, NAME_KEY)?;
    match name.as_str() {
        "DPI" => {
            let val = get_attribute(e, VALUE_KEY)?;
            return Ok(Some(RichDjVu::PageDPI(val.parse::<u32>()?)));
        }
        _ => {}
    }
    Ok(None)
}

pub(crate) fn process_page(e: &BytesStart) -> Result<Option<RichDjVu>, ParamError> {
    let w = get_attribute(e, WIDTH_KEY)?.parse::<u32>()?;
    let h = get_attribute(e, HEIGHT_KEY)?.parse::<u32>()?;
    Ok(Some(RichDjVu::PageDimensions(w, h)))
}
