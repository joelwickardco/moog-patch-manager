<script>
  import PatchCard from "./PatchCard.svelte";
  import SearchBar from "../common/SearchBar.svelte";

  // Placeholder data - will be replaced with Tauri commands
  let patches = $state([
    { id: 1, name: "Deep Bass", is_favorite: true, categories: ["Bass"], notes: "A warm analog bass" },
    { id: 2, name: "Cathedral Pad", is_favorite: false, categories: ["Pad"], notes: "" },
    { id: 3, name: "Screaming Lead", is_favorite: true, categories: ["Lead"], notes: "High gain lead sound" },
  ]);

  let searchQuery = $state("");
  let viewMode = $state("grid"); // 'grid' or 'list'

  let filteredPatches = $derived(
    patches.filter((p) => p.name.toLowerCase().includes(searchQuery.toLowerCase()))
  );
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
    {#if filteredPatches.length === 0}
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
