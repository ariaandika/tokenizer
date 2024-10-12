# Basic Tokenizer, Lexer, Parser, or whatever

inspired by rust `syn` and `proc_macro`

## Tokenizer

Tokenize a stream of bytes, into collection of token trees

Every tokens does not contain the actual value, but instead it holds a `Span`. Span contains 'pointer' to
the actual value in source code. To get the actual value, we can `evaluate` based on source code. This required
the caller to hold the source reference themself. In exchange, we only allocate numbers when tokenizing.

This is not a general tokenizer, because other kind of tokens can have other rules that cannot overlap,
and its not worth to creating another abstraction layer. Instead, specialized tokenizer usually created
on its own, which also can derived from this tokenizer. That also make this tokenizer infallible.

Other kinds of tokenizer mentioned in [extra](#extra)

### `TokenTree`

possible types of token:

- `Ident`
- `Punct`
- `Whitespace`

for more detail, see the generated documentation

```bash
cargo doc --open
```

## Extra

### HTML Parser

Derived from `tokenizer`, we can create a html parser.

Here, we parse open or close element, not the whole element with its children. This is to avoid allocating
new vector when iterating. So the result is a one dimensional tokens. Attributes also not parsed, only validated,
with same the reason above, to avoid allocating new vector. We can iterate attribute on its own if needed.

### `SyntaxTree`

possible types of token:

- `DOCTYPE`, html doctype `<!DOCTYPE html>`
- `Comment`, html comment, `<!-- any value -->`
- `Element`, open or close html element, attributes are only validated
- `Text`, others

