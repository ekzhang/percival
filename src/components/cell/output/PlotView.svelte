<script lang="ts">
  import { isRenderedElement } from "@/lib/types";
  import RelationView from "./RelationView.svelte";

  export let name: string | undefined;
  export let value: unknown;
</script>

{#if value}
  {#if isRenderedElement(value)}
    <div class="plot">{@html value.outerHTML}</div>
  {:else if Array.isArray(value)}
    <RelationView name={name ?? ""} values={value} />
  {:else}
    <div class="font-mono text-[0.95rem] text-gray-700">
      {#if name !== undefined}
        <span class="font-bold">{name}</span>
        <span class="text-gray-400">:=</span>
      {/if}
      {JSON.stringify(value, undefined, 2)}
    </div>
  {/if}
{:else}
  <span class="italic text-sm font-light">(fresh pixels for a plot...)</span>
{/if}

<style lang="postcss">
  .plot :global(svg.plot) {
    display: block;
    font: 10px system-ui, sans-serif;
    background: #fff;
    height: auto;
    max-width: 100%;
  }
  .plot :global(svg.plot text) {
    white-space: pre;
  }
</style>
