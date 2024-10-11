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

for more detail, see the generated documentation

```bash
cargo doc --open
```

# TODO

- lexer
- parser

# Extra

## HTML Tokenizer

an identifier can contains `-`

literal is only string

