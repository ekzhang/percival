import { expect } from "chai";
import init from "percival-wasm";
import { build } from "./runtime";
import { Relation, type RelationSet } from "./types";

async function checkProgram({
  src,
  deps,
  results,
  input,
  output,
}: {
  src: string;
  deps: string[];
  results: string[];
  input: RelationSet;
  output: RelationSet;
}) {
  const result = build(src);
  expect(result.ok).to.be.true;
  if (!result.ok) {
    // unreachable, needed for type inference
    throw null;
  }
  expect(result.deps).to.have.members(deps);
  expect(result.results).to.have.members(results);
  const observed = await result.evaluate(input);
  for (const key of Object.keys(output)) {
    expect(observed[key]).to.have.deep.members(output[key]);
  }
}

describe("basic compilation", () => {
  it("can build code", async () => {
    await init();
    expect(build("tc(x: 3).").ok).to.be.true;
    expect(build("tc(x:).").ok).to.be.false;
  });

  it("evaluates a simple program", async () => {
    await init();
    const result = build("tc(x: 3).");
    expect(result.ok).to.be.true;
    if (!result.ok) throw null; // unreachable
    expect(await result.evaluate({})).to.deep.equal({
      tc: [{ x: 3 }],
    });
  });

  it("evaluates transitive closure from input", async () => {
    await init();
    await checkProgram({
      src: `
tc(x, y) :- edge(x, y).
tc(x, y) :- tc(x, y: z), edge(x: z, y).
`,
      deps: ["edge"],
      results: ["tc"],
      input: {
        edge: Relation([
          { x: 2, y: 3 },
          { x: 3, y: 4 },
        ]),
      },
      output: {
        tc: Relation([
          { x: 2, y: 3 },
          { x: 2, y: 4 },
          { x: 3, y: 4 },
        ]),
      },
    });
  });

  it("evaluates a bigger transitive closure", async () => {
    await init();
    await checkProgram({
      src: `
tc(x, y) :- tc(x, y: z), edge(x: z, y).
tc(x, y) :- edge(x, y).
`,
      deps: ["edge"],
      results: ["tc"],
      input: {
        edge: Relation([
          { x: "hello", y: "world" },
          { x: "world", y: "foo" },
          { x: "foo", y: "baz" },
          { x: "world", y: "bar" },
          { x: "alt-src", y: "foo" },
        ]),
      },
      output: {
        tc: Relation([
          { x: "hello", y: "world" },
          { x: "hello", y: "foo" },
          { x: "hello", y: "baz" },
          { x: "hello", y: "bar" },
          { x: "world", y: "foo" },
          { x: "world", y: "baz" },
          { x: "world", y: "bar" },
          { x: "alt-src", y: "foo" },
          { x: "alt-src", y: "baz" },
          { x: "foo", y: "baz" },
        ]),
      },
    });
  });

  it("evaluates transitive closure inline", async () => {
    await init();
    await checkProgram({
      src: `
edge(x: "foo", y: "bar").
edge(x: "bar", y: "baz").
tc(x, y) :- edge(x, y).
tc(x, y) :- tc(x, y: z), edge(x: z, y).
`,
      deps: [],
      results: ["edge", "tc"],
      input: {},
      output: {
        edge: Relation([
          { x: "foo", y: "bar" },
          { x: "bar", y: "baz" },
        ]),
        tc: Relation([
          { x: "foo", y: "bar" },
          { x: "foo", y: "baz" },
          { x: "bar", y: "baz" },
        ]),
      },
    });
  });

  it("can handle boolean literals", async () => {
    await init();
    await checkProgram({
      src: `ok(x: true, y: false).`,
      deps: [],
      results: ["ok"],
      input: {},
      output: {
        ok: Relation([{ x: true, y: false }]),
      },
    });
  });
});

describe("embedded backtick expressions", () => {
  it("evaluates backtick expressions", async () => {
    await init();
    await checkProgram({
      src: `
name(value: \`first + " " + last\`) :- person(first, last).
`,
      deps: ["person"],
      results: ["name"],
      input: {
        person: Relation([
          {
            first: "eric",
            last: "zhang",
          },
          {
            first: "john",
            last: "doe",
          },
        ]),
      },
      output: {
        name: Relation([{ value: "eric zhang" }, { value: "john doe" }]),
      },
    });
  });

  it("evaluates fibonacci numbers", async () => {
    await init();
    await checkProgram({
      src: `
fib(n: 0, x: 0).
fib(n: 1, x: 1).
fib(n: \`n + 1\`, x: \`x1 + x2\`) :-
  fib(n, x: x1),
  fib(n: \`n - 1\`, x: x2),
  x = \`x1 + x2\`,
  \`n < 10\`.
`,
      deps: [],
      results: ["fib"],
      input: {},
      output: {
        fib: Relation([
          { n: 0, x: 0 },
          { n: 1, x: 1 },
          { n: 2, x: 1 },
          { n: 3, x: 2 },
          { n: 4, x: 3 },
          { n: 5, x: 5 },
          { n: 6, x: 8 },
          { n: 7, x: 13 },
          { n: 8, x: 21 },
          { n: 9, x: 34 },
          { n: 10, x: 55 },
        ]),
      },
    });
  });
});

describe("promise cancellation", () => {
  it("can cancel evaluation", async () => {
    await init();
    const result = build("ok().");
    expect(result.ok).to.be.true;
    if (!result.ok) throw null; // unreachable
    const promise = result.evaluate({});
    promise.cancel();
    try {
      await promise;
      throw new Error("Promise should have thrown");
    } catch (error: any) {
      expect(error.message).to.equal("Promise was cancelled by user");
    }
  });
});

