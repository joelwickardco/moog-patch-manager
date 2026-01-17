<script>
  let { value = $bindable(), placeholder = "Search..." } = $props();
</script>

<div class="relative">
  <input
    type="text"
    bind:value
    {placeholder}
    class="w-full px-4 py-2 pl-10 bg-background border border-border rounded-lg focus:outline-none focus:border-primary focus:ring-1 focus:ring-primary text-text-primary placeholder-text-secondary"
  />
  <span class="absolute left-3 top-1/2 -translate-y-1/2 text-text-secondary">
    🔍
  </span>
  {#if value}
    <button
      class="absolute right-3 top-1/2 -translate-y-1/2 text-text-secondary hover:text-text-primary"
      onclick={() => (value = "")}
    >
      ✕
    </button>
  {/if}
</div>
