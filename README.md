# mdbook-nocomment

[![Crates.io](https://img.shields.io/crates/v/mdbook-nocomment.svg)](https://crates.io/crates/mdbook-nocomment)

A simple [mdbook](https://rust-lang.github.io/mdBook/index.html) preprocessors to clean up html comments in the book.

## Usage

To use, install the tool

```console
$ cargo install mdbook-nocomment
```

activate as a preprocessor in book.toml:

```toml
[preprocessor.nocomment]
```
