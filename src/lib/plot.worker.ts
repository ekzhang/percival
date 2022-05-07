import * as Plot from "@observablehq/plot";
import domino from "domino";
import { RenderedElement } from "./types";

globalThis.document = domino.createDocument();

onmessage = async (event) => {
  const { code, data } = event.data;
  const fn = new Function(
    "Plot",
    "...__percival_data",
    `return (${code})(...__percival_data);`,
  );
  let result = await fn(Plot, ...data);

  if (result && "outerHTML" in result) {
    result = RenderedElement(result);
  }

  postMessage(result);
};
