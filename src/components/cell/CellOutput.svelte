<script lang="ts">
  import type { CellState } from "@/lib/notebook";
  import { ansiToHtml, markdownToHtml } from "@/lib/text";
  import FullView from "./output/FullView.svelte";
  import PlotView from "./output/PlotView.svelte";

  export let state: CellState;
</script>

{#if state.type === "markdown"}
  <div class="markdown-output">
    {@html markdownToHtml(state.value)}
  </div>
{:else if state.result.ok === false}
  {#if state.type === "code"}
    <pre class="error">{@html ansiToHtml(state.result.errors)}</pre>
  {:else}
    <div class="error">
      <span class="text-red-700">Error:</span>
      {state.result.error}
    </div>
  {/if}
{:else if state.graphErrors !== undefined}
  <div class="error">
    <span class="text-red-700">Graph Error:</span>
    {state.graphErrors}
  </div>
{:else if state.runtimeErrors !== undefined}
  <div class="error">
    <span class="text-red-700">Runtime Error:</span>
    {state.runtimeErrors}
  </div>
{:else}
  <div
    class="output"
    class:stale={state.status === "stale"}
    class:pending={state.status === "pending"}
  >
    {#if state.output !== undefined}
      {#if state.type === "code"}
        <FullView value={state.output} />
      {:else}
        <PlotView name={state.result.results[0]} value={state.output} />
      {/if}
    {/if}
  </div>
{/if}

<style lang="postcss">
  .error {
    @apply mb-1 p-3 rounded-sm bg-pink-100 font-mono overflow-auto;
  }

  .output {
    @apply p-3 rounded-sm;
  }
  .output.stale {
    @apply border-yellow-200 opacity-50;
    background-image: repeating-linear-gradient(
      45deg,
      theme(colors.yellow.50),
      theme(colors.yellow.50) 10px,
      theme(colors.yellow.100) 10px,
      theme(colors.yellow.100) 20px
    );
    background-size: 200% 200%;
    animation: move-stripes 0.75s linear 0s infinite;
  }
  .output.pending {
    @apply border-cyan-200 opacity-50;
    background-image: repeating-linear-gradient(
      45deg,
      theme(colors.cyan.50),
      theme(colors.cyan.50) 10px,
      theme(colors.cyan.100) 10px,
      theme(colors.cyan.100) 20px
    );
    background-size: 200% 200%;
    animation: move-stripes 0.75s linear 0s infinite;
  }

  @keyframes move-stripes {
    from {
      background-position: 0 0;
    }
    to {
      background-position: -28.28px 0;
    }
  }

  .markdown-output {
    @apply px-2.5 font-serif text-base text-[1.02rem] max-w-2xl leading-snug;
  }

  .markdown-output :global(h1) {
    @apply text-4xl font-semibold my-4 border-b-2 border-black;
  }
  .markdown-output :global(h2) {
    @apply text-3xl font-semibold my-3;
  }
  .markdown-output :global(h3) {
    @apply text-2xl font-semibold my-3;
  }
  .markdown-output :global(h4) {
    @apply text-xl font-semibold my-3;
  }
  .markdown-output :global(h5) {
    @apply text-lg font-semibold my-3;
  }
  .markdown-output :global(h6) {
    @apply font-semibold my-3;
  }

  .markdown-output :global(p) {
    @apply my-3;
  }

  .markdown-output :global(a) {
    @apply hover:underline text-blue-600;
  }

  .markdown-output :global(ul) {
    @apply list-disc pl-7;
  }
  .markdown-output :global(ol) {
    @apply list-decimal pl-7;
  }

  .markdown-output :global(pre) {
    @apply my-3 px-2 py-1 border border-gray-300 rounded-sm;
  }

  .markdown-output :global(code) {
    @apply text-sm text-gray-900;
  }
</style>
