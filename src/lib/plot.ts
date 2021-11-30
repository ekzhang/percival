import Worker from "./plot.worker?worker&inline";

interface CancellablePromise<T> extends Promise<T> {
  cancel: () => void;
}

type EvalPromise = CancellablePromise<string>;

type PlotResultOk = {
  ok: true;
  evaluate: (data: object[]) => EvalPromise;
  deps: string[];
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
    };
  }

  const result = src.match(/^\s*([a-zA-Z_$][a-zA-Z_$0-9]*)\s*=>/);
  if (result === null) {
    return {
      ok: false,
      error: "Expected plot cell to start with `name =>` syntax",
    };
  }

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
    deps: [result[1]],
  };
}
