# Basic Tokenizer, Lexer, Parser, or whatever

inspired by rust `syn` and `proc_macro`

## Workspace

- `tokenizer`, convert bytes to tokens
- `parser`, more extensible parser
- `html-parser`, the first attempt of parser

## Tokenizer

Tokenize a stream of bytes, into collection of token trees

Every tokens does not contain the actual value, but instead it holds a `Span`. Span contains 'pointer' to
the actual value in source code. To get the actual value, we can `evaluate` based on source code. This required
the caller to hold the source reference themself. In exchange, we only allocate numbers when tokenizing.

This is not a general tokenizer, because other kind of tokens can have other rules that cannot overlap,
and its not worth to creating another abstraction layer. Instead, specialized tokenizer usually created
on its own, which also can derived from this tokenizer. That also make this tokenizer infallible.

### `TokenTree`

possible types of token:

- `Ident`
- `Punct`
- `Whitespace`

for more detail, see the generated documentation

```bash
cargo doc -p tokenizer --open
```

## Parser

More extensible parser, moving out of rust's `Iterator` trait, and make api more like `syn`.

## HTML Parser

The first attempt of parser. Derived from `tokenizer`. HTML tokens itself is pretty simple, so this package is not
really design of extensibility, most of its is hard coded.

Here, we parse open or close element, not the whole element with its children. This is to avoid allocating
new vector when iterating. So the result is a one dimensional tokens. Attributes also not parsed, only validated,
with same the reason above, to avoid allocating new vector. We can iterate attribute on its own if needed.

### `SyntaxTree`

possible types of token:

- `DOCTYPE`, html doctype `<!DOCTYPE html>`
- `Comment`, html comment, `<!-- any value -->`
- `Element`, open or close html element, attributes are only validated
- `Text`, others

