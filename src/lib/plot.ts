import Worker from "./plot.worker?worker&inline";

interface CancellablePromise<T> extends Promise<T> {
  cancel: () => void;
}

type EvalPromise = CancellablePromise<string>;

type PlotResultOk = {
  ok: true;
  evaluate: (data: object[]) => EvalPromise;
  deps: string[];
  results: string[];
};

type PlotResultErr = {
  ok: false;
  error: string;
};

export type PlotResult = PlotResultOk | PlotResultErr;

export function buildPlot(src: string): PlotResult {
  // Empty program should be accepted and do nothing.
  if (src.trim() === "") {
    return {
      ok: true,
      evaluate: () => {
        const promise: Partial<EvalPromise> = Promise.resolve("");
        promise.cancel = () => {};
        return promise as EvalPromise;
      },
      deps: [],
      results: [],
    };
  }

  // TODO: plot with multiple inputs, single output
  // https://svelte.dev/repl/be2cbfee41bd416fb812b15c119d086c?version=3.48.0
  const nameFragment = `[a-zA-Z_$][a-zA-Z_$0-9]*`;
  const resultNameFragment = `(?<resultName>${nameFragment})\\s*=\\s*`;
  const nextDepNameFragment = `(?:\\s*,\\s*(${nameFragment}))?`;
  const either = (l: string, r: string) =>
    "(?:" + [l, r].map((s) => `(?:${s})`).join("|") + ")";
  const fullArgumentsList = [
    `\\(?`,
    `(${nameFragment})`,
    ...new Array(10).fill(undefined).map(() => nextDepNameFragment),
    `\\)?`,
  ].join("");
  const emptyArgumentsList = `\\(\\s*\\)`;
  const argumentsList = either(emptyArgumentsList, fullArgumentsList);
  const regexp = new RegExp(
    `^\\s*(?:${resultNameFragment})?(?:async\\s+)?${argumentsList}\\s*=>`,
  );

  const parsed = src.match(regexp);
  if (parsed === null) {
    return {
      ok: false,
      error: "Expected plot cell to start with `name =>` syntax",
    };
  }

  const resultName = parsed.groups?.resultName;
  const deps = Array.from(parsed.slice(2)).filter(
    (s) => s !== undefined && s !== null,
  );

  return {
    ok: true,
    evaluate: (data: object[]) => {
      const worker = new Worker();
      let rejectCb: (reason?: any) => void;
      const promise: Partial<EvalPromise> = new Promise((resolve, reject) => {
        rejectCb = reject;
        worker.addEventListener("message", (event) => {
          resolve(event.data);
          worker.terminate();
        });
        worker.addEventListener("error", (event) => {
          reject(new Error(event.message));
          worker.terminate();
        });
        worker.postMessage({ code: src, data });
      });
      promise.cancel = () => {
        worker.terminate();
        rejectCb(new Error("Promise was cancelled by user"));
      };
      return promise as EvalPromise;
    },
    deps,
    results: resultName ? [resultName] : [],
  };
}
