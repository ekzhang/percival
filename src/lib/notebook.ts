import { nanoid } from "nanoid";
import { build } from "./runtime";
import type { CompilerResult } from "./runtime";
import { buildPlot } from "./plot";
import type { PlotResult } from "./plot";
import type { Relation, RelationSet } from "./types";

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

export type PlotCellData = {
  type: "plot";
  hidden: boolean;
  value: string;
};

export type CellData = MarkdownCell | CodeCellData | PlotCellData;

export type CodeCellState = CodeCellData & {
  result: CompilerResult;
  status: "stale" | "pending" | "done";
  output?: RelationSet;
  graphErrors?: string;
  runtimeErrors?: string;
  evaluateHandle?: () => void;
};

export type PlotCellState = PlotCellData & {
  result: PlotResult;
  status: "stale" | "pending" | "done";
  output?: unknown;
  graphErrors?: string;
  runtimeErrors?: string;
  evaluateHandle?: () => void;
};

export type CellState = MarkdownCell | CodeCellState | PlotCellState;

function clear(
  cell: CodeCellState | PlotCellState,
  status: CodeCellState["status"],
) {
  cell.evaluateHandle?.(); // cancel evaluation
  cell.graphErrors = cell.runtimeErrors = cell.evaluateHandle = undefined;
  cell.status = status;
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
    } else if (cell.type === "code") {
      this.cells.set(id, {
        ...cell,
        result: build(cell.value),
        status: "stale",
      });
    } else {
      this.cells.set(id, {
        ...cell,
        result: buildPlot(cell.value),
        status: "stale",
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
      clear(cell, "stale");
      cell.result = build(value);
      this.rebuildGraph();
    } else if (cell.type === "plot") {
      clear(cell, "stale");
      cell.result = buildPlot(value);
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

  /**
   * Update graph dependencies and evaluate pending/running cells.
   *
   * This is a fairly complex function. Roughly speaking, it is responsible for
   * the following execution strategy:
   *
   * 1. Find orphaned cells and duplicate outputs, set error messages.
   * 2. Set to "pending" - all stale cells that need to be re-evaluated. Cancel
   *    execution of all previously running cells.
   * 3. Revalidate to track changes.
   * 4. Start evaluating those stale cells asynchronously in separate worker
   *    processes. On error, set the "runtimeErrors" property, and otherwise set
   *    the output on success while marking dependents as stale.
   */
  private rebuildGraph() {
    // For each relation, a list of all cells that create that relation.
    const creators = new Map<string, string[]>();

    for (const [id, cell] of this.executableCells()) {
      if (cell.graphErrors !== undefined) {
        delete cell.graphErrors;
      }
      if (cell.result.ok && cell.result.results.length > 0) {
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
        for (const id of cellIds) {
          const cell = this.getCell(id);
          if (cell.type === "markdown") throw new Error("unreachable");
          clear(cell, "stale");
          cell.graphErrors = `Relation "${relation}" is defined in multiple cells.`;
        }
      }
    }

    // Check for orphaned cells.
    for (const [, cell] of this.executableCells()) {
      if (cell.result.ok) {
        for (const relation of cell.result.deps) {
          if (!creators.has(relation)) {
            clear(cell, "stale");
            cell.graphErrors = `Dependency "${relation}" was not found in any cell.`;
            break;
          }
        }
      }
    }

    // Asynchronously evaluate all stale cells that have dependencies met.
    for (const [, cell] of this.executableCells()) {
      if (
        cell.result.ok &&
        cell.graphErrors === undefined &&
        cell.status === "stale"
      ) {
        let depsOk = true;
        const deps: RelationSet = {};
        for (const relation of cell.result.deps) {
          const cellIds = creators.get(relation);
          if (!cellIds || cellIds.length != 1) {
            depsOk = false;
            break;
          }
          const prev = this.getCell(cellIds[0]);
          if (prev.type === "markdown") throw new Error("unreachable");
          if (
            prev.status === "done" &&
            prev.result.ok &&
            prev.graphErrors === undefined &&
            prev.runtimeErrors === undefined
          ) {
            if (prev.type === "code" && prev.output?.[relation]) {
              deps[relation] = prev.output[relation];
            } else if (prev.type === "plot" && prev.output !== undefined) {
              deps[relation] = prev.output as Relation;
            } else {
              depsOk = false;
              break;
            }
          } else {
            depsOk = false;
            break;
          }
        }

        if (depsOk) {
          clear(cell, "pending");
          if (cell.type === "code") {
            const promise = cell.result.evaluate(deps);
            cell.evaluateHandle = () => promise.cancel();
            const results = cell.result.results; // storing for async callback
            promise
              .then((data) => {
                cell.output = data;
                cell.status = "done";
                this.markUpdate(results);
              })
              .catch((err: Error) => {
                if (err.message !== "Promise was cancelled by user") {
                  cell.status = "done";
                  cell.runtimeErrors = err.message;
                  this.revalidate();
                }
              });
          } else {
            const depValues = cell.result.deps.map((dep) => deps[dep]);
            const promise = cell.result.evaluate(depValues);
            cell.evaluateHandle = () => promise.cancel();
            const results = cell.result.results; // storing for async callback
            promise
              .then((figure) => {
                cell.output = figure;
                cell.status = "done";
                this.markUpdate(results);
              })
              .catch((err: Error) => {
                if (err.message !== "Promise was cancelled by user") {
                  cell.status = "done";
                  cell.runtimeErrors = err.message;
                  this.revalidate();
                }
              });
          }
        }
      }
    }

    this.revalidate();
  }

  private markUpdate(relations: string[]) {
    const changed = new Set(relations);
    for (const [, cell] of this.executableCells()) {
      if (
        cell.result.ok &&
        cell.result.deps.filter((relation) => changed.has(relation)).length > 0
      ) {
        clear(cell, "stale");
      }
    }
    this.rebuildGraph();
  }

  [Symbol.iterator](): IterableIterator<[string, Readonly<CellState>]> {
    return this.iter();
  }

  private *iter(): IterableIterator<[string, CellState]> {
    for (const id of this.order) {
      yield [id, this.getCell(id)];
    }
  }

  private *executableCells(): IterableIterator<
    [string, CodeCellState | PlotCellState]
  > {
    for (const [id, cell] of this.iter()) {
      if (cell.type === "code" || cell.type === "plot") {
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

  /** Save the notebook data in a reproducible format for storage. */
  save(): Readonly<CellData>[] {
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

  /** Load a notebook from cell data. */
  static load(data: Readonly<CellData>[]): NotebookState {
    const notebook = new NotebookState();
    for (let i = 0; i < data.length; i++) {
      notebook.insertCell(i, data[i]);
    }
    notebook.rebuildGraph();
    return notebook;
  }
}
