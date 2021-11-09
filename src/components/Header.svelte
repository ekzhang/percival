<script lang="ts">
  import { fade } from "svelte/transition";

  import logo from "../assets/logo.svg";
  import Link from "./Link.svelte";

  let about = false;
</script>

<header
  class="h-16 border-b flex justify-center hover:bg-zinc-50 transition-colors"
>
  <div class="w-full max-w-screen-lg flex justify-between items-center px-3">
    <a href="/">
      <img src={logo} alt="Percival logo" class="h-12" />
    </a>
    <button
      class="px-2 pt-1 font-serif font-medium text-lg text-gray-500 hover:text-black transition-colors"
      on:click={() => (about = true)}
    >
      About
    </button>
  </div>
</header>

{#if about}
  <div
    class="fixed top-0 left-0 bottom-0 right-0 z-50 bg-black bg-opacity-60 flex justify-center items-center"
    on:click={() => (about = false)}
    transition:fade={{ duration: 100 }}
  >
    <div
      class="w-4/5 max-w-lg max-h-[90%] bg-white shadow-lg rounded-md p-4 space-y-3 text-gray-900 overflow-y-auto"
      on:click={(event) => event.stopPropagation()}
    >
      <img src={logo} alt="Percival logo" class="h-12 mx-auto" />
      <p>
        Percival is a declarative <em>data query and visualization language</em
        >. It provides a reactive, web-based notebook environment for exploring
        complex datasets, producing interactive graphics, and sharing results.
      </p>
      <p>
        Percival combines the interactive beauty of
        <Link external href="https://vega.github.io/vega/"><em>Vega</em></Link
        >-like visualization grammars with the flexibility of
        <Link external href="https://en.wikipedia.org/wiki/Datalog"
          ><em>Datalog</em></Link
        > as a query language for structured, relational data. These declarative
        components are combined through a reactive dataflow system, making it easy
        to quickly investigate datasets. Because Percival is built on web technologies,
        fully-interactive notebooks can be published to anyone with access to a web
        browser, making analyses more tangible to others.
      </p>
      <p>
        In addition to using Vega-Lite, Percival ships with a custom Datalog
        compiler that integrates with its notebook runtime. This compiles the
        query language to JavaScript through a staged evaluation process, which
        can be extended with user-provided JavaScript code. The interface aims
        to be lightweight, friendly, and accessible, and it has no hidden
        workspace state.
      </p>
      <p>
        The code is available on GitHub at <Link
          external
          href="https://github.com/ekzhang/percival">ekzhang/percival</Link
        >.
      </p>
      <button
        class="w-full text-gray-500 hover:text-gray-800"
        on:click={() => (about = false)}
      >
        -- Close this message --
      </button>
    </div>
  </div>
{/if}
