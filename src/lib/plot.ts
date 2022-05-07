import Worker from "./plot.worker?worker&inline";
import type { Relation } from "./types";

interface CancellablePromise<T> extends Promise<T> {
  cancel: () => void;
}

type EvalPromise = CancellablePromise<unknown>;

type PlotResultOk = {
  ok: true;
  evaluate: (dependencies: Relation[]) => EvalPromise;
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

  // This mess of regexps parses the first line of an arrow function declaration.
  // See plot.test.ts for examples of valid declarations.
  const nameFragment = `[a-zA-Z_$][a-zA-Z_$0-9]*`;
  const resultNameFragment = `(?<resultName>${nameFragment})\\s*=\\s*`; // `resultName = `
  const nextFunctionArg = `(?:\\s*,\\s*(${nameFragment}))?`; // `, nextDepName`
  const either = (l: string, r: string) =>
    "(?:" + [l, r].map((s) => `(?:${s})`).join("|") + ")";
  const fullArgumentsList = [
    // matches eg `(depName1, depName2)`
    `\\(?`,
    `(${nameFragment})`, // `depName1`
    // If a capture group is repeated, RegExp only retains the last such capture.
    // To support N arguments, we need N capture groups, or we can do some kind of loop w/ regex.exec(...).
    ...new Array(10).fill(undefined).map(() => nextFunctionArg),
    `\\)?`,
  ].join("");
  const emptyArgumentsList = `\\(\\s*\\)`; // `()`
  const argumentsList = either(emptyArgumentsList, fullArgumentsList); // `()` | `dep` | (dep) | `(dep1, dep2)`
  const regexp = new RegExp( // `resultName = async (dep1, dep2) =>`
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
    evaluate: (data: Relation[]) => {
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
