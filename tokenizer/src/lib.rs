//! a tokenizer
//!
//! the root module contains the [`TokenTree`] specification
//!
//! the actual tokenizer is contained in [`tokenizer`]
use span::Span;
use tokenizer::{Tokenizer, BufIter};

/// helper to quickly tokenize a source
///
/// for more control, use [`Tokenizer`]
pub fn tokenize(src: &[u8]) -> Vec<TokenTree> {
    Tokenizer::new(src).collect()
}

/// a single token
#[derive(Debug)]
pub enum TokenTree {
    Ident(Ident),
    Punct(Punct),
    Whitespace(Whitespace),
}

/// a word consists of alphabetical, numeric, and underscore
///
/// note that identifier may starts with number
#[derive(Debug)]
pub struct Ident {
    span: Span,
}

impl Ident {
    /// is byte qualified as identifier
    #[inline]
    fn peek(byte: &u8) -> bool {
        matches!(byte,b'A'..=b'Z'|b'a'..=b'z'|b'_'|b'0'..=b'9')
    }

    /// consume iterator resulting identifier
    fn parse(iter: &mut BufIter<'_>) -> Self {
        let (mut span, _) = iter.next().expect("should be peeked before");

        loop {
            match iter.peek() {
                Some(byte) if Self::peek(byte) => {
                    let (end_span, _) = iter.next().unwrap();
                    span.spanned_into(end_span);
                },
                _ => break
            }
        }


        Self { span }
    }
}

/// a punctuation, which anything other than identifier or whitespace
#[derive(Debug)]
pub struct Punct {
    span: Span,
}

impl Punct {
    /// consume iterator resulting punctuation
    fn parse(iter: &mut BufIter<'_>) -> Self {
        let (span, _) = iter.next().expect("should be peeked before");
        Self { span }
    }
}

/// a whitespace, which specified in [`u8::is_ascii_whitespace`]
#[derive(Debug)]
pub struct Whitespace {
    span: Span,
}

impl Whitespace {
    /// is byte qualified as whitespace, see [`u8::is_ascii_whitespace`]
    #[inline]
    fn peek(byte: &u8) -> bool {
        byte.is_ascii_whitespace()
    }

    /// consume iterator resulting whitespaces
    fn parse(iter: &mut BufIter<'_>) -> Self {
        let (mut span, _) = iter.next().expect("should be peeked before");

        loop {
            match iter.peek() {
                Some(byte) if Self::peek(byte) => {
                    let (end_span, _) = iter.next().unwrap();
                    span.spanned_into(end_span);
                },
                _ => break
            }
        }

        Self { span }
    }
}


pub mod tokenizer {
    //! the actual tokenizer
    use std::{iter, slice};
    use super::{TokenTree, Ident, Punct, Whitespace};
    use super::span::Span;

    type SlicePeek<'r> = iter::Peekable<slice::Iter<'r,u8>>;

