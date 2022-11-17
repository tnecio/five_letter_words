extern crate prettytable;

mod backtracking_brute;
// mod dancing_links;
mod dancing_links_soa;
mod smart_brute;
mod word_reprs;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use crate::backtracking_brute::{backtracking_brute, backtracking_brute_parallelized};
use crate::dancing_links_soa::dlx_words;
use crate::smart_brute::smart_brute;

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };

    let words: Vec<_> = BufReader::new(file)
        .lines()
        .map(|l| unpack_word(l))
        .filter(|w| is_unique_5_letter(w))
        .collect();

    if args.len() > 2 {
        match &args[2].as_str() {
            &"brute" => backtracking_brute(words),
            &"brute_par" => backtracking_brute_parallelized(words),
            &"smart_brute_par" => smart_brute(words),
            &"dlx" => dlx_words(words),
            _ => dlx_words(words),
        };
    } else {
        dlx_words(words);
    }
}

fn unpack_word<T>(line: Result<String, T>) -> String {
    if let Ok(line) = line {
        line
    } else {
        String::from("")
    }
}

fn is_unique_5_letter(word: &String) -> bool {
    let mut s = word.clone();
    s.truncate(s.trim_end().len());
    if s.chars().count() != 5 {
        return false;
    }
    // Check how many unique chars is there in word
    // TODO: consider using a different method to avoid the set allocation
    let mut chars: HashSet<char> = HashSet::new();
    s.chars().for_each(|ch| {
        chars.insert(ch.clone());
    });
    return chars.len() == 5;
}
