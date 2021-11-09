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

## Development

To get started, first run `npm install` to install dependencies, then run the
following command to start the development server:

```shell
npm run dev
```

This should open a Percival notebook in your browser.
