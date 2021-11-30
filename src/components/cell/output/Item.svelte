<script lang="ts">
  export let item: any;
</script>

{#if typeof item === "number" || typeof item === "bigint"}
  <span class="text-blue-600">{item}</span>
{:else if typeof item === "string"}
  <span class="text-rose-700">"{item}"</span>
{:else if typeof item === "boolean"}
  <span class="text-fuchsia-700">{item}</span>
{:else if item instanceof Date}
  <span class="text-green-700">{item.toISOString()}</span>
{:else}
  <span class="text-green-700">{item}</span>
{/if}
