<script lang="ts">
  import { NotebookState } from "@/lib/notebook";
  import { createNotebookStore } from "@/lib/stores";
  import { onMount } from "svelte";
  import Cell from "./cell/Cell.svelte";
  import CellDivider from "./cell/CellDivider.svelte";

  const notebookStore = createNotebookStore(new NotebookState());
  $: notebook = $notebookStore;

  onMount(() => {
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
  });
</script>

<div class="space-y-3 pt-8 pb-24 px-3">
  {#each [...notebook] as [id, cell] (id)}
    <CellDivider
      on:create={(event) => {
        notebook.addCellBefore(id, {
          type: event.detail.type,
          value: "",
          hidden: false,
        });
      }}
    />
    <Cell
      state={cell}
      on:change={(event) => notebook.editCell(id, event.detail.value)}
      on:toggle={() => notebook.toggleHidden(id)}
      on:delete={() => notebook.deleteCell(id)}
    />
  {/each}
  <CellDivider
    on:create={(event) => {
      notebook.addCell({
        type: event.detail.type,
        value: "",
        hidden: false,
      });
    }}
  />
</div>