describe("import directives", () => {
  it("can load crimea.json", async () => {
    await init();
    await checkProgram({
      src: `import crimea from "npm://vega-datasets@2.1.0/data/crimea.json"`,
      deps: [],
      results: ["crimea"],
      input: {},
      output: {
        crimea: Relation([
          { date: "1854-04-01", wounds: 0, other: 110, disease: 110 },
          { date: "1854-05-01", wounds: 0, other: 95, disease: 105 },
          { date: "1854-06-01", wounds: 0, other: 40, disease: 95 },
          { date: "1854-07-01", wounds: 0, other: 140, disease: 520 },
          { date: "1854-08-01", wounds: 20, other: 150, disease: 800 },
          { date: "1854-09-01", wounds: 220, other: 230, disease: 740 },
          { date: "1854-10-01", wounds: 305, other: 310, disease: 600 },
          { date: "1854-11-01", wounds: 480, other: 290, disease: 820 },
          { date: "1854-12-01", wounds: 295, other: 310, disease: 1100 },
          { date: "1855-01-01", wounds: 230, other: 460, disease: 1440 },
          { date: "1855-02-01", wounds: 180, other: 520, disease: 1270 },
          { date: "1855-03-01", wounds: 155, other: 350, disease: 935 },
          { date: "1855-04-01", wounds: 195, other: 195, disease: 560 },
          { date: "1855-05-01", wounds: 180, other: 155, disease: 550 },
          { date: "1855-06-01", wounds: 330, other: 130, disease: 650 },
          { date: "1855-07-01", wounds: 260, other: 130, disease: 430 },
          { date: "1855-08-01", wounds: 290, other: 110, disease: 490 },
          { date: "1855-09-01", wounds: 355, other: 100, disease: 290 },
          { date: "1855-10-01", wounds: 135, other: 95, disease: 245 },
          { date: "1855-11-01", wounds: 100, other: 140, disease: 325 },
          { date: "1855-12-01", wounds: 40, other: 120, disease: 215 },
          { date: "1856-01-01", wounds: 0, other: 160, disease: 160 },
          { date: "1856-02-01", wounds: 0, other: 100, disease: 100 },
          { date: "1856-03-01", wounds: 0, other: 125, disease: 90 },
        ]),
      },
    });
  });

  it("can load iowa-electricity.csv", async () => {
    await init();
    await checkProgram({
      src: `
import iowa from "npm://vega-datasets@2.1.0/data/iowa-electricity.csv"
count(value: count[1] { iowa() }).
`,
      deps: [],
      results: ["iowa", "count"],
      input: {},
      output: {
        count: Relation([{ value: 51 }]),
      },
    });
  });
});

describe("aggregation operators", () => {
  it("calculates statistics in crimea data", async () => {
    await init();
    await checkProgram({
      src: `
import crimea from "npm://vega-datasets@2.1.0/data/crimea.json"

stats(count, max_wounds, min_wounds, total_wounds, mean_wounds) :-
  count = count[1] { crimea() },
  max_wounds = max[wounds] { crimea(wounds) },
  min_wounds = min[wounds] { crimea(wounds) },
  total_wounds = sum[wounds] { crimea(wounds) },
  mean_wounds = mean[wounds] { crimea(wounds) }.
`,
      deps: [],
      results: ["crimea", "stats"],
      input: {},
      output: {
        stats: Relation([
          {
            count: 24,
            max_wounds: 480,
            min_wounds: 0,
            total_wounds: 3770,
            mean_wounds: 3770 / 24,
          },
        ]),
      },
    });
  });

  it("calculates yearly mpg in car data", async () => {
    await init();
    await checkProgram({
      src: `
import cars from "npm://vega-datasets/data/cars.json"

year(year: Year) :- cars(Year).

yearly_mpg(year, value) :-
  year(year),
  value = mean[Miles_per_Gallon] {
    cars(Year: year, Miles_per_Gallon)
  }.
`,
      deps: [],
      results: ["cars", "year", "yearly_mpg"],
      input: {},
      output: {
        yearly_mpg: Relation([
          {
            value: 33.696551724137926,
            year: "1980-01-01",
          },
          {
            value: 22.703703703703702,
            year: "1974-01-01",
          },
          {
            value: 21.573529411764707,
            year: "1976-01-01",
          },
          {
            value: 30.536065573770493,
            year: "1982-01-01",
          },
          {
            value: 25.09310344827585,
            year: "1979-01-01",
          },
          {
            value: 20.517241379310345,
            year: "1971-01-01",
          },
          {
            value: 23.375,
            year: "1977-01-01",
          },
          {
            value: 18.714285714285715,
            year: "1972-01-01",
          },
          {
            value: 24.061111111111114,
            year: "1978-01-01",
          },
          {
            value: 20.266666666666666,
            year: "1975-01-01",
          },
          {
            value: 17.1,
            year: "1973-01-01",
          },
          {
            value: 14.657142857142857,
            year: "1970-01-01",
          },
        ]),
      },
    });
  });

  it("handles nested aggregates", async () => {
    await init();
    await checkProgram({
      src: `ok(value: sum[min[to] { edge(from, to) }] { vertex(id: from) }).`,
      deps: ["vertex", "edge"],
      results: ["ok"],
      input: {
        vertex: Relation([{ id: 1 }, { id: 2 }, { id: 3 }, { id: 4 }]),
        edge: Relation([
          { from: 1, to: 3 },
          { from: 1, to: 2 },
          { from: 2, to: 4 },
          { from: 3, to: 3 },
          { from: 4, to: 1 },
        ]),
      },
      output: {
        ok: Relation([{ value: 10 }]),
      },
    });
  });
});
