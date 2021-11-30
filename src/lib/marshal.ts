import type { CellData } from "./notebook";

const modeNames = { code: "Code", markdown: "Markdown", plot: "Plot" };

/** Marshal an array of cell data into plaintext (.percival) format. */
export function marshal(data: Readonly<CellData>[]): string {
  const output = ["This is a Percival notebook (https://percival.ink/).\n"];
  for (const cell of data) {
    const prefix = cell.hidden ? "╔═╣" : "╔═╡";
    const mode = modeNames[cell.type];
    output.push(`${prefix} ${mode}\n${cell.value}\n`);
  }
  return output.join("\n");
}

/** Unmarshal a plaintext (.percival) file into an array of cell data. */
export function unmarshal(text: string): CellData[] {
  const data: CellData[] = [];
  const parts = text.split(/\r?\n(╔═╡|╔═╣) (Code|Markdown|Plot)\r?\n/);
  for (let i = 1; i < parts.length; i += 3) {
    const prefix = parts[i];
    const mode = parts[i + 1];
    let value = parts[i + 2];
    if (value.endsWith("\n")) {
      const offset = value.endsWith("\r\n") ? 2 : 1;
      value = value.substring(0, value.length - offset);
    }
    let type: keyof typeof modeNames | undefined = undefined;
    for (const [k, v] of Object.entries(modeNames)) {
      if (v === mode) {
        type = k as any;
        break;
      }
    }
    if (!type) {
      throw new Error(`Unknown cell mode "${mode}"`);
    }
    data.push({
      type,
      hidden: prefix === "╔═╣",
      value,
    });
  }
  return data;
}
