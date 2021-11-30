# Percival

[Percival](https://percival.ink/) is a **declarative data query and
visualization language**. It provides a reactive, web-based notebook environment
for exploring complex datasets, producing interactive graphics, and sharing
results.

Percival combines the flexibility of
[_Datalog_](https://en.wikipedia.org/wiki/Datalog) as a query language for
relational data with the beauty of
[_exploratory visualization grammars_](https://observablehq.com/@observablehq/plot).
These declarative components interact through a reactive dataflow system.
Because Percival uses web technologies (including Web Workers for multithreaded,
sandboxed execution), fully-interactive notebooks can be shared with anyone on
the Internet, making data analyses more tangible to others.

At the core of Percival is a custom Datalog compiler, built with Rust and
WebAssembly, which integrates with its notebook runtime. This compiles the query
language to JavaScript through a staged evaluation process that also allows
users to embed their own JavaScript code. The interface aims to be lightweight,
friendly, and accessible, and there is no hidden workspace state.

This is an early-stage research project, and we welcome your feedback, so please
feel free to say hello at our
[discussions page](https://github.com/ekzhang/percival/discussions)!

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

For the Rust crates, you can run unit tests for the core functionality with:

```shell
cargo test
```

You can also run tests for the WebAssembly component using a headless Chrome or
Firefox browser:

```shell
wasm-pack test --chrome --headless crates/percival-wasm
```

Since Percival uses a Rust-based compiler but outputs JavaScript, the easiest
way to test code generation functionality is within the browser. We use Mocha
and Puppeteer for this, and tests can be run with:

```shell
npm test
```

## Acknowledgement

Created by Eric Zhang ([@ekzhang1](https://twitter.com/ekzhang1)). Licensed
under the [MIT license](LICENSE).
