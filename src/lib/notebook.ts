import { nanoid } from "nanoid";

type MarkdownCell = {
  type: "markdown";
  hidden: boolean;
  value: string;
};

type CodeCell = {
  type: "code";
  hidden: boolean;
  value: string;
};

export type CellData = MarkdownCell | CodeCell;

export class NotebookState {
  /** Order of cells by ID. */
  private order: string[];

  /** Current state of each cell. */
  private cells: Map<string, CellData>;

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
  }

  addCellBefore(id: string, cell: CellData) {
    const index = this.order.findIndex((v) => v === id);
    this.insertCell(index, cell);
  }

  private insertCell(index: number, cell: CellData) {
    if (index < 0 || index > this.order.length) {
      throw new Error(`Invalid cell index: ${index}`);
    }
    const id = nanoid();
    this.order.splice(index, 0, id);
    this.cells.set(id, cell);
    this.revalidate();
  }

  deleteCell(id: string) {
    const index = this.order.findIndex((v) => v === id);
    if (index === -1) {
      throw new Error(`Invalid delete cell ID: ${id}`);
    }
    this.order.splice(index, 1);
    this.cells.delete(id);
    this.revalidate();
  }

  editCell(id: string, value: string) {
    this.cells.get(id).value = value;
    this.revalidate();
  }

  toggleHidden(id: string) {
    const cell = this.cells.get(id);
    cell.hidden = !cell.hidden;
    this.revalidate();
  }

  *[Symbol.iterator](): IterableIterator<[string, Readonly<CellData>]> {
    for (const id of this.order) {
      yield [id, this.cells.get(id)];
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
}
