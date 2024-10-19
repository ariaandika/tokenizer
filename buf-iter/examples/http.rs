use std::str::from_utf8;
use buf_iter::{BufIter, Result};

const SRC: &[u8] = b"\
GET /health?id=4 HTTP/1.1\r
Client-Agent: idk\r
\r\n";

fn tostr(buf: &[u8]) -> &str {
    from_utf8(buf).unwrap()
}

// wrapper to use Display instead of Debug
fn main() {
    if let Err(err) = main2() {
        println!("{err}");
    }
}

fn main2() -> Result<()> {
    let mut iter = BufIter::new(SRC);

    let method = iter.collect_ident()?;
    let method = tostr(method.evaluate(SRC));

    iter.next_as::<b' '>()?;

    let path = iter.collect_with(|e|!matches!(e,b'?'|b' '))?;
    let path = tostr(path.evaluate(SRC));

    let query = iter.collect_as::<b' '>()?;
    let query = tostr(query.evaluate(SRC));

    iter.next_as::<b' '>()?;

    let version = iter.collect_as::<b'\r'>()?;
    let version = tostr(version.evaluate(SRC));

    assert_eq!(method, "GET");
    assert_eq!(path, "/health");
    assert_eq!(query, "?id=4");
    assert_eq!(version, "HTTP/1.1");

    iter.next_as::<b'\r'>()?;
    iter.next_as::<b'\n'>()?;

    println!("Method: {method}, Path: {path}, Query: {query}, Version: {version}");

    loop {
        if iter.peek_required()? == &b'\r' { break; }

        let key = iter.collect_as::<b':'>()?;
        let key = tostr(key.evaluate(SRC));

        iter.next_as::<b':'>()?;
        iter.skip_whitespaces();

        let val = iter.collect_as::<b'\r'>()?;
        let val = tostr(val.evaluate(SRC));

        iter.next_as::<b'\r'>()?;
        iter.next_as::<b'\n'>()?;

        println!("{key}: {val}");
    }

    iter.next_as::<b'\r'>()?;
    iter.next_as::<b'\n'>()?;

    Ok(())
}

