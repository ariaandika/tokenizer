use ::tokenizer::{span::{Span, Spanned}, tokenizer::{Peekable as Peekable1, Tokenizer as Tokenizer1}, TokenTree as Tree1};
use error::{Error, Result};

macro_rules! next {
    ($iter:ident) => {
        match $iter.next() {
            Some(next) => next,
            None => return Err(Error::new($iter.span(), "unexpected eof")),
        }
    };
}

macro_rules! peek {
    ($iter:ident) => {
        match $iter.peek() {
            Some(next) => next,
            None => return Err(Error::new($iter.span(), "unexpected eof")),
        }
    };
}

macro_rules! peeked {
    () => { concat!("[",line!(),":",column!(),"] should be peeked before") };
}

/// tokens that can appear in html
#[derive(Debug)]
pub enum SyntaxTree {
    /// `<!-- comment -->`
    Comment(Comment),
    /// `<!DOCTYPE html>`
    DOCTYPE(DOCTYPE),
    Element(Element),
    Text(Text),
}

/// `<!-- comment -->`
#[derive(Debug)]
pub struct Comment {
    span: Span,
}

impl Comment {
    fn peek(iter: &mut Peekable1<4>, buf: &[u8]) -> bool {
        if !matches!(iter.peek_n(0),Some(Tree1::Punct(punct)) if punct.evaluate(buf)[0] == b'<') {
            return false;
        }
        if !matches!(iter.peek_n(1),Some(Tree1::Punct(punct)) if punct.evaluate(buf)[0] == b'!') {
            return false;
        }
        if !matches!(iter.peek_n(2),Some(Tree1::Punct(punct)) if punct.evaluate(buf)[0] == b'-') {
            return false;
        }
        if !matches!(iter.peek_n(3),Some(Tree1::Punct(punct)) if punct.evaluate(buf)[0] == b'-') {
            return false;
        }
        true
    }

    fn parse(iter: &mut Peekable1<4>, buf: &[u8]) -> Result<Self> {
        eprintln!("parsing Comment");
        let tree = iter.next().expect(peeked!());
        let _ = iter.next().expect(peeked!());
        let _ = iter.next().expect(peeked!());
        let _ = iter.next().expect(peeked!());

        let mut span = tree.span();

        'outer: loop {
            match next!(iter) {
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'-' => {}
                _ => { continue }
            }

            match next!(iter) {
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'-' => {}
                _ => { continue }
            }

            loop {
                match next!(iter) {
                    Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'>' => { break 'outer }
                    Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'-' => { continue }
                    _ => { continue 'outer; }
                }
            }
        }

        span.spanned_into(iter.span());

        Ok(Self { span })
    }
}

/// `<!DOCTYPE html>`
#[derive(Debug)]
pub struct DOCTYPE {
    span: Span
}

impl DOCTYPE {
    fn peek(iter: &mut Peekable1<4>, buf: &[u8]) -> bool {
        match iter.peek_n(0) {
            Some(Tree1::Punct(punct)) if punct.evaluate(buf)[0] == b'<' => {},
            _ => return false
        }
        match iter.peek_n(1) {
            Some(Tree1::Punct(punct)) if punct.evaluate(buf)[0] == b'!' => {},
            _ => return false
        }
        true
    }

    fn parse(iter: &mut Peekable1<4>, buf: &[u8]) -> Result<Self> {
        eprintln!("parsind DOCTYPE");
        let tree = iter.next().expect(peeked!());
        let _ = iter.next().expect(peeked!());

        let mut span = tree.span();

        loop {
            match next!(iter) {
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'>' => break,
                _ => {}
            }
        }

        span.spanned_into(iter.span());

        Ok(Self { span })
    }
}

#[derive(Debug)]
pub struct Element {
    kind: ElementKind,
    span: Span,
    tag_span: Span,
}

#[derive(Debug)]
pub enum ElementKind {
    Open,
    Close,
}

impl Element {
    fn peek(iter: &mut Peekable1<4>, buf: &[u8]) -> bool {
        matches!(iter.peek_n(0),Some(Tree1::Punct(punct)) if punct.evaluate(buf)[0] == b'<')
    }

    fn parse(iter: &mut Peekable1<4>, buf: &[u8]) -> Result<Self> {
        eprintln!("parsing Element");
        let lt = iter.next().expect(peeked!());
        let (tag,kind) = 'out: {
            match next!(iter) {
                Tree1::Ident(tag) => break 'out (tag,ElementKind::Open),
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'/' => {}
                _ => return Err(Error::new(iter.span(), "expected `/` or an identifier"))
            }
            match next!(iter) {
                Tree1::Ident(tag) => break 'out (tag,ElementKind::Close),
                _ => return Err(Error::new(iter.span(), "expected an identifier"))
            }
        };

        let mut span = lt.span();

        if let ElementKind::Close = kind {
            loop {
                match next!(iter) {
                    Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'>' => break,
                    Tree1::Whitespace(_) => continue,
                    _ => return Err(Error::new(iter.span(), "expected `>`"))
                }
            }
            span.spanned_into(iter.span());
            return Ok(Self { kind, span, tag_span: tag.span() });
        }

        // attributes
        loop {
            match peek!(iter) {
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'>' => break,
                Tree1::Whitespace(_) => {
                    iter.next().expect(peeked!());
                    continue
                }
                _ => Attr::scan(iter, buf)?,
            }
        }

        let _gt = iter.next().expect(peeked!());

        span.spanned_into(iter.span());

        Ok(Self { span, kind, tag_span: tag.span() })
    }
}

