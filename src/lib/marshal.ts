import type { CellData } from "./notebook";

/** Marshal an array of cell data into plaintext (.percival) format. */
export function marshal(data: Readonly<CellData>[]): string {
  const output = ["This is a Percival notebook (https://percival.ink/).\n"];
  for (const cell of data) {
    const prefix = cell.hidden ? "╔═╣" : "╔═╡";
    const mode = cell.type === "code" ? "Code" : "Markdown";
    output.push(`${prefix} ${mode}\n${cell.value}\n`);
  }
  return output.join("\n");
}

/** Unmarshal a plaintext (.percival) file into an array of cell data. */
export function unmarshal(text: string): CellData[] {
  const data: CellData[] = [];
  const parts = text.split(/\r?\n(╔═╡|╔═╣) (Code|Markdown)\r?\n/);
  for (let i = 1; i < parts.length; i += 3) {
    const prefix = parts[i];
    const mode = parts[i + 1];
    let value = parts[i + 2];
    if (value.endsWith("\n")) {
      const offset = value.endsWith("\r\n") ? 2 : 1;
      value = value.substring(0, value.length - offset);
    }
    data.push({
      type: mode === "Code" ? "code" : "markdown",
      hidden: prefix === "╔═╣",
      value,
    });
  }
  return data;
}
