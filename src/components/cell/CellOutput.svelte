<script lang="ts">
  import { unified } from "unified";
  import remarkParse from "remark-parse";
  import remarkMath from "remark-math";
  import remarkRehype from "remark-rehype";
  import rehypeKatex from "rehype-katex";
  import rehypeStringify from "rehype-stringify";
  import prettier from "prettier";
  import parserBabel from "prettier/parser-babel";
  import AnsiUp from "ansi_up";
  import { compile } from "percival-wasm";

  import type { CellData } from "@/lib/notebook";

  export let data: CellData;

  function prettify(code: string): string {
    return prettier.format(code, { parser: "babel", plugins: [parserBabel] });
  }

  const ansi_up = new AnsiUp();

  const pipeline = unified()
    .use(remarkParse)
    .use(remarkMath)
    .use(remarkRehype)
    .use(rehypeKatex)
    .use(rehypeStringify);

  $: rendered =
    data.type === "markdown" ? pipeline.processSync(data.value) : null;

  $: compiled = data.type === "code" ? compile(data.value) : null;
</script>

{#if data.type === "markdown"}
  <div class="markdown-output">
    {@html rendered}
  </div>
{:else if compiled.is_ok()}
  <pre class="output">{prettify(compiled.src())}</pre>
{:else}
  <pre class="error">{@html ansi_up.ansi_to_html(compiled.err())}</pre>
{/if}

<style lang="postcss">
  .output {
    @apply mb-1 p-3 rounded-sm bg-green-100 border border-green-300 max-h-64 overflow-y-auto;
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
