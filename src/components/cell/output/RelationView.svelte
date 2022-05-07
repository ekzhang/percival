<script lang="ts">
  import type { Relation } from "@/lib/types";

  import ValueView from "./ValueView.svelte";

  export let name: string;
  export let values: Relation;

  let displaying = 0;
  $: displaying = Math.min(values.length, 5); // hide long lists
</script>

<div class="font-mono text-[0.95rem] text-gray-700">
  <span class="text-gray-500">Table</span>&lbrace;<span class="font-bold"
    >{name}</span
  >&rbrace; <span class="text-gray-400">:=</span>
  [
  <div class="pl-[2ch]">
    {#each values.slice(0, displaying) as value}
      <div class="pl-[2ch]"><ValueView {name} {value} />,</div>
    {/each}
    {#if displaying < values.length}
      <button
        class="bg-gray-50 border rounded-full px-2 text-sm font-sans hover:bg-gray-100 active:bg-gray-200 transition-colors"
        on:click={() => (displaying = Math.min(displaying + 10, values.length))}
      >
        {values.length - displaying} more {values.length - displaying >= 2
          ? "items"
          : "item"}…
      </button>
    {/if}
  </div>
  ]
</div>
