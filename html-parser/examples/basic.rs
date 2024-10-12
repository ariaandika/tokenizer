use core::str;
use std::fs;
use tokenizer::span::Spanned;

fn main() {
    let html = format!("{}/examples/index.html",env!("CARGO_MANIFEST_DIR"));
    let src = fs::read(html).unwrap();
    let result = html_parser::tokenizer::Tokenizer::new(&src).collect::<Result<Vec<_>, _>>();

    match result {
        Ok(tokens) => {
            for tree in tokens {
                let value = str::from_utf8(tree.evaluate(&src)).unwrap();
                dbg!((tree,value));
            }
        },
        Err(err) => eprintln!("Error: {err}"),
    }

}


