<script lang="ts">
  export let href: string;
  export let external: boolean = false;
</script>

<a
  {href}
  target={external ? "_blank" : undefined}
  rel={external ? "noopener noreferrer" : undefined}
  class="text-blue-600 hover:underline"><slot /></a
>
