<script lang="ts">
import { isRenderedElement } from "@/lib/types";


  export let value: unknown;
</script>

{#if value}
{#if isRenderedElement(value)}
  <div class="plot">{@html value.outerHTML}</div>
{:else}
  <div>{JSON.stringify(value)}</div>
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
