export type CellData = {
  type: "markdown" | "code";
  value: string;
};

export class NotebookState {
  cells: CellData[];

  constructor() {
    this.cells = [];
  }

  addCell(cell: CellData) {
    this.cells.push(cell);
  }
}
