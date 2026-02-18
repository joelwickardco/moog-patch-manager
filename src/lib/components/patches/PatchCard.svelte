<script>
  import { toggleFavorite as toggleFavoriteApi } from "../../utils/api.js";
  import {
    isNewPatch,
    isModifiedPatch,
    formatUsageCount,
    generatePatchStateAriaLabel
  } from "../../utils/patchStates.js";

  let {
    patch,
    listView = false,
    isSelected = false,
    onCopy = () => {},
    onSelect = () => {}
  } = $props();

  // Derive patch states once for performance
  let isNew = $derived(isNewPatch(patch.created_at));
  let isModified = $derived(isModifiedPatch(patch.created_at, patch.updated_at));
  let isUntagged = $derived(!patch.tags || patch.tags.length === 0);
  let hasMultipleUses = $derived(patch.usage_count && patch.usage_count > 1);

  // Container classes based on view mode, favorite status, and selection
  let containerClasses = $derived(
    listView
      ? getListViewClasses(patch.is_favorite, isSelected)
      : getGridViewClasses(patch.is_favorite, isSelected)
  );

  function getGridViewClasses(isFavorite, selected) {
    const base = "bg-surface rounded-lg p-4 hover:bg-border/50 transition-colors cursor-pointer relative";

    if (selected && isFavorite) {
      return `${base} border-2 border-favorite ring-2 ring-favorite/30 shadow-glow-gold shadow-lg`;
    }
    if (selected) {
      return `${base} border-2 border-primary ring-2 ring-primary/30 shadow-lg`;
    }
    if (isFavorite) {
      return `${base} border-2 border-favorite shadow-glow-gold hover:bg-favorite/5 hover:shadow-glow-gold-hover`;
    }
    return `${base} border-2 border-transparent`;
  }

  function getListViewClasses(isFavorite, selected) {
    const base = "flex items-center gap-4 p-4 bg-surface rounded-lg hover:bg-border/50 transition-colors cursor-pointer relative";

    if (selected && isFavorite) {
      return `${base} border-2 border-favorite bg-favorite/5 shadow-glow-gold`;
    }
    if (selected) {
      return `${base} border-2 border-primary bg-primary/5`;
    }
    if (isFavorite) {
      return `${base} border-2 border-favorite shadow-glow-gold hover:bg-favorite/5`;
    }
    return `${base} border-2 border-transparent`;
  }

  async function toggleFavorite(e) {
    e.stopPropagation();
    try {
      const newValue = await toggleFavoriteApi(patch.id);
      patch.is_favorite = newValue;
    } catch (e) {
      console.error("Failed to toggle favorite:", e);
    }
  }

  function handleCopy(e) {
    e.stopPropagation();
    onCopy(patch);
  }

  function handleCardClick() {
    onSelect(patch);
  }

  function handleCardKeydown(e) {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      onSelect(patch);
    }
  }

  function handleDragStart(e) {
    e.dataTransfer.setData("application/json", JSON.stringify({
      ...patch,
      sourceType: "library"
    }));
    e.dataTransfer.effectAllowed = "copyMove";
  }
</script>

