//! buffer iterator
//!
//! iterate over buffer using [`BufIter`], peek forward and track position with [`Span`]

/// buffer iterator
pub struct BufIter<'r> {
    buf: &'r [u8],
    offset: usize,
    line: usize,
    col: usize,
}

/// Creation
impl<'r> BufIter<'r> {
    /// create new [`BufIter`]
    pub const fn new(buf: &'r [u8]) -> Self {
        Self { buf, offset: 0, line: 1, col: 0 }
    }

    /// create new [`BufIter`] starting from given span
    ///
    /// this can be used for partial parsing when reading from io
    pub const fn from_span(buf: &'r [u8], span: Span) -> Self {
        Self { buf, offset: span.offset, line: span.line, col: span.col }
    }

    /// clone [`BufIter`] starting from current span
    pub const fn fork(&self) -> Self {
        Self::from_span(self.buf, Span::new(self.offset, 1, self.line, self.col)
)
    }
}

/// Iterate forward
impl<'r> BufIter<'r> {
    /// advance cursor forward by a byte
    ///
    /// possible error is only [`ErrorKind::Eof`]
    #[allow(clippy::should_implement_trait)]
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

    /// required next byte to eq with given byte
    ///
    /// convenient function that can check and return detailed error
    pub fn next_as<const B: u8>(&mut self) -> Result<u8> {
        match self.next() {
            Ok(ok) if ok == B => Ok(B),
            Ok(ok) => Err(self.error(ErrorKind::ExpectFound(B, ok))),
            Err(err) if err.is_eof() => Err(self.error(ErrorKind::ExpectEof(B))),
            Err(err) => Err(err),
        }
    }

    /// required next byte to be alphabetic
    ///
    /// see [`u8::is_ascii_alphabetic`] for what considered alphabetic
    ///
    /// convenient function that can check and return detailed error
    pub fn next_alphabetic(&mut self) -> Result<u8> {
        match self.next()? {
            ok if ok.is_ascii_alphabetic() => Ok(ok),
            ok => Err(self.error(ErrorKind::ExpectAlphabetic(ok))),
        }
    }

    /// required next byte to be identifier
    ///
    /// identifier is alphanumeric or `_`
    ///
    /// see [`u8::is_ascii_alphanumeric`] for what considered alphanumeric
    ///
    /// convenient function that can check and return detailed error
    pub fn next_ident(&mut self) -> Result<u8> {
        match self.next()? {
            ok if ok.is_ascii_alphanumeric() => Ok(ok),
            b'_' => Ok(b'_'),
            ok => Err(self.error(ErrorKind::ExpectIdent(ok))),
        }
    }

    /// [`BufIter::next`] and unwrap error
    ///
    /// usually used after peeking
    ///
    /// # Panic
    ///
    /// panic with message "peeked" if error occur
    pub fn next_peeked(&mut self) {
        self.next().expect("peeked");
    }

    /// keep calling [`BufIter::next`] if whitespace found
    ///
    /// see [`u8::is_ascii_whitespace`] for what considered whitespace
    pub fn skip_whitespaces(&mut self) {
        loop {
            match self.peek() {
                Some(b) if b.is_ascii_whitespace() => self.next_peeked(),
                _ => break,
            }
        }
    }

    /// collecting identifier
    ///
    /// rules:
    /// - length at least 1
    /// - cannot starts with numeric
    /// - specified in [`BufIter::next_ident`]
    ///
    /// EOF aware, means will stop instead of return error when EOF after one character
    pub fn collect_ident(&mut self) -> Result<Span> {
        match self.next()? {
            b'_' => {}
            ok if ok.is_ascii_alphabetic() => {}
            _ => return Err(self.error(ErrorKind::Eof)),
        }
        let span = self.span();
        loop {
            match self.peek() {
                Some(ok) if ok.is_ascii_alphanumeric() => self.next_peeked(),
                Some(_) => break,
                None => break,
            }
        }
        Ok(span.into_spanned(&self.span()))
    }

    /// collecting until specified byte found
    ///
    /// at least one byte must be found, otherwise return [`ErrorKind::Unexpected`]
    ///
    /// the predicate byte is not included in returned span
    ///
    /// EOF aware, means will stop instead of return error
    pub fn collect_as<const B: u8>(&mut self) -> Result<Span> {
        match self.next() {
            Ok(ok) if ok != B => {}
            Ok(ok) => return Err(self.error(ErrorKind::ExpectFound(B, ok))),
            Err(err) if err.is_eof() => return Err(self.error(ErrorKind::ExpectEof(B))),
            Err(err) => return Err(err),
        }
        let span = self.span();
        loop {
            match self.peek() {
                Some(some) if some != &B => self.next_peeked(),
                Some(_) => break,
                None => break,
            }
        }
        Ok(span.into_spanned(&self.span()))
    }

