<script lang="ts">
  import { NotebookState } from "@/lib/notebook";
  import { createNotebookStore } from "@/lib/stores";
  import { marshal, unmarshal } from "@/lib/marshal";
  import starter from "@/samples/starter.percival?raw";
  import Header from "./Header.svelte";
  import Notebook from "./Notebook.svelte";

  const notebookStore = createNotebookStore(
    NotebookState.load(unmarshal(starter)),
  );
  $: notebook = $notebookStore;

  let sharing: "none" | "pending" | { id: string } = "none";

  async function handleShare() {
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

<Header
  {sharing}
  on:share={handleShare}
  on:shareclose={() => (sharing = "none")}
/>
<main class="max-w-screen-lg mx-auto">
  <Notebook {notebook} />
</main>
