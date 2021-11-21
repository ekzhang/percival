import Immutable from "immutable";
import { compile } from "percival-wasm";

export type CompilerResult = {
  evaluate?: (deps: Record<string, object[]>) => Record<string, object[]>;
  errors?: string;
};

export function build(src: string): CompilerResult {
  let result = compile(src);
  if (result.is_ok()) {
    const eval_fn = new Function(
      "__percival_deps",
      "__percival_immutable",
      result.ok()
    );
    return {
      evaluate: (deps: Record<string, object[]>) => eval_fn(deps, Immutable),
    };
  } else {
    return { errors: result.err() };
  }
}
