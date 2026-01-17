<script>
  import Sidebar from "./lib/components/common/Sidebar.svelte";
  import PatchList from "./lib/components/patches/PatchList.svelte";
  import { getAllLibraries } from "./lib/utils/api.js";

  let activeTab = $state("library");
  let libraries = $state([]);
  let selectedLibraryId = $state(null);

  async function loadLibraries() {
    try {
      libraries = await getAllLibraries();
    } catch (e) {
      console.error("Failed to load libraries:", e);
      libraries = [];
    }
  }

  // Load libraries on mount
  $effect(() => {
    loadLibraries();
  });
</script>

<div class="flex h-screen bg-background text-text-primary">
  <Sidebar bind:activeTab {libraries} bind:selectedLibraryId />

  <main class="flex-1 overflow-hidden">
    {#if activeTab === "library"}
      <PatchList {selectedLibraryId} />
    {:else if activeTab === "banks"}
      <div class="p-6">
        <h1 class="text-2xl font-semibold">Banks</h1>
        <p class="text-text-secondary mt-2">Bank management coming soon...</p>
      </div>
    {:else if activeTab === "categories"}
      <div class="p-6">
        <h1 class="text-2xl font-semibold">Categories</h1>
        <p class="text-text-secondary mt-2">Category management coming soon...</p>
      </div>
    {/if}
  </main>
</div>
