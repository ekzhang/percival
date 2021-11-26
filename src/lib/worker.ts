import Immutable from "immutable";

let evaluate:
  | undefined
  | ((deps: Record<string, object[]>) => Record<string, object[]>);

function initialize(js: string) {
  if (evaluate) {
    throw new Error("internal: worker was already initialized");
  }
  const eval_fn = new Function("__percival_deps", "__percival_immutable", js);
  evaluate = (deps: Record<string, object[]>) => eval_fn(deps, Immutable);
}

onmessage = (event) => {
  if (event.data.type === "source") {
    initialize(event.data.code);
  } else if (event.data.type === "eval") {
    if (!evaluate) {
      throw new Error("internal: worker was not initialized");
    }
    const results = evaluate(event.data.deps);
    postMessage(results);
  } else {
    throw new Error(`internal: unknown event type: ${event.data.type}`);
  }
};
