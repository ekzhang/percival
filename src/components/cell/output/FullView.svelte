<script lang="ts">
  import type { RelationSet } from "@/lib/types";

  import RelationView from "./RelationView.svelte";

  export let value: RelationSet;
</script>

<div class="flex flex-col space-y-3">
  {#each Object.entries(value) as [name, values] (name)}
    <RelationView {name} {values} />
  {:else}
    <span class="italic text-sm font-light">(there's nothing here yet...)</span>
  {/each}
</div>
