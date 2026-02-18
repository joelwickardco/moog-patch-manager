<script>
  import PatchCard from "./PatchCard.svelte";
  import SearchBar from "../common/SearchBar.svelte";
  import TagInput from "./TagInput.svelte";
  import CopyPatchSidebar from "./CopyPatchSidebar.svelte";
  import PatchEditorSidebar from "./PatchEditorSidebar.svelte";
  import { getAllPatches, getPatchesForLibrary, getAllLibraries, assignPatchToSlot, getAllTags } from "../../utils/api.js";

  let {
    selectedLibraryId = null,
    onLibrariesChanged = () => {},
    // New props for empty state:
    onImportZip = () => {},
    onImportDirectory = () => {},
    onCreateLibrary = () => {},
    importing = false
  } = $props();

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

  // Copy sidebar state
  let selectedPatchForCopy = $state(null);
  let libraries = $state([]);
  let statusMessage = $state(null);

  // Sidebar state — null | 'edit' | 'copy'
  let sidebarMode = $state(null);
  let selectedPatchId = $state(null);

  let selectedPatchObj = $derived(
    selectedPatchId ? patches.find(p => p.id === selectedPatchId) || null : null
  );

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
    selectedPatchId = patch.id;
    sidebarMode = 'copy';
  }

  function handlePatchSelect(patch) {
    selectedPatchId = patch.id;
    selectedPatchForCopy = null;
    sidebarMode = 'edit';
  }

  function handleSidebarClose() {
    selectedPatchId = null;
    sidebarMode = null;
    selectedPatchForCopy = null;
  }

  function handlePatchDeleted() {
    selectedPatchId = null;
    sidebarMode = null;
    selectedPatchForCopy = null;
    loadPatches();
    loadTags();
  }

  function handleEditSaved() {
    loadTags();
    loadPatches();
  }

  async function handleCopySubmit(libraryId, bankNumber, slotNumber) {
    try {
      await assignPatchToSlot(libraryId, bankNumber, slotNumber, selectedPatchForCopy.id);

      // Success
      const patchName = selectedPatchForCopy.name;
      sidebarMode = null;
      selectedPatchForCopy = null;
      selectedPatchId = null;

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

  <div class="flex-1 overflow-hidden flex">
    <!-- Patch grid area -->
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
        {#if patches && patches.length === 0}
          <div class="flex items-center justify-center h-full">
            <div class="max-w-2xl mx-auto px-8 py-16 text-center">
              <!-- Welcome Icon -->
              <div class="text-6xl mb-6">👋</div>

              <!-- Heading -->
              <h1 class="text-2xl md:text-3xl font-bold text-text-primary mb-4">
                Welcome to Moog Muse Patch Manager
              </h1>

              <!-- Subtext -->
              <p class="text-base text-text-secondary leading-relaxed mb-8">
                Your patch library is empty. Let's get started organizing your Moog Muse sounds.
              </p>

              <!-- Divider -->
              <div class="w-24 h-px bg-border mx-auto mb-8"></div>

              <!-- Getting Started Steps -->
              <div class="mb-8">
                <h2 class="text-lg font-semibold text-text-primary mb-4">
                  Getting Started:
                </h2>
                <ol class="text-base text-text-secondary leading-relaxed space-y-2 list-decimal list-inside max-w-md mx-auto text-left">
                  <li>Import a library (ZIP file or directory)</li>
                  <li>Browse and organize your patches</li>
                  <li>Create banks with 16 patches each</li>
                  <li>Export to USB for your Moog Muse</li>
                </ol>
              </div>

              <!-- Action Buttons -->
              <div class="flex flex-col sm:flex-row justify-center items-center gap-3 sm:gap-4 mb-6">
                <button
                  class="w-full sm:w-auto px-6 py-3 bg-primary text-white font-medium rounded-lg
                         hover:bg-primary/90 transition-colors shadow-md hover:shadow-lg
                         disabled:opacity-50 disabled:cursor-not-allowed"
                  onclick={onImportZip}
                  disabled={importing}
                >
                  {importing ? 'Importing...' : 'Import ZIP'}
                </button>

                <button
                  class="w-full sm:w-auto px-6 py-3 bg-surface text-text-primary font-medium rounded-lg
                         hover:bg-border transition-colors border border-border
                         disabled:opacity-50 disabled:cursor-not-allowed"
                  onclick={onImportDirectory}
                  disabled={importing}
                >
                  Import Directory
                </button>

                <button
                  class="w-full sm:w-auto px-6 py-3 bg-surface text-text-primary font-medium rounded-lg
                         hover:bg-border transition-colors border border-border"
                  onclick={onCreateLibrary}
                >
                  Create Empty Library
                </button>
              </div>
            </div>
          </div>
        {:else}
          <div class="flex items-center justify-center h-full">
            <div class="max-w-xl mx-auto px-6 py-12 text-center">
              <h2 class="text-xl md:text-2xl font-semibold text-text-primary mb-3">
                No patches match your current search or filters
              </h2>
              <p class="text-base text-text-secondary leading-relaxed mb-6">
                Try clearing your search, turning off favorites-only, or adjusting your tag filters to see more patches.
              </p>
            </div>
          </div>
        {/if}
      {:else if viewMode === "grid"}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
          {#each filteredPatches as patch (patch.id)}
            <PatchCard
              {patch}
              isSelected={selectedPatchId === patch.id}
              onSelect={handlePatchSelect}
              onCopy={handleCopyClick}
            />
          {/each}
        </div>
      {:else}
        <div class="space-y-2">
          {#each filteredPatches as patch (patch.id)}
            <PatchCard
              {patch}
              listView
              isSelected={selectedPatchId === patch.id}
              onSelect={handlePatchSelect}
              onCopy={handleCopyClick}
            />
          {/each}
        </div>
      {/if}
    </div>

    <!-- Sidebar wrapper with slide animation -->
    <div
      class="transition-all duration-200 ease-out overflow-hidden {sidebarMode ? 'w-[360px]' : 'w-0'}"
    >
      {#if sidebarMode === 'edit'}
        <PatchEditorSidebar
          patch={selectedPatchObj}
          onClose={handleSidebarClose}
          onSaved={handleEditSaved}
          onDeleted={handlePatchDeleted}
        />
      {:else if sidebarMode === 'copy'}
        <CopyPatchSidebar
          patch={selectedPatchForCopy}
          {libraries}
          onClose={handleSidebarClose}
          onSubmit={handleCopySubmit}
        />
      {/if}
    </div>
  </div>
</div>

<!-- Status Message -->
{#if statusMessage}
  <div class="fixed bottom-4 right-4 px-4 py-2 rounded-lg
              {statusMessage.type === 'success' ? 'bg-green-500' : 'bg-red-500'}
              text-white shadow-lg z-50">
    {statusMessage.text}
  </div>
{/if}
