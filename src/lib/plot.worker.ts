import * as Plot from "@observablehq/plot";
import domino from "domino";

globalThis.document = domino.createDocument();

onmessage = (event) => {
  const { code, data } = event.data;
  const fn = new Function(
    "Plot",
    "__percival_data",
    `return (${code})(__percival_data);`,
  );
  postMessage(fn(Plot, data).outerHTML);
};
