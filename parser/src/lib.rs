use error::{Error, ErrorKind, Result};
use span::Span;

// a parse-able token
pub trait Parse where Self: Sized {
    fn parse(input: &mut Parser) -> Result<Self>;
}

// a parse-able token
pub trait Peek where Self: Sized {
    fn peek(input: &Parser) -> Result<bool>;
}

/// source buffer parser
///
/// there is a couple parsing api:
///
/// - advance by byte, [`Parser::next`], [`Parser::next_as`]
/// - advance by token, [`Parser::parse`]
/// - peeking, [`Parser::peek`], [`Parser::peek_byte`]
/// - utility, [`Parser::skip_whitespaces`]
pub struct Parser<'r> {
    buf: &'r [u8],
    offset: usize,
    line: usize,
    col: usize,
}

impl<'r> Parser<'r> {
    /// create new [`Parser`]
    pub const fn new(buf: &'r [u8]) -> Self {
        Self { buf, offset: 0, line: 1, col: 0 }
    }

    /// create new [`Parser`] starting from given span
    ///
    /// this can be used for partial parsing when reading from io
    pub const fn from_span(buf: &'r [u8], span: Span) -> Self {
        Self { buf, offset: span.offset, line: span.line, col: span.col }
    }

    /// advance cursor forward by byte
    pub fn next(&mut self) -> Result<u8> {
        if self.len() == self.offset {
            return Err(self.eof());
        }

        let val = self.buf[self.offset];

        self.offset += 1;

        if val == b'\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }

        Ok(val)
    }

    /// advanced cursor forward and check if its eq to given byte
    ///
    /// this is convinient function that can check and return detailed error
    pub fn next_as<const B: u8>(&mut self) -> Result<u8> {
        match self.next() {
            Ok(ok) if ok == B => Ok(B),
            Ok(ok) => Err(self.error(ErrorKind::ExpectFound(B, ok))),
            Err(err) if err.is_eof() => Err(Error::new(err.span, ErrorKind::ExpectEof(B))),
            Err(err) => Err(err),
        }
    }

    /// keep [`Parser::next`] if whitespace found
    pub fn skip_whitespaces(&mut self) {
        while let Ok(w) = self.peek_byte() {
            if w.is_ascii_whitespace() {
                self.next().expect("peeked");
            } else {
                break;
            }
        }
    }

    /// call parse for given type
    ///
    /// this will clear leading and trailing whitespaces, see [`Self::skip_whitespaces`]
    pub fn parse<T>(&mut self) -> Result<T> where T: Parse {
        self.skip_whitespaces();
        let res = T::parse(self)?;
        self.skip_whitespaces();
        Ok(res)
    }

    /// peek the next byte without advancing parser
    ///
    /// possible error is only [`ErrorKind::Eof`]
    pub fn peek_byte(&self) -> Result<&u8> {
        match self.buf.get(self.offset) {
            Some(some) => Ok(some),
            None => Err(self.eof()),
        }
    }

    /// peek the next token without advancing parser
    pub fn peek<T>(&self) -> Result<bool> where T: Peek {
        T::peek(self)
    }

    /// remaining byte
    pub const fn remaining(&self) -> usize {
        self.len() - self.offset
    }

    /// return source buffer
    pub const fn source(&self) -> &[u8] {
        self.buf
    }

    /// source buffer len
    pub const fn len(&self) -> usize {
        self.buf.len()
    }

    /// is no remaining bytes
    pub const fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// create [`Span`] of current state
    pub const fn span(&self) -> Span {
        if cfg!(debug_assertions) && self.offset == 0 {
            panic!("at least next should be called once before creating span")
        }
        Span::new(self.offset - 1, 1, self.line, self.col)
    }

    /// create error at current span
    pub const fn error(&self, kind: ErrorKind) -> Error {
        Error::new(self.span(), kind)
    }

    /// create eof error at current span
    pub const fn eof(&self) -> Error {
        Error::eof(self.span())
    }
}

impl<'r> From<&'r [u8]> for Parser<'r> {
    fn from(value: &'r [u8]) -> Self {
        Self::new(value)
    }
}

pub mod span {
    //! a 'pointer' of a value from source buffer
    //!
    //! see [`Span`]

    /// a 'pointer' of a value from source buffer
    ///
    /// the struct only contain 4 usize, which is cheap to clone
    ///
    /// use [`Spanned::evaluate`] to get actual value from given buffer
    #[derive(Debug, Clone)]
    pub struct Span {
        pub offset: usize,
        pub len: usize,
        pub line: usize,
        pub col: usize,
    }

    impl Span {
        /// create new [`Span`]
        pub const fn new(offset: usize, len: usize, line: usize, col: usize) -> Self {
            Self { offset, len, line, col }
        }

        /// create [`Span`] with unknown state, where all value is 0
        pub const fn unknown() -> Self {
            Self { offset: 0, len: 0, line: 0, col: 0 }
        }

        /// is current span unknown, see [`Self::unknown`]
        pub const fn is_unknown(&self) -> bool {
            self.offset == 0 && self.len == 0 && self.line == 0 && self.col == 0
        }

        /// return actual value from given source buffer
        pub fn evaluate<'r>(&self, buf: &'r [u8]) -> &'r [u8] {
            &buf[self.offset..self.offset + self.len]
        }

        /// set length from current span to given span
        pub fn spanned(&mut self, span: &Span) {
            self.len = span.offset - self.offset + 1;
        }

        /// set length from current span to given span
        pub fn into_spanned(mut self, span: &Span) -> Span {
            self.spanned(span);
            self
        }
    }

}

pub mod error {
    //! parsing error
    //!
    //! see [`Error`]
    use crate::span::Span;