    /// iterator that yield [`TokenTree`]
    #[derive(Debug)]
    pub struct Tokenizer<'r> {
        iter: BufIter<'r>
    }

    impl<'r> Tokenizer<'r> {
        /// create new tokenizer from a source
        pub fn new(buf: &'r [u8]) -> Self {
            Self { iter: BufIter::new(buf) }
        }

        pub fn peekable_tokens(self) -> Peekable<'r> {
            Peekable::new(self)
        }
    }

    impl<'r> Iterator for Tokenizer<'r> {
        type Item = TokenTree;

        fn next(&mut self) -> Option<Self::Item> {
            // tokenizer should not advanced iterator
            // instead the tokens should
            let tree = match self.iter.peek()? {
                byte if byte.is_ascii_whitespace() => TokenTree::Whitespace(Whitespace::parse(&mut self.iter)),
                byte if Ident::peek(byte) => TokenTree::Ident(Ident::parse(&mut self.iter)),
                _ => TokenTree::Punct(Punct::parse(&mut self.iter)),
            };

            Some(tree)
        }
    }

    /// iterator that track [`Span`] and yield a byte from source buffer
    #[derive(Debug)]
    pub struct BufIter<'b> {
        iter: SlicePeek<'b>,
        offset: usize,
        line: usize,
        col: usize,
    }

    impl<'b> BufIter<'b> {
        /// create new [`BufIter`] from source buffer
        pub fn new(buf: &'b [u8]) -> Self {
            Self { iter: buf.iter().peekable(), offset: 0, line: 1, col: 1 }
        }

        /// peek the next byte, see [`std::iter::Peekable::peek`]
        pub fn peek(&mut self) -> Option<&&u8> {
            self.iter.peek()
        }
    }

    impl<'r> Iterator for BufIter<'r> {
        type Item = (Span, &'r u8);

        fn next(&mut self) -> Option<Self::Item> {
            let byte = self.iter.next()?;
            let span = Span::new(self.offset, 1, self.line, self.col);

            self.offset += 1;

            if byte == &b'\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }

            Some((span, byte))
        }
    }

    const PEEK_N: usize = 3;

    #[derive(Debug)]
    pub struct Peekable<'r> {
        iter: Tokenizer<'r>,
        peeked: [Option<TokenTree>;PEEK_N],
    }

    impl<'r> Peekable<'r> {
        fn new(iter: Tokenizer<'r>) -> Self {
            Self { iter, peeked: Default::default() }
        }

        fn peek_n(&mut self, n: usize) -> Option<&TokenTree> {
            let some = &mut self.peeked[n];
            if some.is_none() {
                some.replace(self.iter.next()?);
            }
            self.peeked[n].as_ref()
        }

        pub fn peek(&mut self) -> Option<&TokenTree> {
            self.peek_n(0)
        }

        pub fn peek2(&mut self) -> Option<&TokenTree> {
            self.peek_n(0);
            self.peek_n(1)
        }

        pub fn peek3(&mut self) -> Option<&TokenTree> {
            self.peek_n(0);
            self.peek_n(1);
            self.peek_n(2)
        }
    }

    impl<'r> Iterator for Peekable<'r> {
        type Item = TokenTree;

        fn next(&mut self) -> Option<Self::Item> {
            let one = self.peeked[0].take();

            for offset in 0..PEEK_N-1 {
                let [one,two] = &mut self.peeked[offset..offset + 2] else {
                    unreachable!("{PEEK_N} len array")
                };
                std::mem::swap(one, two);
            }

            match one {
                Some(some) => Some(some),
                None => self.iter.next(),
            }
        }
    }

}

pub mod span {
    //! see [`Span`]
    use super::{TokenTree, Ident, Punct, Whitespace};

    /// map of a character to actual buffer
    #[derive(Debug, Clone)]
    pub struct Span {
        offset: usize,
        len: usize,
        line: usize,
        col: usize,
    }

    impl Span {
        /// create new span
        pub fn new(offset: usize, len: usize, line: usize, col: usize) -> Self {
            Self { offset, len, line, col }
        }

        /// set length to provided span
        pub fn spanned_into(&mut self, span: Span) {
            self.len = span.offset - self.offset + 1;
        }

        /// returns (line, column) of the source
        pub fn line_col(&self) -> (usize,usize) {
            (self.line,self.col)
        }
    }

    /// a trait helper to work with [`Span`]
    pub trait Spanned {
        /// returns this object span
        fn span(&self) -> Span;
        /// evaluate the actual value from source via span
        fn evaluate<'r>(&self, buf: &'r [u8]) -> &'r [u8] {
            let span = self.span();
            &buf[span.offset..span.offset + span.len]
        }
    }

    impl Spanned for Ident {
        fn span(&self) -> Span {
            self.span.clone()
        }
    }

    impl Spanned for Punct {
        fn span(&self) -> Span {
            self.span.clone()
        }
    }

    impl Spanned for Whitespace {
        fn span(&self) -> Span {
            self.span.clone()
        }
    }

    impl Spanned for TokenTree {
        fn span(&self) -> Span {
            match self {
                TokenTree::Ident(ident) => ident.span(),
                TokenTree::Punct(punct) => punct.span(),
                TokenTree::Whitespace(whitespace) => whitespace.span(),
            }
        }
    }

}


