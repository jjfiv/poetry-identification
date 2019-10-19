#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WordCoords {
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
    pub base: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RichDjVu {
    PageDimensions(u32, u32),
    PageDPI(u32),
    StartLine,
    Word(Option<WordCoords>, String),
    EndLine,
    EndPage,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookWord {
    pub coords: WordCoords,
    pub text: String,
}
impl BookWord {
    pub fn new(coords: WordCoords, text: String) -> Self {
        BookWord { coords, text }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookPage {
    pub width: u32,
    pub height: u32,
    pub dpi: u32,
    pub lines: Vec<Vec<BookWord>>,
}
impl BookPage {
    pub fn new() -> Self {
        BookPage {
            width: 0,
            height: 0,
            dpi: 0,
            lines: Vec::new(),
        }
    }
    pub fn valid(&self) -> bool {
        self.width != 0 && self.height != 0 && self.dpi != 0 && !self.lines.is_empty()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Book {
    pub pages: Vec<BookPage>
}
impl Book {
    pub fn new() -> Book {
        let mut pages = Vec::new();
        pages.push(BookPage::new());
        Book { pages }
    }
    pub fn current_page(&mut self) -> &mut BookPage {
        self.pages.last_mut().unwrap()
    }
    pub fn current_line(&mut self) -> &mut Vec<BookWord> {
        self.pages.last_mut().unwrap().lines.last_mut().unwrap()
    }
    pub fn end_page(&mut self) {
        self.pages.push(BookPage::new())
    }
    pub fn get_page_text(&self, index: usize) -> String {
        let mut words = String::new();
        let lines = &self.pages[index].lines;

        for line in lines {
            for (i, &BookWord{ ref text, .. }) in line.iter().enumerate() {
                if i > 0 {
                    words.push('\t')
                }
                words.push_str(text)
            }
            words.push('\n')
        }

        words
    }
}

/// Represent the events you usually want from parsing DJVU books: words, lines, and pages.
///
/// By ignoring the rest of the XML, we get a lot of efficiency for the most frequent
/// operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FastDjVu {
    StartPage,
    StartLine,
    Word(String),
    EndLine,
    EndPage,
    Error(String),
}
