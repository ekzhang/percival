<script lang="ts">
  import { CellData, NotebookState } from "@/lib/notebook";
  import Cell from "./cell/Cell.svelte";
  import CellDivider from "./cell/CellDivider.svelte";

  let notebook = new NotebookState();

  function handleChange(cell: CellData, value: string) {
    cell.value = value;
    notebook = notebook;
  }

  function handleCreate(index: number, type: "markdown" | "code") {
    notebook.insertCell(index, { type, value: "", hidden: false });
    notebook = notebook;
  }

  notebook.addCell({
    type: "markdown",
    value: `# A Beginner's Notebook

Hello world! Welcome to my new cell :)

\`\`\`
.decl alias(a:var, b:var) output
alias(X, X) :- assign(X, _).
alias(X, X) :- assign(_, X).
alias(X, Y) :- assign(X, Y).
alias(X, Y) :- ld(X, A, F), alias(A, B), st(B, F, Y).
\`\`\`

The above is a code block.`,
    hidden: false,
  });
  notebook.addCell({
    type: "markdown",
    value: `## This is my first Percival notebook.

Hello **world**! My \`name\` is _Eric_. This is an example of a Markdown cell.

You can edit the source of this cell below and press \`Shift+Enter\` to see your changes.

Ducimus ex veniam distinctio ut maxime. Rerum earum distinctio amet voluptates. Ea aut saepe consectetur rem natus qui et voluptas.

Voluptate voluptatem tempora rerum ipsam accusantium dolorum fuga veniam. Tenetur harum doloribus ex saepe. Consectetur autem omnis nesciunt. Vel sit dicta esse aspernatur nesciunt. Voluptate quo non rerum omnis recusandae blanditiis consequatur.

![Picture of a mountain](https://upload.wikimedia.org/wikipedia/commons/thumb/3/3f/Fronalpstock_big.jpg/800px-Fronalpstock_big.jpg)

Hopefully you see the image above!`,
    hidden: false,
  });
</script>

<div class="space-y-3 pt-8 pb-24 px-3">
  {#each notebook.cells as cell, i (cell)}
    <CellDivider on:create={(event) => handleCreate(i, event.detail.type)} />
    <Cell
      data={cell}
      on:change={(event) => handleChange(cell, event.detail.value)}
      on:toggle={() => {
        cell.hidden = !cell.hidden;
        notebook = notebook;
      }}
      on:delete={() => {
        notebook.deleteCell(i);
        notebook = notebook;
      }}
    />
  {/each}
  <CellDivider
    on:create={(event) =>
      handleCreate(notebook.cells.length, event.detail.type)}
  />
</div>
