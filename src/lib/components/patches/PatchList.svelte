<script>
  import PatchCard from "./PatchCard.svelte";
  import SearchBar from "../common/SearchBar.svelte";
  import { getAllPatches } from "../../utils/api.js";

  let { selectedLibraryId = null } = $props();

  let patches = $state([]);
  let loading = $state(true);
  let error = $state(null);

  let searchQuery = $state("");
  let viewMode = $state("grid"); // 'grid' or 'list'

  let filteredPatches = $derived(
    patches.filter((p) => p.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  async function loadPatches() {
    loading = true;
    error = null;
    try {
      // Patches are now global - selectedLibraryId is used for filtering by source_library if desired
      const filter = selectedLibraryId ? { source_library: null } : null;
      // For now, load all patches (patches are global content store)
      patches = await getAllPatches(null);
    } catch (e) {
      error = e.toString();
      patches = [];
    } finally {
      loading = false;
    }
  }

  // Load patches on mount and when selectedLibraryId changes
  $effect(() => {
    // Track the dependency
    const libraryId = selectedLibraryId;
    loadPatches();
  });
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
          <PatchCard {patch} />
        {/each}
      </div>
    {:else}
      <div class="space-y-2">
        {#each filteredPatches as patch (patch.id)}
          <PatchCard {patch} listView />
        {/each}
      </div>
    {/if}
  </div>
</div>
