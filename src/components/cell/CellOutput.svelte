<script lang="ts">
  import SvelteMarkdown from "svelte-markdown";

  export let value: string;
</script>

<div class="markdown-output">
  <SvelteMarkdown source={value} />
</div>

<style lang="postcss">
  .markdown-output {
    @apply font-serif text-base max-w-2xl leading-snug;
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

  .markdown-output :global(code) {
    @apply text-sm text-gray-900 px-1 py-0.5 bg-gray-50 border rounded-sm;
  }
</style>
