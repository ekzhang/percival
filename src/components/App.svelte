<script lang="ts">
  import { onMount } from "svelte";
  import type { Readable } from "svelte/store";
  import { NotebookState } from "@/lib/notebook";
  import { createNotebookStore } from "@/lib/stores";
  import { marshal, unmarshal } from "@/lib/marshal";
  import starter from "@/samples/starter.percival?raw";
  import Header from "./Header.svelte";
  import Notebook from "./Notebook.svelte";

  let notebookStore: Readable<NotebookState> | undefined;
  $: notebook = notebookStore ? $notebookStore : undefined;

  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    if (params.has("new")) {
      // Construct an empty notebook.
      notebookStore = createNotebookStore(new NotebookState());
    } else {
      // Load either the starter notebook or a Gist.
      let text = starter;
      if (params.has("gist")) {
        try {
          const resp = await fetch(`/api?id=${params.get("gist")!}`);
          if (!resp.ok) {
            throw new Error(await resp.text());
          }
          text = await resp.text();
        } catch (error: any) {
          alert(`Error loading notebook: ${error.message}`);
        }
      }
      notebookStore = createNotebookStore(NotebookState.load(unmarshal(text)));
    }
  });

  /** Handler to prompt the user before navigating away from the page. */
  function beforeUnload(event: BeforeUnloadEvent) {
    event.preventDefault();
    event.returnValue = "";
    return "...";
  }

  let sharing: "none" | "pending" | { id: string } = "none";

  async function handleShare() {
    if (!notebook) return;
    sharing = "pending";
    try {
      const resp = await fetch("/api", {
        method: "POST",
        body: marshal(notebook.save()),
      });
      if (!resp.ok) {
        const text = await resp.text();
        throw new Error("Error when sharing notebook: " + text);
      }
      const result = await resp.json();
      sharing = { id: result.id };
    } catch (error: any) {
      alert(error.message);
      sharing = "none";
    }
  }
</script>

<svelte:window on:beforeunload={beforeUnload} />

<Header
  {sharing}
  on:share={handleShare}
  on:shareclose={() => (sharing = "none")}
/>
<main class="max-w-screen-lg mx-auto">
  {#if notebook === undefined}
    <div class="flex-1 space-y-6 py-1 pt-8 pb-24 px-6 animate-pulse">
      <div class="h-6 bg-blue-300 rounded w-3/4" />
      <div class="space-y-3">
        <div class="h-6 bg-blue-300 rounded" />
        <div class="h-6 bg-blue-300 rounded w-5/6" />
      </div>
    </div>
  {:else}
    <Notebook {notebook} />
  {/if}
</main>
