<script lang="ts">
  import { basicSetup } from "@codemirror/basic-setup";
  import { EditorState, Compartment } from "@codemirror/state";
  import { EditorView, KeyBinding, keymap } from "@codemirror/view";
  import { indentWithTab } from "@codemirror/commands";
  import { markdown } from "@codemirror/lang-markdown";
  import { createEventDispatcher, onDestroy, onMount } from "svelte";

  import { percival } from "@/lib/codemirror/language";
  import type { CellState } from "@/lib/notebook";

  const dispatch = createEventDispatcher();

  export let state: CellState;

  let currentValue: string;

  let editorParent: HTMLDivElement;
  let view: EditorView;
  let languageConf: Compartment;

  function makeRunKeyBinding(key: string): KeyBinding {
    return {
      key,
      run: () => {
        dispatch("change", { value: currentValue });
        return true;
      },
      preventDefault: true,
    };
  }

  $: languageExtension = state.type === "markdown" ? markdown : percival;
  $: if (view) {
    languageConf.reconfigure(languageExtension());
  }

  onMount(() => {
    currentValue = state.value;
    languageConf = new Compartment();
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
          languageConf.of(languageExtension()),
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
