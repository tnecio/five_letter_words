use std::collections::HashMap;

pub fn build_word_representations(words: &Vec<String>) -> (HashMap<u32, Vec<&str>>, Vec<u32>) {
    let mut reprs: HashMap<u32, Vec<&str>> = HashMap::new();
    let mut keys: Vec<u32> = Vec::new();
    for word in words.iter() {
        let repr = get_repr(word);
        if reprs.contains_key(&repr) {
            reprs.get_mut(&repr).unwrap().push(word);
        } else {
            let mut v : Vec<&str> = Vec::new();
            v.push(word);
            reprs.insert(repr, v);
            keys.push(repr);
        }
    }
    keys.sort();
    (reprs, keys)
}

pub fn get_repr(word: &str) -> u32 {
    let mut res = 0u32;
    for ch in word.chars() {
        let val = ch as u32;
        res |= 1 << (val - ('a' as u32));
    }
    res
}