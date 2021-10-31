<script lang="ts">
  import CellInput from "./CellInput.svelte";
  import CellOutput from "./CellOutput.svelte";

  export let type: "markdown" | "code";
  export let value: string;
</script>

<div>
  <CellOutput {value} />
  <CellInput {value} on:change />
</div>
