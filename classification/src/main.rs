extern crate djvuxml;
extern crate num_traits;
extern crate serde;
extern crate serde_json;
extern crate stats;
#[macro_use]
extern crate serde_derive;
extern crate clap;
extern crate zip;

use clap::{App, Arg};
use num_traits::cast::ToPrimitive;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;

pub mod inquery;

struct StreamingStats {
    limits: stats::MinMax<f64>,
    info: stats::OnlineStats,
    total: f64,
}
impl StreamingStats {
    fn new() -> StreamingStats {
        StreamingStats {
            limits: stats::MinMax::new(),
            info: stats::OnlineStats::new(),
            total: 0.0,
        }
    }
    fn push_if_empty(&mut self, x: f64) {
        if self.is_empty() {
            self.push(x);
        }
    }
    fn is_empty(&self) -> bool {
        self.info.len() == 0
    }
    fn push(&mut self, x: f64) {
        self.limits.add(x);
        self.info.add(x);
        self.total += x;
    }
    fn total(&self) -> f64 {
        self.total
    }
    fn count(&self) -> f64 {
        self.info.len() as f64
    }
    fn max(&self) -> f64 {
        *self.limits.max().unwrap_or(&0.0)
    }
    fn min(&self) -> f64 {
        *self.limits.min().unwrap_or(&0.0)
    }
    fn mean(&self) -> f64 {
        self.info.mean()
    }
    fn stddev(&self) -> f64 {
        self.info.stddev()
    }
}

