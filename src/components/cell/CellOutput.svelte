<script lang="ts">
  import type { CellState } from "@/lib/notebook";
  import { ansiToHtml, markdownToHtml } from "@/lib/text";

  export let state: CellState;
</script>

{#if state.type === "markdown"}
  <div class="markdown-output">
    {@html markdownToHtml(state.value)}
  </div>
{:else if state.compilerResult.ok === false}
  <pre class="error">{@html ansiToHtml(state.compilerResult.errors)}</pre>
{:else if state.status === "pending"}
  <div class="pending">Pending...</div>
{:else}
  <div class="output">Done! (TODO: Display outputs.)</div>
{/if}

<style lang="postcss">
  .pending {
    @apply mb-1 p-3 rounded-sm bg-cyan-50 border border-cyan-300 text-cyan-800 italic;
  }

  .output {
    @apply mb-1 p-3 rounded-sm border border-slate-200;
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
