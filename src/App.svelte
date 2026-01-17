<script>
  import { open } from "@tauri-apps/plugin-dialog";
  import Sidebar from "./lib/components/common/Sidebar.svelte";
  import PatchList from "./lib/components/patches/PatchList.svelte";
  import { getAllLibraries, importLibraryZip } from "./lib/utils/api.js";

  let activeTab = $state("library");
  let libraries = $state([]);
  let selectedLibraryId = $state(null);
  let importing = $state(false);
  let importMessage = $state(null);

  async function loadLibraries() {
    try {
      libraries = await getAllLibraries();
    } catch (e) {
      console.error("Failed to load libraries:", e);
      libraries = [];
    }
  }

  async function handleImport() {
    console.log("Import button clicked");
    try {
      // Open file picker for ZIP files
      console.log("Opening file dialog...");
      const selected = await open({
        multiple: false,
        filters: [{
          name: "ZIP Archives",
          extensions: ["zip"]
        }]
      });
      console.log("File selected:", selected);

      if (!selected) {
        // User cancelled
        console.log("User cancelled file selection");
        return;
      }

      importing = true;
      importMessage = null;

      // Import the library
      const result = await importLibraryZip(selected);

      // Show success message
      importMessage = {
        type: "success",
        text: `Imported "${result.library_name}": ${result.patches_imported} patches, ${result.sequences_imported} sequences`
      };

      // Reload libraries to show the new one
      await loadLibraries();

      // Select the newly imported library
      selectedLibraryId = result.library_id;
      activeTab = "library";

    } catch (e) {
      console.error("Import failed:", e);
      importMessage = {
        type: "error",
        text: `Import failed: ${e}`
      };
    } finally {
      importing = false;

      // Clear message after 5 seconds
      setTimeout(() => {
        importMessage = null;
      }, 5000);
    }
  }

  // Load libraries on mount
  $effect(() => {
    loadLibraries();
  });
</script>

<div class="flex h-screen bg-background text-text-primary">
  <Sidebar
    bind:activeTab
    {libraries}
    bind:selectedLibraryId
    onImport={handleImport}
    {importing}
  />

  <main class="flex-1 overflow-hidden flex flex-col">
    {#if importMessage}
      <div
        class="mx-4 mt-4 p-3 rounded-lg {importMessage.type === 'success'
          ? 'bg-green-500/20 text-green-400 border border-green-500/30'
          : 'bg-red-500/20 text-red-400 border border-red-500/30'}"
      >
        {importMessage.text}
      </div>
    {/if}

    <div class="flex-1 overflow-hidden">
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
    </div>
  </main>
</div>
