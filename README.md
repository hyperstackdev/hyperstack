# HyperStack

Rust-based code graph for parsing and traversing source code using natural language.

> ⚠️ We're currently building out full TypeScript support, with more languages to come

Parses and embeds locally using [tree-sitter](https://tree-sitter.github.io/tree-sitter/) and [fastembed](https://docs.rs/fastembed/latest/fastembed/).

Running `cargo run` will download and cache [Jina Embeddings V2](https://huggingface.co/jinaai/jina-embeddings-v2-base-code) for future builds. A lightweight TUI will boot up allowing you to run basic queries against our example source code in `src/example.rs`.
