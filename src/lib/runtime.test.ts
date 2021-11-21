import { expect } from "chai";
import init from "percival-wasm";
import { build } from "./runtime";

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

  it("evaluates transitive closure", async () => {
    await init();
    const src = `
tc(x, y) :- edge(x, y).
tc(x, y) :- tc(x, y: z), edge(x: z, y).
`;
    const result = build(src);
    expect(result.evaluate).not.to.be.undefined;
    const output = result.evaluate({
      edge: [
        { x: 2, y: 3 },
        { x: 3, y: 4 },
      ],
    });
    expect(output).to.have.keys("tc");
    expect(output.tc).to.have.deep.members([
      { x: 2, y: 3 },
      { x: 2, y: 4 },
      { x: 3, y: 4 },
    ]);
  });
});
