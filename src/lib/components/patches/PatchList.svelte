<script>
  import PatchCard from "./PatchCard.svelte";
  import SearchBar from "../common/SearchBar.svelte";
  import TagInput from "./TagInput.svelte";
  import CopyPatchModal from "./CopyPatchModal.svelte";
  import PatchEditModal from "./PatchEditModal.svelte";
  import { getAllPatches, getPatchesForLibrary, getAllLibraries, assignPatchToSlot, getAllTags } from "../../utils/api.js";

  let { selectedLibraryId = null, onLibrariesChanged = () => {} } = $props();

  let patches = $state([]);
  let loading = $state(true);
  let error = $state(null);

  let searchQuery = $state("");
  let viewMode = $state("grid"); // 'grid' or 'list'

  // Tag filtering state
  let availableTags = $state([]);
  let selectedFilterTags = $state([]);
  let tagFilterMode = $state("any"); // 'any' or 'all'
  let showFavoritesOnly = $state(false);

  // Copy modal state
  let showCopyModal = $state(false);
  let selectedPatchForCopy = $state(null);
  let libraries = $state([]);
  let statusMessage = $state(null);

  // Edit modal state
  let showEditModal = $state(false);
  let selectedPatchForEdit = $state(null);

  let filteredPatches = $derived(
    patches
      .filter((p) => p.name.toLowerCase().includes(searchQuery.toLowerCase()))
      .filter((p) => !showFavoritesOnly || p.is_favorite)
  );

  async function loadPatches() {
    loading = true;
    error = null;
    try {
      if (selectedLibraryId) {
        // Show only patches assigned to this library's bank slots
        patches = await getPatchesForLibrary(selectedLibraryId);
      } else {
        // Build filter for getAllPatches
        const filter = {};
        if (selectedFilterTags.length > 0) {
          filter.tags = selectedFilterTags;
          filter.require_all_tags = tagFilterMode === "all";
        }
        if (showFavoritesOnly) {
          filter.is_favorite = true;
        }
        patches = await getAllPatches(Object.keys(filter).length > 0 ? filter : null);
      }
    } catch (e) {
      error = e.toString();
      patches = [];
    } finally {
      loading = false;
    }
  }

  async function loadTags() {
    try {
      const tagDtos = await getAllTags();
      availableTags = tagDtos.map(t => t.name);
    } catch (e) {
      console.error("Failed to load tags:", e);
      availableTags = [];
    }
  }

  // Load patches on mount and when selectedLibraryId or tag filter changes
  $effect(() => {
    // Track dependencies
    void selectedLibraryId;
    void selectedFilterTags;
    void tagFilterMode;
    void showFavoritesOnly;
    loadPatches();
  });

  // Load libraries on mount
  $effect(() => {
    loadLibraries();
  });

  // Load tags on mount
  $effect(() => {
    loadTags();
  });

  async function loadLibraries() {
    try {
      libraries = await getAllLibraries();
    } catch (e) {
      console.error("Failed to load libraries:", e);
      libraries = [];
    }
  }

  function handleCopyClick(patch) {
    selectedPatchForCopy = patch;
    showCopyModal = true;
  }

  function handleEditClick(patch) {
    selectedPatchForEdit = patch;
    showEditModal = true;
  }

  function handleEditSaved() {
    // Reload tags list and patches to reflect changes
    loadTags();
    loadPatches();
  }

  async function handleCopySubmit(libraryId, bankNumber, slotNumber) {
    try {
      await assignPatchToSlot(libraryId, bankNumber, slotNumber, selectedPatchForCopy.id);

      // Success
      showCopyModal = false;
      const patchName = selectedPatchForCopy.name;
      selectedPatchForCopy = null;

      statusMessage = {
        type: "success",
        text: `Copied "${patchName}" to Bank ${bankNumber}, Slot ${slotNumber}`
      };

      // Auto-dismiss after 5 seconds
      setTimeout(() => { statusMessage = null; }, 5000);

      // Refresh libraries to update patch counts
      onLibrariesChanged();

    } catch (e) {
      console.error("Failed to copy patch:", e);
      throw e; // Let modal display the error
    }
  }
</script>