{#if listView}
  <div
    class={containerClasses}
    tabindex="0"
    aria-label={generatePatchStateAriaLabel(patch)}
    aria-selected={isSelected}
    draggable="true"
    ondragstart={handleDragStart}
    onclick={handleCardClick}
    onkeydown={handleCardKeydown}
  >
    <button
      class="text-2xl {patch.is_favorite ? 'text-favorite' : 'text-text-secondary hover:text-favorite'}"
      onclick={toggleFavorite}
      aria-label={patch.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
    >
      {patch.is_favorite ? "★" : "☆"}
    </button>
    <div class="flex-1 min-w-0">
      <h3 class="font-medium truncate" title={patch.name}>
        {#if isModified}
          <span
            class="w-1.5 h-1.5 rounded-full bg-orange-500 inline-block mr-1.5"
            title="Modified since import"
            aria-label="Modified"
          ></span>
        {/if}
        {patch.name}
      </h3>
      <div class="flex items-center gap-2 mt-1 flex-wrap">
        {#if patch.source_library}
          <span class="text-xs px-2 py-0.5 rounded bg-secondary/20 text-secondary">
            {patch.source_library}
          </span>
        {/if}
        {#if isNew}
          <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-green-500 text-white" aria-label="Recently imported">
            New
          </span>
        {/if}
        {#if isUntagged}
          <span class="px-2 py-0.5 text-xs rounded bg-gray-600/50 text-gray-300" aria-label="Patch has no tags">
            Untagged
          </span>
        {/if}
        {#if hasMultipleUses}
          <span class="px-2 py-0.5 text-xs font-semibold rounded bg-blue-500 text-white" aria-label="Used in {patch.usage_count} banks">
            {formatUsageCount(patch.usage_count)}
          </span>
        {/if}
        {#if patch.tags && patch.tags.length > 0}
          {#each patch.tags as tag}
            <span class="text-xs px-2 py-0.5 rounded bg-primary/20 text-primary">
              {tag}
            </span>
          {/each}
        {/if}
      </div>
    </div>
    <div class="flex gap-2 flex-shrink-0">
      <button onclick={handleCopy} class="p-2 hover:bg-surface rounded" title="Copy to library">
        <span class="text-text-secondary">📋</span>
      </button>
    </div>
  </div>
{:else}
  <div
    class={containerClasses}
    role="option"
    tabindex="0"
    aria-label={generatePatchStateAriaLabel(patch)}
    aria-selected={isSelected}
    draggable="true"
    ondragstart={handleDragStart}
    onclick={handleCardClick}
    onkeydown={handleCardKeydown}
  >
    {#if isNew}
      <span
        class="absolute top-2 left-2 px-2 py-0.5 text-xs font-medium rounded-full bg-green-500 text-white shadow-md z-10"
        aria-label="Recently imported"
      >
        New
      </span>
    {/if}

    {#if isUntagged}
      <span
        class="absolute top-2 right-2 px-2 py-0.5 text-xs rounded-md bg-gray-600/80 text-gray-300 backdrop-blur-sm border border-gray-500/50 z-10"
        aria-label="Patch has no tags"
      >
        Untagged
      </span>
    {/if}

    <div class="flex items-start justify-between mb-2">
      <h3 class="font-medium truncate flex-1" title={patch.name}>
        {#if isModified}
          <span
            class="w-1.5 h-1.5 rounded-full bg-orange-500 inline-block mr-1.5"
            title="Modified since import"
            aria-label="Modified"
          ></span>
        {/if}
        {patch.name}
      </h3>
      <div class="flex gap-1 ml-2">
        <button
          class="text-xl hover:text-primary transition-colors"
          onclick={handleCopy}
          title="Copy to library"
        >
          📋
        </button>
        <button
          class="text-xl {patch.is_favorite ? 'text-favorite' : 'text-text-secondary hover:text-favorite'}"
          onclick={toggleFavorite}
          aria-label={patch.is_favorite ? 'Remove from favorites' : 'Add to favorites'}
        >
          {patch.is_favorite ? "★" : "☆"}
        </button>
      </div>
    </div>

    {#if patch.source_library}
      <div class="mb-2">
        <span class="text-xs px-2 py-0.5 rounded bg-secondary/20 text-secondary">
          {patch.source_library}
        </span>
      </div>
    {/if}

    {#if patch.tags && patch.tags.length > 0}
      <div class="flex flex-wrap gap-1 mb-2">
        {#each patch.tags as tag}
          <span class="text-xs px-2 py-0.5 rounded bg-primary/20 text-primary">
            {tag}
          </span>
        {/each}
      </div>
    {/if}

    {#if patch.notes}
      <p class="text-sm text-text-secondary line-clamp-2">{patch.notes}</p>
    {/if}

    {#if hasMultipleUses}
      <span
        class="absolute bottom-2 right-2 px-1.5 py-0.5 text-xs font-semibold rounded bg-blue-500 text-white shadow-md z-10"
        aria-label="Used in {patch.usage_count} banks"
      >
        {formatUsageCount(patch.usage_count)}
      </span>
    {/if}

  </div>
{/if}
