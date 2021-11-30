import Immutable from "immutable";

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

let evaluate:
  | undefined
  | ((deps: Record<string, object[]>) => Promise<Record<string, object[]>>);

function initialize(js: string) {
  if (evaluate) {
    throw new Error("internal: worker was already initialized");
  }
  const fn = new AsyncFunction(
    "__percival_deps",
    "__percival_immutable",
    "__percival_aggregates",
    js,
  );
  evaluate = (deps: Record<string, object[]>) =>
    fn(deps, Immutable, aggregates);
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
