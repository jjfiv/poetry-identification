use std;
use std::cmp::max;
use types::WordCoords;

#[derive(Clone, Debug)]
enum CoordsParsingErr {
    Unicode(std::str::Utf8Error),
    Number(std::num::ParseIntError),
    NumberOfNumbers(),
}

impl std::fmt::Display for CoordsParsingErr {
    fn fmt(&self, _output: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        Ok(())
    }
}
impl std::error::Error for CoordsParsingErr {
    fn description(&self) -> &str {
        "CoordsParsingErr"
    }
}

impl From<std::str::Utf8Error> for CoordsParsingErr {
    fn from(err: std::str::Utf8Error) -> Self {
        CoordsParsingErr::Unicode(err)
    }
}

impl From<std::num::ParseIntError> for CoordsParsingErr {
    fn from(err: std::num::ParseIntError) -> Self {
        CoordsParsingErr::Number(err)
    }
}

impl WordCoords {
    /// Consider the example coordinates:
    ///
    /// <WORD coords="1329,2598,1495,255!">Music</WORD>
    /// <WORD coords="1523,2613,1801,2551">Academy,</WORD>
    /// <WORD coords="814,1248,1012,1168,1247">THE</WORD>
    /// <WORD coords="1068,1250,1548,1168,1248">TRAGEDY</WORD>
    ///
    /// We can infer that they are in the order: (x1,y2,x2,y1,base).
    fn parse(bytes: &[u8]) -> Result<WordCoords, CoordsParsingErr> {
        let coords: Result<Vec<u16>, CoordsParsingErr> = bytes
            .split(|x| *x == b',')
            .map(|coord_bytes| -> Result<u16, CoordsParsingErr> {
                let coord_str = std::str::from_utf8(coord_bytes)?;
                if coord_str.contains('!') {
                    let coord_str = coord_str.to_string();
                    Ok(coord_str.replace('!', "1").parse::<u16>()?)
                } else {
                    Ok(coord_str.parse::<u16>()?)
                }
            })
            .collect();

        let coords = coords?;

        match coords.len() {
            5 => Ok(WordCoords {
                x1: coords[0],
                y2: coords[1],
                x2: coords[2],
                y1: coords[3],
                base: coords[4],
            }),
            4 => Ok(WordCoords {
                x1: coords[0],
                y2: coords[1],
                x2: coords[2],
                y1: coords[3],
                base: max(coords[1], coords[3]),
            }),
            _ => Err(CoordsParsingErr::NumberOfNumbers()),
        }
    }
    pub(crate) fn parse_opt(bytes: &[u8]) -> Option<WordCoords> {
        match WordCoords::parse(bytes) {
            Ok(x) => Some(x),
            Err(_) => {
                eprintln!("Couldn't parse: {:?}", std::str::from_utf8(bytes));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abiographicaldi01bakegoog_djvu_line_50817() {
        let t1 = WordCoords::parse_opt(b"1329,2598,1495,255!").unwrap();
        let e1 = WordCoords {
            x1: 1329,
            y2: 2598,
            x2: 1495,
            y1: 2551,
            base: 2598,
        };
        assert_eq!(t1, e1);
    }

    #[test]
    fn happy_path() {
        let t1 = WordCoords::parse_opt(b"814,1248,1012,1168,1247").unwrap();
        let e1 = WordCoords {
            x1: 814,
            y2: 1248,
            x2: 1012,
            y1: 1168,
            base: 1247,
        };
        assert_eq!(t1, e1);

        let t2 = WordCoords::parse_opt(b"1068,1250,1548,1168,1248").unwrap();
        let e2 = WordCoords {
            x1: 1068,
            y2: 1250,
            x2: 1548,
            y1: 1168,
            base: 1248,
        };
        assert_eq!(t2, e2);
    }
}
