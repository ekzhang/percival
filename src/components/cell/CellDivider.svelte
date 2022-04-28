<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import FaPlus from "svelte-icons/fa/FaPlus.svelte";

  const dispatch = createEventDispatcher();

  export let visible: boolean = false;

  const types = [
    { type: "code", label: "Code Cell" },
    { type: "markdown", label: "Markdown Cell" },
    { type: "plot", label: "Plot Cell" },
  ];
</script>

<div class="divider" class:visible>
  {#each types as { type, label }}
    <button on:click={() => dispatch("create", { type })} tabindex="-1">
      <div class="h-3 w-3 mr-[4px]"><FaPlus /></div>
      <span>{label}</span>
    </button>
  {/each}
  <hr />
</div>

<style lang="postcss">
  .divider {
    z-index: 10;
    @apply -my-1 py-1 relative flex justify-center items-center transition-opacity
      opacity-0 hover:opacity-100 focus:opacity-100;
    /* @apply border border-dotted border-red-500; */
  }

  .divider.visible {
    @apply opacity-100;
  }

  .divider > hr {
    @apply absolute w-full border-dotted border-gray-300;
  }

  .divider > button {
    @apply flex items-center bg-white z-10 px-2.5 py-1 text-xs rounded-md shadow
      transition-colors hover:bg-gray-50 active:bg-gray-200;
  }

  .divider > button:not(:first-child) {
    @apply ml-4;
  }
</style>
