<script lang="ts">
  import CellInput from "./CellInput.svelte";
  import CellOutput from "./CellOutput.svelte";

  export let type: "markdown" | "code";
  export let value: string;
</script>

<CellOutput {value} />
<CellInput />
