use buf_iter::BufIter;

const SRC: &[u8] = b"\
GET /health?id=4 HTTP/1.1\r
Client-Agent: idk\r
Content-Length: 142\r
\r\n";

#[test]
fn http_parse() -> Result<(),Box<dyn std::error::Error>> {
    let mut iter = BufIter::new(SRC);

    let method = iter.collect_ident()?;
    assert_eq!(method.evaluate(SRC), b"GET");

    iter.next_as::<b' '>()?;

    let path = iter.collect_with(|e|!matches!(e,b'?'|b' '))?;
    assert_eq!(path.evaluate(SRC), b"/health");

    let query = iter.collect_as::<b' '>()?;
    assert_eq!(query.evaluate(SRC), b"?id=4");

    iter.next_as::<b' '>()?;

    let version = iter.collect_as::<b'\r'>()?;
    assert_eq!(version.evaluate(SRC), b"HTTP/1.1");

    iter.next_as::<b'\r'>()?;
    iter.next_as::<b'\n'>()?;

    {
        let key = iter.collect_as::<b':'>()?;
        assert_eq!(key.evaluate(SRC), b"Client-Agent");

        iter.next_as::<b':'>()?;
        iter.skip_whitespaces();

        let val = iter.collect_as::<b'\r'>()?;
        assert_eq!(val.evaluate(SRC), b"idk");

        iter.next_as::<b'\r'>()?;
        iter.next_as::<b'\n'>()?;
    }

    {
        let key = iter.collect_as::<b':'>()?;
        assert_eq!(key.evaluate(SRC), b"Content-Length");

        iter.next_as::<b':'>()?;
        iter.skip_whitespaces();

        let val = iter.collect_as::<b'\r'>()?;
        assert_eq!(val.evaluate(SRC), b"142");

        iter.next_as::<b'\r'>()?;
        iter.next_as::<b'\n'>()?;
    }

    iter.next_as::<b'\r'>()?;
    iter.next_as::<b'\n'>()?;
    assert!(iter.is_empty());

    Ok(())
}


