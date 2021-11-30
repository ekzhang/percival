<script lang="ts">
  export let value: string;
</script>

{#if value}
  <div class="plot">{@html value}</div>
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
