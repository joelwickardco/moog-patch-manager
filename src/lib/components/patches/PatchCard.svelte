<script>
  let { patch, listView = false } = $props();

  function toggleFavorite() {
    // Will call Tauri command
    patch.is_favorite = !patch.is_favorite;
  }
</script>

{#if listView}
  <div class="flex items-center gap-4 p-4 bg-surface rounded-lg hover:bg-border/50 transition-colors">
    <button
      class="text-2xl {patch.is_favorite ? 'text-favorite' : 'text-text-secondary hover:text-favorite'}"
      onclick={toggleFavorite}
    >
      {patch.is_favorite ? "★" : "☆"}
    </button>
    <div class="flex-1">
      <h3 class="font-medium">{patch.name}</h3>
      {#if patch.categories?.length}
        <div class="flex gap-1 mt-1">
          {#each patch.categories as category}
            <span class="text-xs px-2 py-0.5 rounded bg-category-bass/20 text-category-bass">
              {category}
            </span>
          {/each}
        </div>
      {/if}
    </div>
    <div class="flex gap-2">
      <button class="p-2 hover:bg-surface rounded" title="Edit">
        <span class="text-text-secondary">✏️</span>
      </button>
      <button class="p-2 hover:bg-surface rounded" title="Delete">
        <span class="text-text-secondary">🗑️</span>
      </button>
    </div>
  </div>
{:else}
  <div class="bg-surface rounded-lg p-4 hover:bg-border/50 transition-colors">
    <div class="flex items-start justify-between mb-2">
      <h3 class="font-medium truncate flex-1">{patch.name}</h3>
      <button
        class="text-xl ml-2 {patch.is_favorite ? 'text-favorite' : 'text-text-secondary hover:text-favorite'}"
        onclick={toggleFavorite}
      >
        {patch.is_favorite ? "★" : "☆"}
      </button>
    </div>

    {#if patch.categories?.length}
      <div class="flex flex-wrap gap-1 mb-2">
        {#each patch.categories as category}
          <span class="text-xs px-2 py-0.5 rounded bg-category-bass/20 text-category-bass">
            {category}
          </span>
        {/each}
      </div>
    {/if}

    {#if patch.notes}
      <p class="text-sm text-text-secondary line-clamp-2">{patch.notes}</p>
    {/if}

    <div class="flex gap-2 mt-3 pt-3 border-t border-border">
      <button class="flex-1 py-1 text-sm hover:bg-border rounded transition-colors">
        Edit
      </button>
      <button class="flex-1 py-1 text-sm hover:bg-border rounded transition-colors text-red-400">
        Delete
      </button>
    </div>
  </div>
{/if}
