use itertools::Itertools;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use std::collections::HashMap;
use std::io::prelude::*;

use crate::word_reprs::*;

pub fn smart_brute(words: Vec<String>) {
    let (reprs, keys) = build_word_representations(&words);
    let keys2vec = build_unique_pairs(&keys);
    // build map (combo_key_2) -> [(key1, key2), (pos1, pos2)]
    let mut keys2map: FxHashMap<u32, Vec<([u32; 2], [usize; 2])>> = FxHashMap::default();
    for &(combo_key, key_arr, pos_arr) in keys2vec.iter() {
        if keys2map.contains_key(&combo_key) {
            keys2map
                .get_mut(&combo_key)
                .unwrap()
                .push((key_arr, pos_arr));
        } else {
            keys2map.insert(combo_key, vec![(key_arr, pos_arr)]);
        }
    }
    build_unique_triplets(&keys, &keys2vec, &keys2map, &reprs);
}

fn build_unique_pairs(keys: &Vec<u32>) -> Vec<(u32, [u32; 2], [usize; 2])> {
    // (combo_key, combo, positions)
    let mut res: Vec<(u32, [u32; 2], [usize; 2])> = Vec::new();
    for (pos1, key1) in keys.into_iter().enumerate() {
        for (pos2, key2) in keys.into_iter().enumerate().skip(pos1) {
            if key1 & key2 != 0 {
                continue;
            }
            res.push((key1 | key2, [*key1, *key2], [pos1, pos2]));
        }
    }
    res
}

fn build_unique_triplets(
    keys: &Vec<u32>,
    keys2: &Vec<(u32, [u32; 2], [usize; 2])>,
    keys2map: &FxHashMap<u32, Vec<([u32; 2], [usize; 2])>>,
    reprs: &HashMap<u32, Vec<&str>>,
) {
    keys2
        .into_par_iter()
        .for_each(|(combo_key, [key1, key2], [_pos1, pos2])| {
            for (pos3, key3) in keys.into_iter().enumerate().skip(*pos2) {
                if combo_key & key3 != 0 {
                    continue;
                }
                let triple_combo = combo_key | key3;
                // We have a unique triplet.
                // Calculate the 26 possible two-word combo_keys that will match with this triplet
                let possible_two_word_combo_keys: [u32; 11] =
                    get_matching_two_word_combo_keys(triple_combo);
                for two_word_combo in possible_two_word_combo_keys {
                    match keys2map.get(&two_word_combo) {
                        None => (),
                        Some(v) => {
                            for ([key4, key5], [pos4, _pos5]) in v.iter() {
                                if *pos4 <= pos3 {
                                    // We need strict ordering here! pos1 < pos2 < pos3 < pos4 < pos5
                                    continue;
                                }
                                combo_pretty_print([*key1, *key2, *key3, *key4, *key5], reprs)
                            }
                        }
                    }
                }
            }
        });
}

fn get_matching_two_word_combo_keys(triple_combo: u32) -> [u32; 11] {
    let mut res = [0u32; 11];
    let full = !(0xff << 26) ^ triple_combo;
    let mut index: usize = 0;
    for i in 0..26u32 {
        let new = full & !(1 << i);
        if new != full {
            res[index] = new;
            index += 1;
        }
    }
    res
}

fn combo_pretty_print(combo: [u32; 5], reprs: &HashMap<u32, Vec<&str>>) {
    for word_combo in combo
        .iter()
        .map(|key| reprs.get(key).unwrap())
        .multi_cartesian_product()
    {
        let word_combo = word_combo.into_iter().fold(String::new(), |mut acc, word| {
            acc.push_str(word);
            acc.push_str(" ");
            acc
        });
        let _ = writeln!(std::io::stdout(), "{}", word_combo);
    }
}
