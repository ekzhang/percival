import Immutable from "immutable";

const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;

let evaluate:
  | undefined
  | ((deps: Record<string, object[]>) => Promise<Record<string, object[]>>);

function initialize(js: string) {
  if (evaluate) {
    throw new Error("internal: worker was already initialized");
  }
  const fn = new AsyncFunction("__percival_deps", "__percival_immutable", js);
  evaluate = (deps: Record<string, object[]>) => fn(deps, Immutable);
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
        throw error;
      });
  } else {
    throw new Error(`internal: unknown event type: ${event.data.type}`);
  }
};
