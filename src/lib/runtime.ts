import Immutable from "immutable";
import { compile } from "percival-wasm";

type CompilerResultOk = {
  ok: true;
  evaluate?: (deps: Record<string, object[]>) => Record<string, object[]>;
  deps?: string[];
  results?: string[];
};

type CompilerResultErr = {
  ok: false;
  errors?: string;
};

export type CompilerResult = CompilerResultOk | CompilerResultErr;

export function build(src: string): CompilerResult {
  let result = compile(src);
  if (result.is_ok()) {
    const eval_fn = new Function(
      "__percival_deps",
      "__percival_immutable",
      result.js(),
    );
    return {
      ok: true,
      evaluate: (deps: Record<string, object[]>) => eval_fn(deps, Immutable),
      deps: result.deps(),
      results: result.results(),
    };
  } else {
    return { ok: false, errors: result.err() };
  }
}
