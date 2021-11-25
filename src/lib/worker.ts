import Immutable from "immutable";

let evaluate: (deps: Record<string, object[]>) => Record<string, object[]>;

function initialize(js: string) {
  if (evaluate) {
    throw new Error("worker was already initialized");
  }
  const eval_fn = new Function("__percival_deps", "__percival_immutable", js);
  evaluate = (deps: Record<string, object[]>) => eval_fn(deps, Immutable);
}

onmessage = (event) => {
  if (event.data.type === "source") {
    initialize(event.data.code);
  } else if (event.data.type === "eval") {
    const results = evaluate(event.data.deps);
    postMessage(results);
  } else {
    throw new Error(`unknown event type: ${event.data.type}`);
  }
};
