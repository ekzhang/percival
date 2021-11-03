<script lang="ts">
  import { unified } from "unified";
  import remarkParse from "remark-parse";
  import remarkMath from "remark-math";
  import remarkRehype from "remark-rehype";
  import rehypeKatex from "rehype-katex";
  import rehypeStringify from "rehype-stringify";

  export let value: string;

  const pipeline = unified()
    .use(remarkParse)
    .use(remarkMath)
    .use(remarkRehype)
    .use(rehypeKatex)
    .use(rehypeStringify);

  $: rendered = pipeline.processSync(value);
</script>

<div class="markdown-output">
  {@html rendered}
</div>

<style lang="postcss">
  .markdown-output {
    @apply pl-2.5 font-serif text-base max-w-2xl leading-snug;
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
    @apply my-3 px-2 py-1 border rounded-sm;
  }

  .markdown-output :global(code) {
    @apply text-sm text-gray-900;
  }
</style>
