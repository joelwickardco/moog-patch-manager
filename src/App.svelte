<script>
  import { open, save } from "@tauri-apps/plugin-dialog";
  import Sidebar from "./lib/components/common/Sidebar.svelte";
  import PatchList from "./lib/components/patches/PatchList.svelte";
  import BanksView from "./lib/components/banks/BanksView.svelte";
  import NewLibraryModal from "./lib/components/common/NewLibraryModal.svelte";
  import { getAllLibraries, importLibraryZip, exportLibrary, createLibrary } from "./lib/utils/api.js";

  let activeTab = $state("library");
  let libraries = $state([]);
  let selectedLibraryId = $state(null);
  let importing = $state(false);
  let exporting = $state(false);
  let statusMessage = $state(null);
  let showNewLibraryModal = $state(false);

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
      statusMessage = null;

      // Import the library
      const result = await importLibraryZip(selected);

      // Show success message
      statusMessage = {
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
      statusMessage = {
        type: "error",
        text: `Import failed: ${e}`
      };
    } finally {
      importing = false;

      // Clear message after 5 seconds
      setTimeout(() => {
        statusMessage = null;
      }, 5000);
    }
  }

  async function handleExport() {
    if (!selectedLibraryId) {
      statusMessage = {
        type: "error",
        text: "Please select a library to export"
      };
      return;
    }

    // Get the library name for the default filename
    const library = libraries.find(l => l.id === selectedLibraryId);
    const defaultFilename = library ? `${library.name}.zip` : "library.zip";

    try {
      // Open save dialog for ZIP file location
      const outputPath = await save({
        defaultPath: defaultFilename,
        filters: [{
          name: "ZIP Archives",
          extensions: ["zip"]
        }]
      });

      if (!outputPath) {
        // User cancelled
        return;
      }

      exporting = true;
      statusMessage = null;

      // Export the library
      const result = await exportLibrary(selectedLibraryId, outputPath);

      // Show success message
      statusMessage = {
        type: "success",
        text: `Exported "${result.library_name}": ${result.patches_exported} patches, ${result.sequences_exported} sequences`
      };

    } catch (e) {
      console.error("Export failed:", e);
      statusMessage = {
        type: "error",
        text: `Export failed: ${e}`
      };
    } finally {
      exporting = false;

      // Clear message after 5 seconds
      setTimeout(() => {
        statusMessage = null;
      }, 5000);
    }
  }

  async function handleCreateLibrary(name) {
    try {
      await createLibrary(name);

      // Close modal
      showNewLibraryModal = false;

      // Show success message
      statusMessage = {
        type: "success",
        text: `Created library "${name}"`
      };

      // Reload libraries
      await loadLibraries();

      // Clear message after 5 seconds
      setTimeout(() => {
        statusMessage = null;
      }, 5000);

    } catch (e) {
      console.error("Failed to create library:", e);
      statusMessage = {
        type: "error",
        text: `Failed to create library: ${e}`
      };

      // Close modal on error too
      showNewLibraryModal = false;

      // Clear message after 5 seconds
      setTimeout(() => {
        statusMessage = null;
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
    onExport={handleExport}
    {exporting}
    onNewLibrary={() => showNewLibraryModal = true}
  />

  <main class="flex-1 overflow-hidden flex flex-col">
    {#if statusMessage}
      <div
        class="mx-4 mt-4 p-3 rounded-lg {statusMessage.type === 'success'
          ? 'bg-green-500/20 text-green-400 border border-green-500/30'
          : 'bg-red-500/20 text-red-400 border border-red-500/30'}"
      >
        {statusMessage.text}
      </div>
    {/if}

    <div class="flex-1 overflow-hidden">
      {#if activeTab === "library"}
        <PatchList {selectedLibraryId} onLibrariesChanged={loadLibraries} />
      {:else if activeTab === "banks"}
        <BanksView {selectedLibraryId} {libraries} />
      {:else if activeTab === "categories"}
        <div class="p-6">
          <h1 class="text-2xl font-semibold">Categories</h1>
          <p class="text-text-secondary mt-2">Category management coming soon...</p>
        </div>
      {/if}
    </div>
  </main>
</div>

<NewLibraryModal
  open={showNewLibraryModal}
  onClose={() => showNewLibraryModal = false}
  onSubmit={handleCreateLibrary}
/>