pub struct Attr;

impl Attr {
    /// consume iterator of one attribute
    fn scan(iter: &mut Peekable1<4>, buf: &[u8]) -> Result<()> {
        // key
        loop {
            match next!(iter) {
                Tree1::Ident(_) => break,
                Tree1::Whitespace(_) => continue,
                Tree1::Punct(_) => return Err(Error::new(iter.span(), "expected an identifier")),
            }
        }

        // eq
        loop {
            match peek!(iter) {
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'=' => {
                    iter.next().expect(peeked!());
                    break
                }
                Tree1::Whitespace(_) => {
                    iter.next().expect(peeked!());
                    continue
                }
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'>' => return Ok(()),
                Tree1::Punct(_) => return Err(Error::new(iter.span(), "expected `=` or `>`")),
                Tree1::Ident(_) => return Ok(()),
            }
        }

        // open quote
        loop {
            match peek!(iter) {
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'"' => {
                    iter.next().expect(peeked!());
                    break
                }
                Tree1::Ident(_) |
                Tree1::Punct(_) => return Err(Error::new(iter.span(), "expected `\"`")),
                Tree1::Whitespace(_) => {
                    iter.next().expect(peeked!());
                    continue
                }
            }
        }

        // close quote
        loop {
            match next!(iter) {
                Tree1::Punct(punct) if punct.evaluate(buf)[0] == b'"' => break,
                _ => continue,
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Text {
    span: Span
}

impl Text {
    fn parse(iter: &mut Peekable1<4>, buf: &[u8]) -> Result<Self> {
        eprintln!("parsing Text");
        let tree = iter.next().expect(peeked!());
        let mut span = tree.span();

        loop {
            match iter.peek() {
                Some(Tree1::Punct(punct)) if punct.evaluate(buf)[0] == b'<' => break,
                Some(_) => { iter.next().expect(peeked!()); },
                None => break,
            }
        }

        span.spanned_into(iter.span());

        Ok(Self { span })
    }
}

pub mod tokenizer {
    use crate::{error::{Error, Result}, Comment, Element, Peekable1, SyntaxTree, Text, Tokenizer1, DOCTYPE};

    /// tokenizer iterator are fallible
    ///
    /// when error occurs, its more likely the parsing did not proceed fully. Calling next in this
    /// state will continue parsing and may resulting in premature parsing, so its recommended to
    /// terminate iterator when error occurs.
    ///
    /// we can use [`std::result::Result`]'s [`std::iter::FromIterator`] when `collect`ing
    ///
    /// ```
    /// let tokenizer = Tokenizer::new(b"source");
    /// let result: Result<Vec<SyntaxTree>> = tokenizer.collect();
    /// ```
    #[derive(Debug)]
    pub struct Tokenizer<'r> {
        buf: &'r [u8],
        iter: Peekable1<'r,4>,
    }

    impl<'r> Tokenizer<'r> {
        pub fn new(src: &'r [u8]) -> Self {
            Self { buf: src, iter: Tokenizer1::new(src).peekable_tokens() }
        }
    }

    macro_rules! nerr {
        ($ex:expr) => {
            match $ex {
                Ok(ok) => ok,
                Err(err) => return Some(Err(err)),
            }
        };
    }

    impl<'r> Iterator for Tokenizer<'r> {
        type Item = Result<SyntaxTree>;

        fn next(&mut self) -> Option<Self::Item> {
            let tree = match () {
                _ if Comment::peek(&mut self.iter, &self.buf)
                    => SyntaxTree::Comment(nerr!(Comment::parse(&mut self.iter, &self.buf))),
                _ if DOCTYPE::peek(&mut self.iter, &self.buf)
                    => SyntaxTree::DOCTYPE(nerr!(DOCTYPE::parse(&mut self.iter, &self.buf))),
                _ if Element::peek(&mut self.iter, &self.buf)
                    => SyntaxTree::Element(nerr!(Element::parse(&mut self.iter, &self.buf))),
                _ => if self.iter.peek().is_some() {
                    SyntaxTree::Text(nerr!(Text::parse(&mut self.iter, &self.buf)))
                } else {
                    return None
                },
            };

            Some(Ok(tree))
        }
    }

}

pub mod error {
    use ::tokenizer::span::Span;

    #[derive(Debug)]
    pub struct Error {
        span: Span,
        msg: &'static str,
    }

    pub type Result<T,E = Error> = std::result::Result<T,E>;

    impl Error {
        pub fn new(span: Span, msg: &'static str) -> Self {
            Self { span, msg }
        }
    }

    impl std::error::Error for Error { }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let (line,col) = self.span.line_col();
            write!(f, "[{line}:{col}] ")?;
            write!(f, "{}", self.msg)
        }
    }
}

mod impls {
    use super::*;

    impl Spanned for Comment {
        fn span(&self) -> Span {
            self.span.clone()
        }
    }

    impl Spanned for DOCTYPE {
        fn span(&self) -> Span {
            self.span.clone()
        }
    }

    impl Spanned for Element {
        fn span(&self) -> Span {
            self.span.clone()
        }
    }

    impl Spanned for Text {
        fn span(&self) -> Span {
            self.span.clone()
        }
    }

    impl Spanned for SyntaxTree {
        fn span(&self) -> Span {
            match self {
                SyntaxTree::Comment(comment) => comment.span(),
                SyntaxTree::DOCTYPE(doctype) => doctype.span(),
                SyntaxTree::Element(element) => element.span(),
                SyntaxTree::Text(text) => text.span(),
            }
        }
    }

}

