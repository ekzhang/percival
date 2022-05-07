import { expect } from "chai";
import { buildPlot } from "./plot";

function check(
  str: string,
  deps: string[] = [],
  result: string | undefined = undefined,
) {
  const context = (name: string) => `build('${str}').${name}`;
  const plot = buildPlot(str);
  expect(plot.ok, context("ok")).to.be.true;
  if (plot.ok) {
    expect(plot.deps, context("deps")).to.deep.eq(deps);
    if (result) {
      expect(plot.results, context("results")).to.deep.eq([result]);
    }
  }
}

describe("buildPlot", () => {
  it("parses empty string", () => check(""));

  it("parses a basic function", () => check("x => cool", ["x"]));

  it("parses a two-arg function", () => check("(x, y) => x + y", ["x", "y"]));

  it("parses a one-arg result", () =>
    check("stuff = x => nice", ["x"], "stuff"));

  it("parses a no-arg result", () =>
    check("result = () => swanky", [], "result"));

  it("parses an async function", () =>
    check('_ = async () => import("lodash")', [], "_"));
});
