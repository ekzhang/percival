<script lang="ts">
  import Item from "./Item.svelte";

  export let name: string;
  export let value: object;

  $: entries = Object.entries(value);
</script>

<span class="value">
  <span class="font-bold text-gray-700">{name}</span
  >({#each entries as [key, item], i (key)}<span
      ><span class="text-gray-500">{key}:</span>
      <Item {item} />{#if i < entries.length - 1},
      {/if}</span
    >{/each})
</span>

<style>
  .value {
    text-indent: 2ch;
    margin-left: -2ch;
  }
</style>