<div class="h-full flex flex-col">
  <header class="p-4 border-b border-border">
    <div class="flex items-center justify-between mb-4">
      <h1 class="text-2xl font-semibold">Patch Library</h1>
      <div class="flex gap-2">
        <button
          class="px-3 py-1 rounded {viewMode === 'grid' ? 'bg-primary text-white' : 'bg-surface'}"
          onclick={() => (viewMode = "grid")}
        >
          Grid
        </button>
        <button
          class="px-3 py-1 rounded {viewMode === 'list' ? 'bg-primary text-white' : 'bg-surface'}"
          onclick={() => (viewMode = "list")}
        >
          List
        </button>
      </div>
    </div>
    <SearchBar bind:value={searchQuery} placeholder="Search patches..." />

    <!-- Favorites Filter -->
    <div class="mt-3 flex items-center gap-2">
      <button
        class="flex items-center gap-1 px-3 py-2 rounded text-sm {showFavoritesOnly ? 'bg-favorite text-white' : 'bg-surface'}"
        onclick={() => (showFavoritesOnly = !showFavoritesOnly)}
        title={showFavoritesOnly ? "Show all patches" : "Show favorites only"}
      >
        <span>{showFavoritesOnly ? '★' : '☆'}</span>
        <span>{showFavoritesOnly ? 'Favorites Only' : 'Show All'}</span>
      </button>
      {#if showFavoritesOnly}
        <span class="text-xs text-text-secondary">
          Showing {filteredPatches.length} favorite{filteredPatches.length !== 1 ? 's' : ''}
        </span>
      {/if}
    </div>

    <!-- Tag Filtering -->
    {#if !selectedLibraryId}
      <div class="mt-3">
        <div class="flex items-center gap-2 mb-2">
          <span class="text-sm text-text-secondary">Filter by tags:</span>
          <div class="flex gap-1">
            <button
              class="text-xs px-2 py-1 rounded {tagFilterMode === 'any' ? 'bg-primary text-white' : 'bg-surface'}"
              onclick={() => (tagFilterMode = "any")}
              title="Show patches with ANY of the selected tags"
            >
              Any
            </button>
            <button
              class="text-xs px-2 py-1 rounded {tagFilterMode === 'all' ? 'bg-primary text-white' : 'bg-surface'}"
              onclick={() => (tagFilterMode = "all")}
              title="Show patches with ALL of the selected tags"
            >
              All
            </button>
          </div>
        </div>
        <TagInput
          bind:selectedTags={selectedFilterTags}
          {availableTags}
          placeholder="Select tags to filter..."
        />
      </div>
    {/if}
  </header>

  <div class="flex-1 overflow-auto p-4">
    {#if loading}
      <div class="text-center text-text-secondary py-12">
        <p class="text-lg">Loading patches...</p>
      </div>
    {:else if error}
      <div class="text-center text-red-500 py-12">
        <p class="text-lg">Error loading patches</p>
        <p class="text-sm mt-2">{error}</p>
      </div>
    {:else if filteredPatches.length === 0}
      <div class="text-center text-text-secondary py-12">
        <p class="text-lg">No patches found</p>
        <p class="text-sm mt-2">Import patches to get started</p>
      </div>
    {:else if viewMode === "grid"}
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
        {#each filteredPatches as patch (patch.id)}
          <PatchCard {patch} onCopy={handleCopyClick} onEdit={handleEditClick} />
        {/each}
      </div>
    {:else}
      <div class="space-y-2">
        {#each filteredPatches as patch (patch.id)}
          <PatchCard {patch} listView onCopy={handleCopyClick} onEdit={handleEditClick} />
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Copy Patch Modal -->
<CopyPatchModal
  open={showCopyModal}
  patch={selectedPatchForCopy}
  {libraries}
  onClose={() => { showCopyModal = false; selectedPatchForCopy = null; }}
  onSubmit={handleCopySubmit}
/>

<!-- Edit Patch Modal -->
<PatchEditModal
  open={showEditModal}
  patch={selectedPatchForEdit}
  onClose={() => { showEditModal = false; selectedPatchForEdit = null; }}
  onSaved={handleEditSaved}
/>

<!-- Status Message -->
{#if statusMessage}
  <div class="fixed bottom-4 right-4 px-4 py-2 rounded-lg
              {statusMessage.type === 'success' ? 'bg-green-500' : 'bg-red-500'}
              text-white shadow-lg z-50">
    {statusMessage.text}
  </div>
{/if}
