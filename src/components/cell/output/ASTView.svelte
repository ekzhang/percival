<script lang="ts">
  import type { CellState } from "@/lib/notebook";
  import JSONTree from "svelte-json-tree";

  export let state: CellState;

  let expandAll = false;
  $: level = expandAll ? 10 : 0;
</script>

{#if state.type === "code" && state.displayDebug && state.result.ok && state.result.ast !== undefined}
  <div class="flex flex-col space-y-3 px-3 py-1">
    <div class="font-mono text-[0.95rem] text-gray-700 json-tree ">
      <div class="float-left inline-flex flex-row mr-4">
        <span class="text-gray-500">AST</span>
        <button
          class="button"
          on:click={() => {
            expandAll = !expandAll;
          }}
        >
          Toggle
        </button>
        <span class="text-gray-400 pr-2">:=</span>
      </div>
      {#key expandAll}
        <JSONTree value={state.result.ast} defaultExpandedLevel={level} />
      {/key}
    </div>
  </div>
{/if}

<style lang="postcss">
  .button {
    @apply font-sans text-[0.8rem] bg-white z-10 px-1 py-0.5 text-xs rounded-md shadow
      transition-colors hover:bg-gray-50 active:bg-gray-200 mx-1;
  }

  /* https://github.com/tanhauhau/svelte-json-tree#overriding-styles */
  .json-tree {
    --json-tree-string-color: theme(colors.rose[700]);
    --json-tree-symbol-color: theme(colors.rose[700]);
    --json-tree-boolean-color: #112aa7;
    --json-tree-function-color: #112aa7;
    --json-tree-number-color: theme(colors.blue[600]);
    --json-tree-label-color: theme(colors.gray[700]);
    --json-tree-property-color: theme(colors.gray[500]);
    --json-tree-arrow-color: #727272;
    --json-tree-operator-color: theme(colors.gray[700]);
    --json-tree-null-color: #8d8d8d;
    --json-tree-undefined-color: #8d8d8d;
    --json-tree-date-color: #8d8d8d;
    --json-tree-internal-color: grey;
    --json-tree-regex-color: #cb3f41;
    --json-tree-font-size: 0.95rem;
    --json-tree-font-family: theme(fontFamily.mono);
  }
</style>
