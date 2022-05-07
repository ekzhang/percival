import Immutable from "immutable";
import { autoType, csvParse, tsvParse } from "d3-dsv";
import { Relation } from "./types";
import type { RelationSet } from "./types";

/** Load data from an external source. */
async function load(url: string): Promise<Relation> {
  const resp = await fetch(url);
  if (!resp.ok) {
    throw new Error(`Failed to fetch ${url}:\n${await resp.text()}`);
  }
  const contentType = resp.headers.get("Content-Type");
  if (url.endsWith(".json") || contentType?.match(/application\/json/i)) {
    return resp.json();
  } else if (url.endsWith(".csv") || contentType?.match(/text\/csv/i)) {
    return Relation(csvParse(await resp.text(), autoType));
  } else if (
    url.endsWith(".tsv") ||
    contentType?.match(/text\/tab-separated-values/i)
  ) {
    return Relation(tsvParse(await resp.text(), autoType));
  } else {
    throw new Error(
      `Unknown file format for ${url}. Only JSON, CSV, and TSV are supported.
Try adding a file extension to the URL or providing a MIME Content-Type header.`,
    );
  }
}

/** Implementations of aggregates. Keep this in sync with `codegen.rs`. */
const aggregates: Record<string, (results: any[]) => any> = {
  count(results) {
    return results.length;
  },
  sum(results) {
    return results.reduce((x, y) => x + y, 0);
  },
  mean(results) {
    return results.reduce((x, y) => x + y, 0) / results.length;
  },
  min(results) {
    let min = null;
    for (const x of results) {
      if (min === null || x < min) {
        min = x;
      }
    }
    return min;
  },
  max(results) {
    let max = null;
    for (const x of results) {
      if (max === null || x > max) {
        max = x;
      }
    }
    return max;
  },
};

const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;

let evaluate: undefined | ((deps: RelationSet) => Promise<RelationSet>);

function initialize(js: string) {
  if (evaluate) {
    throw new Error("internal: worker was already initialized");
  }
  const fn = new AsyncFunction("__percival_deps", "__percival", js);
  evaluate = (deps: RelationSet) => fn(deps, { Immutable, load, aggregates });
}

onmessage = (event) => {
  if (event.data.type === "source") {
    initialize(event.data.code);
  } else if (event.data.type === "eval") {
    if (!evaluate) {
      throw new Error("internal: worker was not initialized");
    }
    evaluate(event.data.deps)
      .then((results) => {
        postMessage(results);
      })
      .catch((error: unknown) => {
        // Bubble up asynchronous errors to the global worker context.
        setTimeout(() => {
          throw error;
        });
      });
  } else {
    throw new Error(`internal: unknown event type: ${event.data.type}`);
  }
};
