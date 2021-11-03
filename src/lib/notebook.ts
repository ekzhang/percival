export type CellData = {
  type: "markdown" | "code";
  value: string;
  hidden: boolean;
};

export class NotebookState {
  cells: CellData[];

  constructor() {
    this.cells = [];
  }

  addCell(cell: CellData) {
    this.cells.push(cell);
  }

  insertCell(index: number, cell: CellData) {
    this.cells.splice(index, 0, cell);
  }

  deleteCell(index: number) {
    this.cells.splice(index, 1);
  }
}
