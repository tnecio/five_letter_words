mod backtracking_brute;
mod smart_brute;
mod word_reprs;

use std::env;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use crate::backtracking_brute::{backtracking_brute, backtracking_brute_parallelized};

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = Path::new(&args[1]);
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", path.display(), why),
        Ok(file) => file,
    };
    
    let words: Vec<_> =
        BufReader::new(file)
            .lines()
            .map(|l| unpack_word(l))
            .filter(|w| is_unique_5_letter(w))
            .collect();

    backtracking_brute_parallelized(words);
}

fn unpack_word<T>(line: Result<String, T>) -> String {
    if let Ok(line) = line { line } else { String::from("") }
}

fn is_unique_5_letter(word: &String) -> bool {
    let mut s = word.clone();
    s.truncate(s.trim_end().len());
    if s.chars().count() != 5 {
        return false;
    }
    // Check how many unique chars is there in word
    // TODO: consider using a different method to avoid the set allocation
    let mut chars : HashSet<char> = HashSet::new();
    s.chars().for_each(|ch| { chars.insert(ch.clone()); });
    return chars.len() == 5;
}
