import { readable } from "svelte/store";
import type { NotebookState } from "./notebook";

export function createNotebookStore(notebook: NotebookState) {
  return readable(notebook, (set) => {
    const dispose = notebook.listen(() => set(notebook));
    return dispose;
  });
}
