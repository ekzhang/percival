<script lang="ts">
  import { NotebookState } from "@/lib/notebook";
  import { createNotebookStore } from "@/lib/stores";
  import Cell from "./cell/Cell.svelte";
  import CellDivider from "./cell/CellDivider.svelte";
  import starter from "@/samples/starter.percival?raw";
  import { unmarshal } from "@/lib/marshal";

  const notebookStore = createNotebookStore(
    NotebookState.load(unmarshal(starter)),
  );
  $: notebook = $notebookStore;
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
