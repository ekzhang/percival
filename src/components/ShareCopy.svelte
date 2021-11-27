<script lang="ts">
  export let value: string;

  let copied = false;

  async function copy() {
    try {
      await navigator.clipboard.writeText(value);
      if (!copied) {
        copied = true;
        setTimeout(() => (copied = false), 2000);
      }
    } catch (error: any) {
      alert(error.message);
    }
  }
</script>

<div class="container">
  <input readonly {value} />
  <div
    class="absolute w-full h-full pr-2 flex flex-col justify-center items-end pointer-events-none"
  >
    <button on:click={copy} class:copied>{copied ? "Done!" : "Copy"}</button>
  </div>
</div>

<style lang="postcss">
  .container {
    @apply flex relative;
  }

  .container:hover input {
    @apply border-blue-600 text-black bg-gray-50;
  }

  input {
    @apply text-[0.92rem] w-full py-1.5 pl-3 pr-[72px] rounded-md border border-gray-300
      text-gray-700 outline-none transition-colors;
  }

  button {
    @apply w-[58px] text-sm font-semibold bg-gray-100 rounded-md py-0.5
      pointer-events-auto transition-colors hover:bg-gray-200 active:bg-gray-300;
  }

  button.copied {
    @apply bg-green-100 hover:bg-green-200 active:bg-green-300;
  }
</style>
