<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { fade } from "svelte/transition";
  import FaChevronRight from "svelte-icons/fa/FaChevronRight.svelte";
  import FaTrashAlt from "svelte-icons/fa/FaTrashAlt.svelte";

  import type { CellState } from "@/lib/notebook";
  import CellInput from "./CellInput.svelte";
  import CellOutput from "./CellOutput.svelte";

  const dispatch = createEventDispatcher();

  export let state: CellState;

  function toggleDisplayInput() {
    dispatch("toggle");
  }
</script>

<div class="cell" transition:fade>
  <button class="sidebar" on:click={toggleDisplayInput}>
    <div class="mb-3" class:rotate-90={!state.hidden}>
      <FaChevronRight />
    </div>
    <button
      class="text-gray-400 hover:text-red-600 transition-colors"
      class:hidden={state.hidden}
      on:click|stopPropagation={() => dispatch("delete")}
    >
      <FaTrashAlt />
    </button>
  </button>
  <CellOutput {state} />
  <CellInput {state} on:change />
</div>

<style lang="postcss">
  .cell {
    @apply relative min-h-[32px] my-0 pl-0.5;
  }

  .cell:hover .sidebar {
    @apply opacity-100;
  }

  .sidebar {
    left: calc(-50vw + 50% - 0.6rem);
    width: calc(50vw - 50% + 0.6rem);
    @apply absolute h-full
      flex flex-col justify-start items-end space-y-2 p-1 pt-2 border-r-2 hover:border-gray-300
      transition-all hover:bg-zinc-50 opacity-0 text-gray-400 hover:text-gray-800;
  }

  .sidebar > * {
    @apply w-4 h-4 transition-all;
  }
</style>
