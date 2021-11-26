<script lang="ts">
  import { EditorState, basicSetup } from "@codemirror/basic-setup";
  import { EditorView, KeyBinding, keymap } from "@codemirror/view";
  import { indentWithTab } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { createEventDispatcher, onDestroy, onMount } from "svelte";

  import type { CellState } from "@/lib/notebook";

  const dispatch = createEventDispatcher();

  export let state: CellState;

  let currentValue: string;

  let editorParent: HTMLDivElement;
  let view: EditorView;

  function run() {
    dispatch("change", { value: currentValue });
  }

  function makeRunKeyBinding(key: string): KeyBinding {
    return {
      key,
      run: () => {
        run();
        return true;
      },
      preventDefault: true,
    };
  }

  onMount(() => {
    currentValue = state.value;
    view = new EditorView({
      state: EditorState.create({
        doc: state.value,
        extensions: [
          basicSetup,
          EditorView.lineWrapping,
          keymap.of([
            indentWithTab,
            makeRunKeyBinding("Shift-Enter"),
            makeRunKeyBinding("Ctrl-Enter"),
          ]),
          markdown(),
          EditorView.updateListener.of((update) => {
            currentValue = update.state.doc.toJSON().join("\n");
          }),
        ],
      }),
      parent: editorParent,
    });
  });

  onDestroy(() => {
    view.destroy();
  });
</script>

<div bind:this={editorParent} class:dirty={state.value !== currentValue} />

<style lang="postcss">
  .dirty :global(.cm-editor .cm-scroller) {
    @apply border-orange-200 bg-orange-50;
  }
  .dirty :global(.cm-editor .cm-line) {
    @apply border-orange-100;
  }
  .dirty :global(.cm-editor .cm-line.cm-activeLine) {
    @apply border-orange-300;
  }
</style>
