# Percival

Percival is a declarative _data query and visualization language_. It provides a
reactive, web-based notebook environment for exploring complex datasets,
producing interactive graphics, and sharing results.

Percival combines the interactive beauty of
[_Vega_](https://vega.github.io/vega/)-like visualization grammars with the
flexibility of [_Datalog_](https://en.wikipedia.org/wiki/Datalog) as a query
language for structured, relational data. These declarative components are
combined through a reactive dataflow system, making it easy to quickly
investigate datasets. Because Percival is built on web technologies,
fully-interactive notebooks can be published to anyone with access to a web
browser, making analyses more tangible to others.

In addition to using Vega-Lite, Percival ships with a custom Datalog compiler
that integrates with its notebook runtime. This compiles the query language to
JavaScript through a staged evaluation process, which can be extended with
user-provided JavaScript code. The interface aims to be lightweight, friendly,
and accessible, and it has no hidden workspace state.

## Getting Started

Building Percival from scratch requires [Node v16+](https://nodejs.org/en/),
[NPM v8+](https://www.npmjs.com/), [Rust 1.56+](https://www.rust-lang.org/),
[Cargo](https://crates.io/), and
[Wasm-Pack](https://rustwasm.github.io/wasm-pack/) installed on your machine. To
build the Rust/WebAssembly portion of the project, use the command:

```shell
wasm-pack build --target web crates/percival-wasm
```

Next, run `npm install` to install JavaScript dependencies, then run the
following command to start the development server:

```shell
npm run dev
```

This should open a Percival notebook in your browser.

## Development

To build, lint, and format the Svelte project, use the corresponding scripts:

```shell
npm run build
npm run check
npm run format
```

For the Rust crates, you can run automated tests for the core functionality
with:

```shell
cargo test
```

You can also run tests for the WebAssembly component using a headless Chrome or
Firefox browser:

```shell
wasm-pack test --chrome --headless rustpad-wasm
```
