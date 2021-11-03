<script lang="ts">
  import { fade } from "svelte/transition";

  import logo from "../assets/logo.svg";

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
      class="w-4/5 max-w-lg max-h-4/5 bg-white shadow-lg rounded-md p-4 space-y-3 text-gray-900 overflow-y-auto"
      on:click={(event) => event.stopPropagation()}
    >
      <img src={logo} alt="Percival logo" class="h-12 mx-auto" />
      <p>
        Percival is a reactive editor for declarative data analysis and
        visualization, enabling flexible, high-fidelity analyses of complex
        datasets.
      </p>
      <p>
        In Percival, the user writes code in a combined data querying and
        visualization language. The web interface provides a notebook
        environment with a reactive execution model, making it easy to answer
        real-world questions and communicate results.
      </p>
      <p>
        Percival supports data querying, joining, and aggregation using a custom
        dialect of Datalog built for reactive execution, which is compiled to
        JavaScript through a staged evaluation process. It is lightweight, has
        no hidden workspace state, and provides a friendly and accessible editor
        UI.
      </p>
      <p>
        The code is available on GitHub at <a
          href="https://github.com/ekzhang/percival"
          target="_blank"
          rel="noopener noreferrer"
          class="underline">ekzhang/percival</a
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
