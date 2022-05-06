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
        <PlotView value={state.output} />
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

  .markdown-output :global(:where(h1, h2, h3, h4, h5, h6) span.icon.icon-link) {
    @apply transition-all;
  }

  .markdown-output
    :global(:where(h1, h2, h3, h4, h5, h6):hover span.icon.icon-link) {
    display: inline;
    background-image: url('data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" version="1.1" width="16" height="16" aria-hidden="true"><path fill-rule="evenodd" d="M7.775 3.275a.75.75 0 001.06 1.06l1.25-1.25a2 2 0 112.83 2.83l-2.5 2.5a2 2 0 01-2.83 0 .75.75 0 00-1.06 1.06 3.5 3.5 0 004.95 0l2.5-2.5a3.5 3.5 0 00-4.95-4.95l-1.25 1.25zm-4.69 9.64a2 2 0 010-2.83l2.5-2.5a2 2 0 012.83 0 .75.75 0 001.06-1.06 3.5 3.5 0 00-4.95 0l-2.5 2.5a3.5 3.5 0 004.95 4.95l1.25-1.25a.75.75 0 00-1.06-1.06l-1.25 1.25a2 2 0 01-2.83 0z"></path></svg>');
    background-repeat: no-repeat;
    background-position: center center;
    background-size: 0.75em;
    padding-left: 1em;
    padding-right: 0.5em;
  }

  .markdown-output
    :global(:where(h1, h2, h3, h4, h5, h6):hover span.icon.icon-link:hover) {
    background-size: 0.9em;
    filter: invert(0.2);
  }
</style>
