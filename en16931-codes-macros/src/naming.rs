//! The names of the variants built from the names of codes.

use crate::prelude::{HashMap, HashSet, ToPascalCase};

// The apostrophes the names of codes use: the typewriter one and the typographic one.
const APOSTROPHES: [char; 2] = ['\'', '\u{2019}'];

// The Latin letters with diacritics the names of codes use, and their base letters.
const LETTERS: [(&str, char); 14] = [
    ("ÀÁÂÃÄÅ", 'A'),
    ("àáâãäå", 'a'),
    ("Ç", 'C'),
    ("ç", 'c'),
    ("ÈÉÊË", 'E'),
    ("èéêë", 'e'),
    ("ÌÍÎÏ", 'I'),
    ("ìíîï", 'i'),
    ("Ñ", 'N'),
    ("ñ", 'n'),
    ("ÒÓÔÕÖØ", 'O'),
    ("òóôõöø", 'o'),
    ("ÙÚÛÜ", 'U'),
    ("ùúûü", 'u'),
];

/// Names the variants of the codes of a list, given as pairs of a code and a name.
///
/// The variants of equal names get the code as a suffix.
pub(crate) fn variants(list: &str, codes: &[(String, String)]) -> Vec<String> {
    let names: Vec<_> = codes.iter().map(|(_, name)| variant(name)).collect();
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for name in &names {
        *counts.entry(name.as_str()).or_default() += 1;
    }
    let mut seen = HashSet::new();
    codes
        .iter()
        .zip(&names)
        .map(|((code, _), name)| {
            let variant = if counts[name.as_str()] > 1 {
                format!("{name}{}", words(code).to_pascal_case())
            } else {
                name.clone()
            };
            assert!(
                seen.insert(variant.clone()),
                "list {list}: the variant `{variant}` of the code `{code}` repeats"
            );
            variant
        })
        .collect()
}

// Builds the variant name of a code from its name, spelling out a leading number.
fn variant(name: &str) -> String {
    let digits: String = name.chars().take_while(char::is_ascii_digit).collect();
    let name = if digits.is_empty() {
        name.to_owned()
    } else {
        let rest = &name[digits.len()..];
        let value: u32 = digits.parse().expect("a leading number fits u32");
        let suffix = ["st", "nd", "rd", "th"]
            .into_iter()
            .find(|suffix| rest.to_ascii_lowercase().starts_with(suffix));
        match suffix {
            Some(suffix) => format!("{} {}", ordinal(value), &rest[suffix.len()..]),
            None => format!("{} {rest}", cardinal(value)),
        }
    };
    let variant = words(&name).to_pascal_case();
    assert!(
        variant.starts_with(|c: char| c.is_ascii_alphabetic()),
        "the name {name:?} gives no valid variant name"
    );
    variant
}

// Drops the apostrophes, which join the words of a possessive, replaces the letters
// with diacritics by their base letters, and every other character but a letter
// or a digit by a space.
fn words(name: &str) -> String {
    name.chars()
        .filter(|c| !APOSTROPHES.contains(c))
        .map(|c| {
            LETTERS
                .iter()
                .find(|(letters, _)| letters.contains(c))
                .map_or(c, |&(_, base)| base)
        })
        .map(|c| if c.is_ascii_alphanumeric() { c } else { ' ' })
        .collect()
}

// Spells out a number in English words.
fn cardinal(value: u32) -> String {
    const ONES: [&str; 20] = [
        "zero",
        "one",
        "two",
        "three",
        "four",
        "five",
        "six",
        "seven",
        "eight",
        "nine",
        "ten",
        "eleven",
        "twelve",
        "thirteen",
        "fourteen",
        "fifteen",
        "sixteen",
        "seventeen",
        "eighteen",
        "nineteen",
    ];
    const TENS: [&str; 10] = [
        "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
    ];
    match value {
        0..20 => ONES[value as usize].to_owned(),
        20..100 => match value % 10 {
            0 => TENS[(value / 10) as usize].to_owned(),
            ones => format!("{} {}", TENS[(value / 10) as usize], ONES[ones as usize]),
        },
        100..1000 => match value % 100 {
            0 => format!("{} hundred", ONES[(value / 100) as usize]),
            rest => format!(
                "{} hundred {}",
                ONES[(value / 100) as usize],
                cardinal(rest)
            ),
        },
        _ => match value % 1000 {
            0 => format!("{} thousand", cardinal(value / 1000)),
            rest => format!("{} thousand {}", cardinal(value / 1000), cardinal(rest)),
        },
    }
}

// Spells out an ordinal number in English words.
fn ordinal(value: u32) -> String {
    let cardinal = cardinal(value);
    let (head, last) = cardinal.rsplit_once(' ').unwrap_or(("", &cardinal));
    let last = match last {
        "one" => "first".to_owned(),
        "two" => "second".to_owned(),
        "three" => "third".to_owned(),
        "five" => "fifth".to_owned(),
        "eight" => "eighth".to_owned(),
        "nine" => "ninth".to_owned(),
        "twelve" => "twelfth".to_owned(),
        word if word.ends_with('y') => format!("{}ieth", &word[..word.len() - 1]),
        word => format!("{word}th"),
    };
    if head.is_empty() {
        last
    } else {
        format!("{head} {last}")
    }
}