    /// collecting with predicate
    ///
    /// keep collecting while predicate return true
    ///
    /// at least one predicate must success, otherwise return [`ErrorKind::Unexpected`]
    ///
    /// EOF aware, means will stop instead of return error
    pub fn collect_with<F>(&mut self, predicate: F) -> Result<Span> where F: Fn(&u8) -> bool {
        match self.next()? {
            b if predicate(&b) => {}
            b => return Err(self.error(ErrorKind::Unexpected(b))),
        }
        let span = self.span();
        loop {
            match self.peek() {
                Some(some) if predicate(some) => self.next_peeked(),
                Some(_) => break,
                None => break,
            }
        }
        Ok(span.into_spanned(&self.span()))
    }
}

/// Peek forward without advancing iterator
impl<'r> BufIter<'r> {
    /// peek the next byte without advancing iterator
    ///
    /// return [`None`] if eof, to return error instead use [`Self::peek_required`]
    pub fn peek(&self) -> Option<&u8> {
        self.buf.get(self.offset)
    }

    /// peek n byte forward without advancing parser
    ///
    /// return [`None`] if eof, to return error instead use [`Self::peek_required`]
    pub fn peek_n(&self, n: usize) -> Option<&u8> {
        self.buf.get(self.offset + n)
    }

    /// [`BufIter::peek`] and return [`Error`] if eof
    pub fn peek_required(&self) -> Result<&u8> {
        match self.peek() {
            Some(some) => Ok(some),
            None => Err(self.eof()),
        }
    }

    /// [`BufIter::peek`] and check if its eq to given byte
    ///
    /// return false if eof
    pub fn peek_as<const B: u8>(&self) -> bool {
        matches!(self.peek(), Some(b) if b == &B)
    }

    /// [`BufIter::peek_required`] and [`BufIter::peek_as`]
    pub fn peek_required_as<const B: u8>(&self) -> Result<bool> {
        Ok(self.peek_required()? == &B)
    }
}

/// Utility
impl<'r> BufIter<'r> {
    /// return source buffer
    pub const fn source(&self) -> &[u8] {
        self.buf
    }

    /// source buffer len
    pub const fn len(&self) -> usize {
        self.buf.len()
    }

    /// remaining byte
    pub const fn remaining(&self) -> usize {
        self.len() - self.offset
    }

    /// is no remaining bytes
    pub const fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    /// create [`Span`] of current iterator
    ///
    /// next should be called at least once
    ///
    /// # Panic
    ///
    /// panic if next have not been called once
    pub const fn span(&self) -> Span {
        #[cfg(debug_assertions)]
        if self.offset == 0 {
            panic!("next should be called at least once before creating span")
        }
        Span::new(self.offset - 1, 1, self.line, self.col)
    }

    /// create [`Error`] at current span
    pub const fn error(&self, kind: ErrorKind) -> Error {
        Error::new(self.span(), kind)
    }

    /// see [`Error::eof`]
    pub const fn eof(&self) -> Error {
        Error::eof(self.span())
    }
}

impl<'r> From<&'r [u8]> for BufIter<'r> {
    fn from(value: &'r [u8]) -> Self {
        Self::new(value)
    }
}

impl<'r> From<(&'r [u8],Span)> for BufIter<'r> {
    fn from((value, span): (&'r [u8],Span)) -> Self {
        Self::from_span(value, span)
    }
}

/// a 'pointer' of a value from source buffer
///
/// the struct only contain 4 usize, which is cheap to clone
///
/// use [`Span::evaluate`] to get actual value from given buffer
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
    pub const fn into_spanned(mut self, span: &Span) -> Span {
        self.len = span.offset - self.offset + 1;
        self
    }
}

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
    /// unexpected `_`
    Unexpected(u8),
    /// expect `_`, found EOF
    ExpectEof(u8),
    /// expect `_`, found `_`
    ExpectFound(u8,u8),
    /// expect alphabetical found `_`
    ExpectAlphabetic(u8),
    /// expect identifier found `_`
    ExpectIdent(u8),
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
            Self::Eof => f.write_str("unexpected EOF"),
            Self::Unexpected(fd) => {
                f.write_str("unexpected `")?;
                f.write_char(*fd as char)?;
                f.write_char('`')
            }
            ErrorKind::ExpectEof(ex) => {
                f.write_str("expect `")?;
                f.write_char(*ex as char)?;
                f.write_str("` found EOF")
            }
            ErrorKind::ExpectFound(ex, fd) => {
                f.write_str("expect `")?;
                f.write_char(*ex as char)?;
                f.write_str("` found `")?;
                f.write_char(*fd as char)?;
                f.write_char('`')
            }
            ErrorKind::ExpectAlphabetic(fd) => {
                f.write_str("expect alphabetical found `")?;
                f.write_char(*fd as char)?;
                f.write_char('`')
            }
            ErrorKind::ExpectIdent(fd) => {
                f.write_str("expect identifier found `")?;
                f.write_char(*fd as char)?;
                f.write_char('`')
            }
        }
    }
}

