use std::str::from_utf8;
use buf_iter::{BufIter, Result, Span};

const BUF: &[u8] = include_bytes!("./index.html");

fn main() {
    if let Err(err) = main2() {
        eprintln!("Oof: {err:?}\n{err}");
    }
}

fn main2() -> Result<()> {
    let mut input = BufIter::new(BUF);
    let mut is_html = matches!(input.peek_required()?,b'<');
    let mut tokens = vec![];

    while !input.is_empty() {
        if is_html {
            let html = Html::parse(&mut input)?;
            if let Html::EndTag(_) = html {
                is_html = false;
            }

            let val = from_utf8(html.span().evaluate(BUF)).unwrap();
            println!("HTML: {val}");

            tokens.push(Token::Html(html));

        } else {
            let text = input.collect_as::<b'<'>()?;

            let val = from_utf8(text.evaluate(BUF)).unwrap();
            println!("TEXT: {val:?}");

            tokens.push(Token::Text(text));
            is_html = true;

        }
    }

    Ok(())
}

/// one dimensional
#[derive(Debug)]
#[allow(unused)]
enum Token {
    /// see [`Html`]
    Html(Html),
    /// `any value, white space preserved`
    Text(Span),
}

#[derive(Debug)]
enum Html {
    /// `<!`
    StartDocTag(Span),
    /// `</`
    StartCloseTag(Span),
    /// `<`
    StartTag(Span),
    /// `>`
    EndTag(Span),
    /// `=`
    Eq(Span),
    /// `"phone"`, quote is excluded
    Lit(Span),
    /// `some-value`, tag or attr name
    Ident(Span),
}

impl Html {
    fn parse(input: &mut BufIter) -> Result<Self> {
        let token = match input.peek_required()? {
            b'<' => {
                input.next_peeked();
                let span = input.span();

                match input.peek_required()? {
                    b'!' => {
                        input.next_peeked();
                        Self::StartDocTag(span.into_spanned(&input.span()))
                    }
                    b'/' => {
                        input.next_peeked();
                        Self::StartCloseTag(span.into_spanned(&input.span()))
                    }
                    _ => Self::StartTag(input.span()),
                }
            }
            b'>' => {
                input.next_peeked();
                Self::EndTag(input.span())
            }
            b'=' => {
                input.next_peeked();
                Self::Eq(input.span())
            }
            b'"' => {
                input.next_peeked();
                let lit = Self::Lit(input.collect_with(|e|!matches!(e,b'"'))?);
                input.next_as::<b'"'>()?;
                lit
            }
            _ => {
                input.skip_whitespaces();
                Self::Ident(input.collect_with(|e|{
                    !(e.is_ascii_whitespace() || matches!(e,b'='|b'>'))
                })?)
            }
        };

        Ok(token)
    }

    fn span(&self) -> &Span {
        match self {
            Html::StartDocTag(span) => span,
            Html::StartCloseTag(span) => span,
            Html::StartTag(span) => span,
            Html::EndTag(span) => span,
            Html::Eq(span) => span,
            Html::Lit(span) => span,
            Html::Ident(span) => span,
        }
    }
}


