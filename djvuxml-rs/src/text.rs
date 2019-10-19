use regex::Regex;

lazy_static! {
    static ref NOT_WORD: Regex = Regex::new(r"\W").unwrap();
    static ref PUNCT: Regex = Regex::new(r"\pP").unwrap();
    static ref ONLY_DIGITS: Regex = Regex::new(r"^\d+$").unwrap();
}

pub fn is_numeric(input: &str) -> bool {
    ONLY_DIGITS.is_match(input)
}

pub fn clean_word(input: &str) -> Option<String> {
    let x1 = input.to_lowercase();
    let x2 = NOT_WORD.replace_all(x1.as_ref(), "");
    let x3 = PUNCT.replace_all(x2.as_ref(), "");
    let x4 = x3.as_ref().trim();

    if x4.is_empty() {
        None
    } else {
        Some(String::from(x4))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_effect() {
        assert_eq!("the", clean_word("the").unwrap());
    }

    #[test]
    fn ditch_punctuation() {
        assert_eq!("the", clean_word("the?").unwrap());
        assert_eq!("the", clean_word("!the").unwrap());
        assert_eq!("the", clean_word("!t.he").unwrap());
    }

    #[test]
    fn ditch_capitals() {
        assert_eq!("the", clean_word("tHe").unwrap());
        assert_eq!("the", clean_word("tHE").unwrap());
        assert_eq!("the", clean_word("The").unwrap());
    }

    #[test]
    fn empty_result() {
        assert_eq!(None, clean_word(" ? "));
    }
}
