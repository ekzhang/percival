<script lang="ts">
  import { EditorState, EditorView, basicSetup } from "@codemirror/basic-setup";
  import { markdown } from "@codemirror/lang-markdown";
  import { onMount } from "svelte";

  export let value: string;

  let editorParent: HTMLDivElement;
  let view: EditorView;

  onMount(() => {
    view = new EditorView({
      state: EditorState.create({
        extensions: [basicSetup, EditorView.lineWrapping, markdown()],
        doc: value,
      }),
      parent: editorParent,
    });
  });
</script>

<div bind:this={editorParent} />
