import { expect } from "chai";
import init from "percival-wasm";
import { build } from "./runtime";

function checkProgram({
  src,
  input,
  output,
}: {
  src: string;
  input: Record<string, object[]>;
  output: Record<string, object[]>;
}) {
  const result = build(src);
  expect(result.evaluate).not.to.be.undefined;
  const observed = result.evaluate(input);
  expect(observed).to.have.keys(...Object.keys(output));
  for (const key of Object.keys(output)) {
    expect(observed[key]).to.have.deep.members(output[key]);
  }
}

describe("compilation tests", () => {
  it("can build code", async () => {
    await init();
    expect(build("tc(x: 3).").errors).to.be.undefined;
    expect(build("tc(x:).").errors).not.to.be.undefined;
  });

  it("evaluates a simple program", async () => {
    await init();
    const result = build("tc(x: 3).");
    expect(result.evaluate).not.to.be.undefined;
    expect(result.evaluate({})).to.be.deep.equal({ tc: [{ x: 3 }] });
  });

  it("evaluates transitive closure from input", async () => {
    await init();
    checkProgram({
      src: `
tc(x, y) :- edge(x, y).
tc(x, y) :- tc(x, y: z), edge(x: z, y).
`,
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

  it("evaluates transitive closure inline", async () => {
    await init();
    checkProgram({
      src: `
edge(x: "foo", y: "bar").
edge(x: "bar", y: "baz").
tc(x, y) :- edge(x, y).
tc(x, y) :- tc(x, y: z), edge(x: z, y).
`,
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
