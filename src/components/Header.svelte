<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import FaShareSquare from "svelte-icons/fa/FaShareSquare.svelte";
  import FaRegQuestionCircle from "svelte-icons/fa/FaRegQuestionCircle.svelte";

  import logo from "@/assets/logo.svg";
  import Dialog from "./Dialog.svelte";
  import Link from "./Link.svelte";
  import ShareCopy from "./ShareCopy.svelte";

  const dispatch = createEventDispatcher();

  export let sharing: "none" | "pending" | { id: string };

  let about = false;
</script>

<header class="h-16 border-b flex justify-center">
  <div class="w-full max-w-screen-lg flex justify-between items-center px-3">
    <a href="/">
      <img src={logo} alt="Percival logo" class="h-12" />
    </a>
    <div class="nav-buttons">
      <button
        on:click={() => dispatch("share")}
        class:loading={sharing === "pending"}
        disabled={sharing !== "none"}><i><FaShareSquare /></i> Share</button
      >
      <button on:click={() => (about = true)} disabled={about}
        ><i class="mb-0.5"><FaRegQuestionCircle /></i> About</button
      >
    </div>
  </div>
</header>

{#if sharing !== "none" && sharing !== "pending"}
  <Dialog on:close={() => dispatch("shareclose")}>
    <h2 class="text-2xl text-center font-bold font-serif mb-2">
      Notebook Sharing
    </h2>

    <p>Your notebook can be viewed at this link:</p>
    <div class="!mt-2 !mb-3">
      <ShareCopy value={`https://percival.ink/?gist=${sharing.id}`} />
    </div>

    <p>You can also see the source gist at:</p>
    <div class="!mt-2 !mb-3">
      <ShareCopy value={`https://gist.github.com/${sharing.id}`} />
    </div>
  </Dialog>
{/if}

{#if about}
  <Dialog on:close={() => (about = false)}>
    <img src={logo} alt="Percival logo" class="h-12 mx-auto" />
    <p>
      Percival is a declarative <em>data query and visualization language</em>.
      It provides a reactive, web-based notebook environment for exploring
      complex datasets, producing interactive graphics, and sharing results.
    </p>
    <p>
      Percival combines the interactive beauty of
      <Link external href="https://vega.github.io/vega/"><em>Vega</em></Link
      >-like visualization grammars with the flexibility of
      <Link external href="https://en.wikipedia.org/wiki/Datalog"
        ><em>Datalog</em></Link
      > as a query language for structured, relational data. These declarative components
      are combined through a reactive dataflow system, making it easy to quickly
      investigate datasets. Because Percival is built on web technologies, fully-interactive
      notebooks can be published to anyone with access to a web browser, making analyses
      more tangible to others.
    </p>
    <p>
      In addition to using Vega-Lite, Percival ships with a custom Datalog
      compiler that integrates with its notebook runtime. This compiles the
      query language to JavaScript through a staged evaluation process, which
      can be extended with user-provided JavaScript code. The interface aims to
      be lightweight, friendly, and accessible, and it has no hidden workspace
      state.
    </p>
    <p>
      The code is available on GitHub at <Link
        external
        href="https://github.com/ekzhang/percival">ekzhang/percival</Link
      >.
    </p>
  </Dialog>
{/if}

<style lang="postcss">
  .nav-buttons {
    @apply flex space-x-1;
  }

  .nav-buttons button {
    @apply px-2.5 pt-1.5 pb-0.5 font-serif font-medium text-lg rounded-md
      flex justify-center items-center text-gray-600 transition-colors;
  }
  .nav-buttons button:enabled {
    @apply text-gray-600 hover:text-black hover:bg-gray-100;
  }
  .nav-buttons button.loading {
    @apply text-blue-900 animate-pulse;
  }

  .nav-buttons button i {
    @apply inline-block h-5 w-5 mr-[5px];
  }
</style>
