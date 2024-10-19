use buf_iter::{BufIter, Result, Span};

fn main() {
    let buf = include_bytes!("./index.html");
    let mut input = BufIter::new(buf);
    let mut tokens = vec![];

    if let Err(err) = doc(&mut input, &mut tokens) {
        panic!("{err}: {:?}",err.span);
    }

    for token in tokens {
        match token {
            Token::Text(text) => {
                dbg!(text);
            }
            Token::Ident(ident) => {
                dbg!(core::str::from_utf8(ident.span.evaluate(buf)));
                dbg!(core::str::from_utf8(ident.ident_span.evaluate(buf)));
            }
        }
    }
}

// peeker, doesnt consume iterator
fn doc(input: &mut BufIter, tokens: &mut Vec<Token>) -> Result<()> {
    match input.peek() {
        Some(b'{') => expr(input, tokens),
        Some(_) => text(input, tokens),
        None => Ok(())
    }
}

// consumer and peeker
fn text(input: &mut BufIter, tokens: &mut Vec<Token>) -> Result<()> {
    input.next_peeked();
    let span = input.span();

    loop {
        match input.peek() {
            Some(b'{') => {
                tokens.push(Token::Text(Text { span: span.into_spanned(&input.span()) }));
                expr(input, tokens)?;
                break
            }
            Some(_) => input.next_peeked(),
            None => {
                tokens.push(Token::Text(Text { span: span.into_spanned(&input.span()) }));
                break;
            }
        }
    }

    Ok(())
}

// consumer
fn expr(input: &mut BufIter, tokens: &mut Vec<Token>) -> Result<()> {
    input.next_peeked();
    let span = input.span();

    loop {
        match input.peek() {
            Some(b'@') => {
                input.next_peeked();
                input.skip_whitespaces();
                let ident_span = input.collect_ident()?;
                input.skip_whitespaces();
                input.next_as::<b'}'>()?;

                tokens.push(Token::Ident(Ident {
                    span: span.into_spanned(&input.span()),
                    ident_span
                }));
                doc(input, tokens)?;
                break
            }
            Some(b) => return Err(input.error(buf_iter::ErrorKind::ExpectFound(b'@', *b))),
            None => {
                break
            }
        }
    }


    Ok(())
}

#[derive(Debug)]
enum Token {
    Text(Text),
    Ident(Ident),
}

#[derive(Debug)]
struct Text {
    span: Span
}

#[derive(Debug)]
struct Ident {
    span: Span,
    ident_span: Span,
}

