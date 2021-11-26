import { nanoid } from "nanoid";
import { build } from "./runtime";
import type { CompilerResult } from "./runtime";

export type MarkdownCell = {
  type: "markdown";
  hidden: boolean;
  value: string;
};

export type CodeCellData = {
  type: "code";
  hidden: boolean;
  value: string;
};

export type CellData = MarkdownCell | CodeCellData;

export type CodeCellState = CodeCellData & {
  stale: boolean;
  result: CompilerResult;
  status: "pending" | "done";
  output?: Record<string, object>;
  graphErrors?: string;
  runtimeErrors?: string;
  evaluateHandle?: () => void;
};

export type CellState = MarkdownCell | CodeCellState;

function clear(state: CodeCellState) {
  state.evaluateHandle?.(); // cancel evaluation
  state.graphErrors = state.runtimeErrors = state.evaluateHandle = undefined;
}

export class NotebookState {
  /** Order of cells by ID. */
  private order: string[];

  /** Current state of each cell. */
  private cells: Map<string, CellState>;

  /** Callbacks on notebook state change. */
  private callbacks: Map<string, () => void>;

  constructor() {
    this.order = [];
    this.cells = new Map();
    this.callbacks = new Map();
  }

  get length() {
    return this.order.length;
  }

  addCell(cell: CellData) {
    this.insertCell(this.order.length, cell);
    this.rebuildGraph();
  }

  addCellBefore(id: string, cell: CellData) {
    const index = this.order.findIndex((v) => v === id);
    this.insertCell(index, cell);
    this.rebuildGraph();
  }

  private insertCell(index: number, cell: CellData) {
    if (index < 0 || index > this.order.length) {
      throw new Error(`Invalid cell index: ${index}`);
    }
    const id = nanoid();
    this.order.splice(index, 0, id);
    if (cell.type === "markdown") {
      this.cells.set(id, cell);
    } else {
      this.cells.set(id, {
        ...cell,
        stale: true,
        result: build(cell.value),
        status: "pending",
      });
    }
  }

  deleteCell(id: string) {
    const index = this.order.findIndex((v) => v === id);
    if (index === -1) {
      throw new Error(`Invalid cell ID: ${id}`);
    }
    this.order.splice(index, 1);
    this.cells.delete(id);
    this.rebuildGraph();
  }

  editCell(id: string, value: string) {
    const cell = this.getCell(id);
    cell.value = value;
    if (cell.type === "code") {
      clear(cell);
      cell.stale = true;
      cell.result = build(value);
      cell.status = "pending";
      this.rebuildGraph();
    } else {
      this.revalidate();
    }
  }

  toggleHidden(id: string) {
    const cell = this.getCell(id);
    cell.hidden = !cell.hidden;
    this.revalidate();
  }

  private getCell(id: string): CellState {
    const cell = this.cells.get(id);
    if (!cell) {
      throw new Error(`Invalid cell ID: ${id}`);
    }
    return cell;
  }

  private rebuildGraph() {
    // TODO: Update graph dependencies and pending/running cells.
    //   1. Find orphaned cells and duplicate outputs, set error messages.
    //   2. Set to "pending" - all cells that need to be reevaluated. Cancel
    //      execution of all previously pending cells.
    //   3. Construct a graph and evaluate in reverse topological order.
    //   4. Revalidate.

    // For each relation, a list of all cells that create that relation.
    const creators = new Map<string, string[]>();

    for (const [id, cell] of this.codeCells()) {
      if (cell.result.ok) {
        for (const relation of cell.result.results) {
          const array = creators.get(relation) ?? [];
          array.push(id);
          creators.set(relation, array);
        }
      }
    }

    // Check for duplicate outputs.
    for (const [relation, cellIds] of creators) {
      if (cellIds.length > 1) {
        // todo
      }
    }

    // Check for orphaned cells.
    for (const [id, cell] of this.codeCells()) {
      if (cell.result.ok) {
        for (const relation of cell.result.deps) {
          if (!creators.has(relation)) {
            // todo
          }
        }
      }
    }

    // Topological sorting algorithm, giving preference to non-stale cells.

    // Cancel past runs and start executing everything in order.

    this.revalidate();
  }

  *[Symbol.iterator](): IterableIterator<[string, Readonly<CellState>]> {
    for (const id of this.order) {
      yield [id, this.getCell(id)];
    }
  }

  *codeCells(): IterableIterator<[string, Readonly<CodeCellState>]> {
    for (const [id, cell] of this) {
      if (cell.type === "code") {
        yield [id, cell];
      }
    }
  }

  /**
   * Listen to changes in the notebook, returning a function that can be used to
   * dispose of the listener when completed.
   */
  listen(callback: () => void): () => void {
    const callbackId = nanoid();
    this.callbacks.set(callbackId, callback);
    return () => {
      this.callbacks.delete(callbackId);
    };
  }

  /** Send a message to all listeners that the state was changed.  */
  private revalidate() {
    this.callbacks.forEach((callback) => {
      callback();
    });
  }

  /** Save the notebook in a reproducible format appropriate for storage. */
  marshal(): CellData[] {
    const data = [];
    for (const [, cell] of this) {
      data.push({
        type: cell.type,
        hidden: cell.hidden,
        value: cell.value,
      });
    }
    return data;
  }

  /** Load a marshalled notebook. */
  static unmarshal(data: CellData[]): NotebookState {
    const notebook = new NotebookState();
    for (let i = 0; i < data.length; i++) {
      notebook.insertCell(i, data[i]);
    }
    notebook.rebuildGraph();
    return notebook;
  }
}
