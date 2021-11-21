<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { fade } from "svelte/transition";
  import FaChevronDown from "svelte-icons/fa/FaChevronDown.svelte";
  import FaChevronRight from "svelte-icons/fa/FaChevronRight.svelte";
  import FaTrashAlt from "svelte-icons/fa/FaTrashAlt.svelte";

  import type { CellData } from "@/lib/notebook";
  import CellInput from "./CellInput.svelte";
  import CellOutput from "./CellOutput.svelte";

  const dispatch = createEventDispatcher();

  export let data: CellData;
</script>

<div class="cell" transition:fade>
  <button class="sidebar" on:click={() => dispatch("toggle")}>
    <div class="w-4 h-4">
      {#if data.hidden}
        <FaChevronRight />
      {:else}
        <FaChevronDown />
      {/if}
    </div>
    <button
      class="w-4 h-4 text-gray-400 hover:text-red-600 transition-colors"
      on:click={() => dispatch("delete")}
    >
      <FaTrashAlt />
    </button>
  </button>
  <CellOutput {data} />
  {#if !data.hidden}
    <CellInput {data} on:change />
  {/if}
</div>

<style lang="postcss">
  .cell {
    @apply relative min-h-[32px];
  }

  .cell:hover .sidebar {
    @apply opacity-100;
  }

  .sidebar {
    @apply absolute h-full left-[-2000px] w-[2000px]
      flex justify-end items-start space-x-2 p-2
      transition-all hover:bg-zinc-50 opacity-0 text-gray-400 hover:text-gray-800;
  }
</style>