    /// parsing error [`std::result::Result`] alias
    pub type Result<T,E = Error> = std::result::Result<T,E>;

    /// parsing error
    #[derive(Debug)]
    pub struct Error {
        pub kind: ErrorKind,
        pub span: Span,
    }

    /// parsing error kind
    #[derive(Debug)]
    pub enum ErrorKind {
        /// unexpected eof
        Eof,
        /// expect `_`, found EOF
        ExpectEof(u8),
        /// expect `_`, found `_`
        ExpectFound(u8,u8),
        /// expect alphabetical, found `_`
        ExpectAlphabetic(u8),
    }

    impl Error {
        /// create new [`Error`]
        pub const fn new(span: Span, kind: ErrorKind) -> Self {
            Self { span, kind }
        }

        /// create new [`Error`] with [`ErrorKind::Eof`]
        pub const fn eof(span: Span) -> Self {
            Self::new(span, ErrorKind::Eof)
        }

        /// is [`ErrorKind::Eof`]
        pub const fn is_eof(&self) -> bool {
            matches!(self.kind,ErrorKind::Eof)
        }
    }

    impl std::error::Error for Error { }

    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            std::fmt::Display::fmt(&self.kind, f)
        }
    }

    impl std::fmt::Display for ErrorKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            use std::fmt::Write;
            match self {
                ErrorKind::Eof => f.write_str("unexpected EOF"),
                ErrorKind::ExpectEof(fd) => {
                    f.write_str("expected ")?;
                    f.write_char(*fd as char)?;
                    f.write_str("found EOF")
                }
                ErrorKind::ExpectFound(ex, fd) => {
                    f.write_str("expected ")?;
                    f.write_char(*ex as char)?;
                    f.write_str("found ")?;
                    f.write_char(*fd as char)
                }
                ErrorKind::ExpectAlphabetic(fd) => {
                    f.write_str("expected alphabetical, ")?;
                    f.write_str("found ")?;
                    f.write_char(*fd as char)
                }
            }
        }
    }

}

pub mod token {
    //! built in tokens act as building block to create more tokens

    use crate::{error::{ErrorKind, Result}, span::Span, Parse, Parser};

    /// parse identifier
    ///
    /// identifer contains alphanumeric and underscore, but cant starts with number
    #[derive(Debug)]
    pub struct Ident {
        pub span: Span,
    }

    impl Parse for Ident {
        fn parse(input: &mut Parser) -> Result<Self> {
            let lead = input.next()?;
            let span = input.span();

            if !lead.is_ascii_alphabetic() && lead != b'_' {
                return Err(input.error(ErrorKind::ExpectAlphabetic(lead)));
            }

            fn check(byte: &u8) -> bool {
                byte.is_ascii_alphanumeric() || byte == &b'_'
            }

            loop {
                match input.peek_byte() {
                    Ok(byte) if check(byte) => input.next().expect("peeked"),
                    Ok(_) => break,
                    Err(err) if err.is_eof() => break,
                    Err(err) => return Err(err),
                };
            }

            Ok(Self { span: span.into_spanned(&input.span()) })
        }
    }

    /// parse phrase
    ///
    /// phrase is sequence of anything but whitespace
    pub struct Phrase {
        pub span: Span,
    }

    impl Parse for Phrase {
        fn parse(input: &mut Parser) -> Result<Self> {
            let lead = input.next()?;
            let span = input.span();

            if lead.is_ascii_whitespace() {
                return Err(input.error(ErrorKind::ExpectAlphabetic(lead)));
            }

            fn check(byte: &u8) -> bool {
                !byte.is_ascii_whitespace()
            }

            loop {
                match input.peek_byte() {
                    Ok(byte) if check(byte) => input.next().expect("peeked"),
                    Ok(_) => break,
                    Err(err) if err.is_eof() => break,
                    Err(err) => return Err(err),
                };
            }

            Ok(Self { span: span.into_spanned(&input.span()) })
        }
    }

    /// quoted literal string
    pub struct LitStr {
        pub span: Span
    }

    impl Parse for LitStr {
        fn parse(input: &mut Parser) -> Result<Self> {
            let quoted = Quoted::new(input)?;

            while quoted.next(input)? {
                input.next()?;
            }

            Ok(Self { span: quoted.span.into_spanned(&input.span()) })
        }
    }

    /// literal quoted string
    pub struct Quoted {
        pub span: Span
    }

    impl Quoted {
        pub fn new(input: &mut Parser<'_>) -> Result<Self> {
            let _quote = input.next_as::<b'"'>()?;
            Ok(Self { span: input.span() })
        }

        pub fn next(&self, input: &mut Parser<'_>) -> Result<bool> {
            if input.peek_byte()? == &b'"' {
                input.next().expect("peeked");
                return Ok(false);
            }
            Ok(true)
        }
    }

}

#[cfg(debug_assertions)]
pub mod test {
    use crate::{token::{Ident, LitStr}, Parser};

    pub fn test() {
        let buf = br#" aoawd awd  " deez app " iaiwdj  aiwjd  aijd "#;
        let mut input = Parser::new(buf);

        loop {
            if input.is_empty() { break; }
            if input.peek_byte().unwrap() == &b'"' {
                let lit = input.parse::<LitStr>().unwrap();
                let lit_val = lit.span.evaluate(buf);
                let s = std::str::from_utf8(lit_val).unwrap();
                dbg!(s);
            } else {
                let ident = input.parse::<Ident>().unwrap();
                let ident_val = ident.span.evaluate(buf);
                let s = std::str::from_utf8(ident_val).unwrap();
                dbg!(s);
            }
        }
    }
}

