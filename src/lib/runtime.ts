import { compile } from "percival-wasm";
import Worker from "./runtime.worker?worker&inline";

interface CancellablePromise<T> extends Promise<T> {
  cancel: () => void;
}

type EvalPromise = CancellablePromise<Record<string, object[]>>;

type CompilerResultOk = {
  ok: true;
  evaluate: (deps: Record<string, object[]>) => EvalPromise;
  deps: string[];
  results: string[];
};

type CompilerResultErr = {
  ok: false;
  errors: string;
};

export type CompilerResult = CompilerResultOk | CompilerResultErr;

export function build(src: string): CompilerResult {
  let result = compile(src);
  if (result.is_ok()) {
    const code = result.js();
    return {
      ok: true,
      evaluate: (deps) => {
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
          worker.postMessage({ type: "source", code });
          worker.postMessage({ type: "eval", deps });
        });
        promise.cancel = () => {
          worker.terminate();
          rejectCb(new Error("Promise was cancelled by user"));
        };
        return promise as EvalPromise;
      },
      deps: result.deps()!,
      results: [...result.results()!],
    };
  } else {
    return { ok: false, errors: result.err()! };
  }
}
