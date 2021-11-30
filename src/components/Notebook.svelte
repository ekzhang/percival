<script lang="ts">
  import type { NotebookState } from "@/lib/notebook";
  import Cell from "./cell/Cell.svelte";
  import CellDivider from "./cell/CellDivider.svelte";

  export let notebook: NotebookState;
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
    visible={notebook.length === 0}
    on:create={(event) => {
      notebook.addCell({
        type: event.detail.type,
        value: "",
        hidden: false,
      });
    }}
  />
</div>
