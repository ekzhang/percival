<script lang="ts">
  import ValueView from "./ValueView.svelte";

  export let name: string;
  export let values: object[];
</script>

<div class="font-mono text-gray-700">
  <span class="text-gray-500">Table</span>&lbrace;<span class="font-bold"
    >{name}</span
  >&rbrace; <span class="text-gray-400">:=</span>
  [
  <div class="pl-[2ch]">
    {#each values as value}
      <div><ValueView {name} {value} />,</div>
    {/each}
  </div>
  ]
</div>
