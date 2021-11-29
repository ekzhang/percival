<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { fade } from "svelte/transition";

  const dispatch = createEventDispatcher();
</script>

<div
  class="fixed top-0 left-0 bottom-0 right-0 z-50 bg-black bg-opacity-60 flex justify-center items-center"
  on:click={() => dispatch("close")}
  transition:fade={{ duration: 100 }}
>
  <div
    class="w-4/5 max-w-xl max-h-[90%] bg-white shadow-lg rounded-md p-4 md:p-6 space-y-3 overflow-y-auto"
    on:click={(event) => event.stopPropagation()}
  >
    <slot />
    <button
      class="w-full p-1.5 rounded-md text-gray-600 hover:bg-gray-100 hover:text-black active:bg-gray-200 active:text-black transition-colors"
      on:click={() => dispatch("close")}
    >
      Close this message
    </button>
  </div>
</div>
