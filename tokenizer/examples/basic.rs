use core::str;
use std::fs;
use tokenizer::{span::Spanned, tokenize};

fn main() {
    let html = format!("{}/examples/index.html",env!("CARGO_MANIFEST_DIR"));
    let src = fs::read(html).unwrap();
    let tokens = tokenize(&src);

    for tree in tokens {
        let value = str::from_utf8(tree.evaluate(&src)).unwrap();
        dbg!((tree,value));
    }
}

