# Percival

[Percival](https://percival.ink/) is a **declarative data query and
visualization language**. It provides a reactive, web-based notebook environment
for exploring complex datasets, producing interactive graphics, and sharing
results.

<p align="center">
  <a href="https://percival.ink/">
    <img src="https://i.imgur.com/zW5cuBH.png" width="720"><br>
    <strong>percival.ink</strong>
  </a>
</p>

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

If you've gotten to this point in the README, please first try out the web
application and demo notebook at [percival.ink](https://percival.ink/)! The
information below is technical documentation intended for contributors.

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

This should open a Percival notebook in your browser, with live reloading.

## Architecture

This section outlines the high-level technical design of Percival.

### User Interface

Percival is a client-side web application running fully in the user's browser.
The notebook interface is built with [Svelte](https://svelte.dev/) and styled
with [Tailwind CSS](https://tailwindcss.com/). It relies on numerous other open
source libraries, including [CodeMirror 6](https://codemirror.net/6/) for live
code editing and syntax highlighting,
[Remark](https://github.com/remarkjs/remark) and [KaTeX](https://katex.org/) for
Markdown rendering, and [Vite](https://vitejs.dev/) for frontend bundling.

The code for the web frontend is located in `src/`, which contains a mix of
Svelte (in `src/components/`) and TypeScript (in `src/lib/`). These modules are
bundled into a static website at build time, and there is no dynamic server-side
rendering.

### JIT Compiler

Users write code cells in a custom dialect of Datalog, and they are translated
to JavaScript by a Rust compiler, which itself is compiled to WebAssembly using
[wasm-bindgen](https://github.com/rustwasm/wasm-bindgen). The Percival
compiler's code is located in the `crates/` folder. For ergonomic parsing with
human-readable error messages, the compiler relies on
[chumsky](https://github.com/zesterer/chumsky), a parser combinator library.

After the `percival-wasm` crate is compiled to WebAssembly, it can be used by
client-side code. The compiler processes code cells, then sends the resulting
JavaScript to separate
[web workers](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API)
that sandbox the code and execute it just-in-time. As the user writes queries,
their notebook automatically tracks inter-cell dependencies and evaluates cells
in topological order, spawning / terminating worker threads on demand.

### Data Visualization

Plotting is done using a specialized web worker that runs JavaScript code with
access to the [Observable Plot](https://observablehq.com/@observablehq/plot)
library. In order for this library (and D3) to run in a worker context, we patch
the global document with a lightweight virtual DOM implementation ported from
[Domino](https://github.com/fgnass/domino).

### Deployment

In production, the `main` branch of this repository is continuously deployed to
[percival.ink](https://percival.ink/) via [Vercel](https://vercel.com/), which
hosts the static website. It also runs a serverless function (see
`api/index.go`) that allows users to share notebooks through the GitHub Gist
API.

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

## Acknowledgements

Created by Eric Zhang ([@ekzhang1](https://twitter.com/ekzhang1)). Licensed
under the [MIT license](LICENSE).
