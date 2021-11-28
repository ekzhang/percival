import { expect } from "chai";
import init from "percival-wasm";
import { build } from "./runtime";

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
  input: Record<string, object[]>;
  output: Record<string, object[]>;
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
  expect(observed).to.have.keys(...Object.keys(output));
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
        edge: [
          { x: 2, y: 3 },
          { x: 3, y: 4 },
        ],
      },
      output: {
        tc: [
          { x: 2, y: 3 },
          { x: 2, y: 4 },
          { x: 3, y: 4 },
        ],
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
        edge: [
          { x: "hello", y: "world" },
          { x: "world", y: "foo" },
          { x: "foo", y: "baz" },
          { x: "world", y: "bar" },
          { x: "alt-src", y: "foo" },
        ],
      },
      output: {
        tc: [
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
        ],
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
        edge: [
          { x: "foo", y: "bar" },
          { x: "bar", y: "baz" },
        ],
        tc: [
          { x: "foo", y: "bar" },
          { x: "foo", y: "baz" },
          { x: "bar", y: "baz" },
        ],
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
        person: [
          {
            first: "eric",
            last: "zhang",
          },
          {
            first: "john",
            last: "doe",
          },
        ],
      },
      output: {
        name: [{ value: "eric zhang" }, { value: "john doe" }],
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
    \`n < 10\`.
`,
      deps: [],
      results: ["fib"],
      input: {},
      output: {
        fib: [
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
        ],
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
  it("can load vega-datasets/crimea", async () => {
    await init();
    await checkProgram({
      src: `@import crimea from "npm://vega-datasets@2.1.0/data/crimea.json"`,
      deps: [],
      results: ["crimea"],
      input: {},
      output: {
        crimea: [
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
        ],
      },
    });
  });
});
