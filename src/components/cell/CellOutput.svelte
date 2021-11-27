<script lang="ts">
  import type { CellState } from "@/lib/notebook";
  import { ansiToHtml, markdownToHtml } from "@/lib/text";

  export let state: CellState;
</script>

{#if state.type === "markdown"}
  <div class="markdown-output">
    {@html markdownToHtml(state.value)}
  </div>
{:else if state.result.ok === false}
  <pre class="error">{@html ansiToHtml(state.result.errors)}</pre>
{:else if state.status === "pending"}
  {#if state.output === undefined}
    <div class="pending">
      <div class="animate-pulse bg-blue-200 rounded-md h-6 w-3/4 mb-4" />
      <div class="animate-pulse bg-blue-200 rounded-md h-6 w-100 mb-2" />
      <div class="animate-pulse bg-blue-200 rounded-md h-6 w-5/6" />
    </div>
  {:else}
    <div class="output stale">
      {JSON.stringify(state.output)}
    </div>
  {/if}
{:else if state.graphErrors !== undefined}
  <div class="error">
    <span class="text-red-600 font-semibold">[Graph Error]</span>
    {state.graphErrors}
  </div>
{:else if state.runtimeErrors !== undefined}
  <div class="error">
    <span class="text-red-600 font-semibold">[Runtime Error]</span>
    {state.runtimeErrors}
  </div>
{:else}
  <div class="output" class:stale={state.status === "stale"}>
    {JSON.stringify(state.output)}
  </div>
{/if}

<style lang="postcss">
  .pending {
    @apply mb-1 p-3 rounded-sm border border-blue-200;
  }

  .output {
    @apply mb-1 p-3 rounded-sm border border-slate-200;
  }

  .stale {
    @apply text-yellow-900 border-yellow-200 bg-yellow-50 animate-pulse;
  }

  .error {
    @apply mb-1 p-3 rounded-sm bg-pink-100 border border-pink-300;
  }

  .markdown-output {
    @apply px-2.5 font-serif text-base max-w-2xl leading-snug;
  }

  .markdown-output :global(h1) {
    @apply text-4xl font-bold my-4 border-b-2;
  }
  .markdown-output :global(h2) {
    @apply text-3xl font-bold my-3;
  }
  .markdown-output :global(h3) {
    @apply text-2xl font-bold my-3;
  }
  .markdown-output :global(h4) {
    @apply text-xl font-bold my-3;
  }
  .markdown-output :global(h5) {
    @apply text-lg font-bold my-3;
  }
  .markdown-output :global(h6) {
    @apply font-bold my-3;
  }

  .markdown-output :global(p) {
    @apply my-3;
  }

  .markdown-output :global(pre) {
    @apply my-3 px-2 py-1 border border-gray-300 rounded-sm;
  }

  .markdown-output :global(code) {
    @apply text-sm text-gray-900;
  }
</style>
