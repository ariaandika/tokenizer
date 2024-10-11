# Basic Tokenizer, Lexer, or whatever

inspired by rust `syn` and `proc_macro`

# Tokenizer

tokenize a stream of bytes, into collection of token trees

this is not a general tokenizer, because other kind of tokens can have other rules that cannot overlap each other,
and its not worth to creating another abstraction layer. Other kinds of tokenizer mentioned in [extra](#Extra)

every tokens does not contain the actual value, but instead a [`Span`](##Span). Span contains information of the
actual value in source.

## `TokenTree`

possible types of token:

- `Ident`
- `Punct`
- `Whitespace`

### `Ident`

a single word

rules:

- a collection of alphabetic, numeric, and underscore
- cannot starts with numeric

### `Punct`

a single punctuation

rules:

- anything other than specified in identifiers
- anything other than whitespace

### `Whitespace`

a whitespaces

rules:

- collection of spaces, newlines, and tabs

> it uses rules specified in rust's `core::num::is_ascii_whitespace`

## `Span`

a token map of the actual source

a single span contains:

- `offset`, 0 indexed position
- `len`, length
- `line`, 1 indexed line
- `col`, 1 indexed column

# TODO

- lexer
- parser

# Extra

## HTML Tokenizer

an identifier can contains `-`

literal is only string

