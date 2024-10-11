# Basic Tokenizer, Lexer, or whatever

inspired by rust `syn` and `proc_macro`

# Tokenizer

tokenize a stream of bytes, into collection of token trees

every tokens does not contain the actual value, but instead it holds a [`Span`](##Span).
Span contains 'pointer' to the actual value in source code.

this is not a general tokenizer, because other kind of tokens can have other rules that cannot overlap,
and its not worth to creating another abstraction layer. Instead, specialized tokenizer usually specified
on its own, and can also derived from this tokenizer. That also make this tokenizer infallible.

Other kinds of tokenizer mentioned in [extra](#Extra)

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

- HTML Tokenizer

