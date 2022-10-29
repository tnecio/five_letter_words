pub fn dancing_links(words: Vec<String>) {
    let (reprs, keys) = build_word_representations(&words);
    let combo: [u32; 5] = [0; 5];
    aux(0, combo, 0, 0, &keys, &reprs);
}