#[derive(Serialize, Deserialize)]
struct PageFeatures {
    book: String,
    page: u32,
    score: f64,
    features: HashMap<String, f64>,
    text: Option<String>,
}
impl PageFeatures {
    fn new(book: &str, page: u32, features: HashMap<String, f64>) -> PageFeatures {
        PageFeatures {
            book: book.to_owned(),
            page,
            score: 0.0,
            features,
            text: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum DecisionTreeNode {
    Branch {
        fid: usize,
        threshold: f64,
        lhs: Box<DecisionTreeNode>,
        rhs: Box<DecisionTreeNode>,
    },
    Leaf {
        leaf: [f64; 2],
    },
}
#[derive(Serialize, Deserialize)]
struct PyForestModel {
    feature_names: Vec<String>,
    forest: Vec<Vec<Box<DecisionTreeNode>>>,
}

impl DecisionTreeNode {
    fn predict(&self, features: &[f64]) -> (f64, f64) {
        match self {
            &DecisionTreeNode::Branch {
                fid,
                threshold,
                ref lhs,
                ref rhs,
            } => {
                if features[fid] <= threshold {
                    lhs.predict(features)
                } else {
                    rhs.predict(features)
                }
            }
            &DecisionTreeNode::Leaf { leaf } => (leaf[0], leaf[1]),
        }
    }
}

impl PyForestModel {
    fn predict(&self, page: &PageFeatures) -> f64 {
        let mut linear_features = vec![0.0; self.feature_names.len() + 1];
        for (feature, value) in &page.features {
            if let Some(index) = self.feature_names.iter().position(|f| f == feature) {
                linear_features[index] = *value
            } else {
                // unused features.
            }
        }

        let mut score_sum = 0.0;
        let mut score_total = 0.0;
        for forest in self.forest.iter() {
            let total = forest.len() as f64;
            let mut yes = 0.0;
            for tree in forest.iter() {
                let (y1, y2) = tree.predict(&linear_features);
                yes += y2 / (y1 + y2);
            }
            score_sum += yes;
            score_total += total;
        }

        return score_sum / score_total;
    }
}

fn process_book<W: io::Write, R: io::BufRead>(
    out: &mut W,
    path: &str,
    model: &PyForestModel,
    reader: R,
) -> Result<(), Box<Error>> {
    let book = djvuxml::process_book(reader)?;

    let mut page_words_stats = StreamingStats::new();
    let mut page_punct_stats = StreamingStats::new();
    let mut punct_by_page = Vec::new();
    let num_pages = book.pages.len();
    for p in &book.pages {
        let mut n_words = 0;
        let mut p_count = 0;
        for l in &p.lines {
            n_words += l.len();
            p_count += l
                .iter()
                .map(|bw| &bw.text)
                .filter(|w| w.chars().any(|c| c.is_ascii_punctuation()))
                .count();
        }
        punct_by_page.push(p_count);
        page_words_stats.push(n_words as f64);
        // Avoid division by zero:
        page_punct_stats.push(fraction(p_count, n_words))
    }

    let avg_punct = fmax(1.0, page_punct_stats.mean());
    let avg_words = fmax(1.0, page_words_stats.mean());

    for (i, p) in book.pages.iter().enumerate() {
        let mut features = HashMap::new();
        features.insert("page_fraction".to_owned(), fraction(i, num_pages));
        //insert_stats(&mut features, "," stats)

        let mut total_letters = 0;
        let mut letters_cap = 0;
        let mut letters_digits = 0;
        let mut letters_or_digits = 0;
        let mut stopwords = 0;
        let mut left_margin = StreamingStats::new();
        let mut right_margin = StreamingStats::new();
        let mut words_per_line = StreamingStats::new();
        let mut cap_lines = StreamingStats::new();
        let mut cap_words = StreamingStats::new();
        let mut num_words = 0;
        for l in &p.lines {
            num_words += l.len();
            words_per_line.push(l.len() as f64);
            for bw in l {
                let word = &bw.text;
                let token = word.trim().to_lowercase();
                if inquery::is_stopword(&token) {
                    stopwords += 1;
                }
                if word
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    cap_words.push(1.0)
                } else {
                    cap_words.push(0.0)
                }
                for c in word.chars() {
                    total_letters += 1;
                    if c.is_uppercase() {
                        letters_cap += 1;
                    }
                    if c.is_digit(10) {
                        letters_digits += 1;
                    } else if c.is_alphabetic() {
                        letters_or_digits += 1;
                    }
                }
            }
            if l.is_empty() {
                left_margin.push(0.5);
                right_margin.push(0.5);
                cap_lines.push(0.0);
            } else {
                let first = &l[0].coords;
                let last = &l.last().expect("Last word has coordinates!").coords;
                left_margin.push(fraction(min(first.x1, first.x2), p.width));
                right_margin.push(fraction(max(last.x1, last.x2), p.width));
                if let Some(true) = (&l[0].text).chars().next().map(|c| c.is_uppercase()) {
                    cap_lines.push(1.0);
                } else {
                    cap_lines.push(0.0);
                }
            }
        }

        left_margin.push_if_empty(0.5);
        right_margin.push_if_empty(0.5);
        cap_lines.push_if_empty(0.0);
        cap_words.push_if_empty(0.0);
        words_per_line.push_if_empty(0.0);

        features.insert(
            "scaled_punct".to_owned(),
            fraction(punct_by_page[i], avg_punct),
        );
        features.insert("scaled_len".to_owned(), fraction(num_words, avg_words));
        features.insert(
            "cap_letters".to_owned(),
            fraction(letters_cap, total_letters),
        );
        features.insert(
            "digits_letters".to_owned(),
            fraction(letters_digits, total_letters),
        );
        features.insert(
            "alphanum_letters".to_owned(),
            fraction(letters_or_digits, total_letters),
        );
        features.insert("num_pages".to_owned(), num_pages as f64);
        features.insert("page_fraction".to_owned(), fraction(i, num_pages));
        features.insert("num_words".to_owned(), num_words as f64);
        features.insert("stopwords".to_owned(), fraction(stopwords, num_words));
        insert_stats(&mut features, "left_margin", &left_margin);
        insert_stats(&mut features, "right_margin", &right_margin);
        insert_stats(&mut features, "words_per_line", &words_per_line);
        insert_stats(&mut features, "cap_lines", &cap_lines);
        insert_stats(&mut features, "cap_words", &cap_words);

        let mut output = PageFeatures::new(path, i as u32, features);
        let score = model.predict(&output);
        output.score = score;
        output.text = Some(book.get_page_text(i));
        write!(out, "{}\n", serde_json::to_string(&output)?)?;
    }

    Ok(())
}

fn fraction<A, B>(numerator: A, denominator: B) -> f64
where
    A: ToPrimitive,
    B: ToPrimitive,
{
    let x = numerator.to_f64().expect("Fraction Numerator");
    let y = fmax(1.0, denominator.to_f64().expect("Fraction Denominator"));
    x / y
}

fn fmax(lhs: f64, rhs: f64) -> f64 {
    if lhs > rhs {
        lhs
    } else {
        rhs
    }
}

fn insert_stats(map: &mut HashMap<String, f64>, name: &str, stats: &StreamingStats) {
    map.insert(format!("{}_max", name), stats.max());
    map.insert(format!("{}_min", name), stats.min());
    map.insert(format!("{}_mean", name), stats.mean());
    map.insert(format!("{}_stddev", name), stats.stddev());
    map.insert(format!("{}_total", name), stats.total());
    map.insert(format!("{}_count", name), stats.count());
}

fn run(archive_path: &str, model: &PyForestModel) -> Result<(), Box<Error>> {
    let f = File::open(archive_path)?;
    eprintln!("Opened file.");
    let mut zip = zip::ZipArchive::new(io::BufReader::new(f)).expect("ZipArchive::new");
    eprintln!("Opened ZipArchive.");
    let n = zip.len();
    eprintln!("Got ZipArchive length={}", n);

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for i in 0..n {
        let mut file = zip.by_index(i)?;
        let name = file.name().to_owned();
        eprintln!("{}/{}: {}", i, n, name);
        process_book(&mut out, &name, model, io::BufReader::new(file))?;
    }

    Ok(())
}

fn load_model(model_path: &str) -> Result<PyForestModel, Box<Error>> {
    let fp = io::BufReader::new(File::open(model_path)?);
    let forest: PyForestModel = serde_json::from_reader(fp)?;
    Ok(forest)
}

fn main() {
    let matches = App::new("classification")
        .version("0.1")
        .author("John Foley <jfoley@cs.umass.edu>")
        .about("Given a ZIP file with DJVUXML books, extract features from every page to JSON.")
        .arg(
            Arg::with_name("input_books")
                .long("books")
                .value_name("FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("model")
                .long("model")
                .value_name("FILE")
                .takes_value(true),
        )
        .get_matches();

    let archive_path = matches
        .value_of("input_books")
        .unwrap_or("/mnt/net/roaming/jfoley/web-docs/inex500.zip");
    let model_path = matches.value_of("model").expect("Model is required.");
    let model = load_model(model_path).expect("Model should be readable.");

    if let Err(e) = run(archive_path, &model) {
        eprintln!("Error! {:?}", e)
    }
}
