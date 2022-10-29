// Goal 6: Parallelize

use std::collections::HashMap;
use std::io::prelude::*;
use itertools::Itertools;
use rayon::prelude::*;

use crate::word_reprs::*;

pub fn backtracking_brute(words: Vec<String>) {
    let (reprs, keys) = build_word_representations(&words);
    aux(0, [0; 5], 0, 0, &keys, &reprs);
}

pub fn backtracking_brute_parallelized(words: Vec<String>) {
    let (reprs, keys) = build_word_representations(&words);
    (&keys).into_iter().enumerate().collect::<Vec<_>>()
    .par_chunks(keys.len() / 24)
    .for_each(|chunk| {
        for (pos, key) in chunk {
            aux(1, [**key, 0, 0, 0, 0], **key, *pos, &keys, &reprs);
        }
    });
}

fn aux(depth: u8, combo: [u32; 5], combo_repr: u32, pos: usize, keys: &Vec<u32>,
       reprs: &HashMap<u32, Vec<&str>>) {
    if depth == 5 {
        combo_pretty_print(combo, reprs);
        return;
    }

    let mut new_combo: [u32; 5] = combo.clone();
    for (pos2, key) in keys.iter().skip(pos).enumerate() {
        if key & combo_repr != 0 {
            continue;
        }
        new_combo[depth as usize] = *key;
        let new_combo_repr = combo_repr | key;
        let new_pos = pos + pos2;
        aux(depth + 1, new_combo, new_combo_repr, new_pos, keys, reprs);
    }
}

fn combo_pretty_print(combo: [u32; 5], reprs: &HashMap<u32, Vec<&str>>) {
    for word_combo in combo.iter().map(|key| reprs.get(key).unwrap()).multi_cartesian_product() {
        let word_combo = word_combo.into_iter().fold(String::new(), |mut acc, word| { acc.push_str(word); acc.push_str(" "); acc });
        let _ = writeln!(std::io::stdout(), "{}", word_combo);
    }
}
