<script>
  import { open, save } from "@tauri-apps/plugin-dialog";
  import UpdateChecker from "./lib/components/common/UpdateChecker.svelte";
  import Sidebar from "./lib/components/common/Sidebar.svelte";
  import PatchList from "./lib/components/patches/PatchList.svelte";
  import BankButtonStrip from "./lib/components/banks/BankButtonStrip.svelte";
  import BankDetail from "./lib/components/banks/BankDetail.svelte";
  import NewLibraryModal from "./lib/components/common/NewLibraryModal.svelte";
  import PatchEditorSidebar from "./lib/components/patches/PatchEditorSidebar.svelte";
  import CopyPatchSidebar from "./lib/components/patches/CopyPatchSidebar.svelte";
  import { getAllLibraries, importLibraryZip, importBankDirectory, exportLibrary, createLibrary, getBanksForLibrary, assignPatchToSlot, assignSequenceToSlot } from "./lib/utils/api.js";

  let libraries = $state([]);
  let selectedLibraryId = $state(null);
  let importing = $state(false);
  let exporting = $state(false);
  let statusMessage = $state(null);
  let showNewLibraryModal = $state(false);

  // Bank state
  let banks = $state([]);
  let selectedBankNumber = $state(1);
  let banksLoading = $state(false);

  // Bank patch sidebar state
  let selectedBankPatch = $state(null);
  let selectedBankSlotIndex = $state(null);
  let bankSidebarMode = $state(null); // null | 'edit' | 'copy'
  let selectedBankPatchForCopy = $state(null);

  let selectedLibrary = $derived(
    libraries.find(l => l.id === selectedLibraryId) || null
  );

  let selectedBank = $derived(
    banks.find(b => b.bank_number === selectedBankNumber) || null
  );

  // Load banks when library changes
  let lastLoadedLibraryId = $state(null);
  $effect(() => {
    if (selectedLibraryId) {
      loadBanks(selectedLibraryId);
    } else {
      banks = [];
      selectedBankNumber = 1;
      lastLoadedLibraryId = null;
    }
  });

  // Close sidebar when the user switches to a different bank
  $effect(() => {
    selectedBankNumber;
    handleBankSidebarClose();
  });

  async function loadBanks(libraryId) {
    const isLibraryChange = libraryId !== lastLoadedLibraryId;

    // Only show loading indicator when changing libraries or on initial load
    if (isLibraryChange || banks.length === 0) {
      banksLoading = true;
    } else {
      banksLoading = false;
    }

    if (isLibraryChange) {
      selectedBankNumber = 1;
    }
    lastLoadedLibraryId = libraryId;
    try {
      const result = await getBanksForLibrary(libraryId);
      // Guard against stale responses from rapid library switching
      if (lastLoadedLibraryId === libraryId) {
        banks = result;
      }
    } catch (e) {
      console.error("Failed to load banks:", e);
      if (lastLoadedLibraryId === libraryId) {
        banks = [];
        showStatus("error", `Failed to load banks: ${e}`);
      }
    } finally {
      if (lastLoadedLibraryId === libraryId) {
        banksLoading = false;
      }
    }
  }

  async function loadLibraries() {
    try {
      libraries = await getAllLibraries();
    } catch (e) {
      console.error("Failed to load libraries:", e);
      libraries = [];
    }
  }

  function showStatus(type, text) {
    statusMessage = { type, text };
    setTimeout(() => {
      statusMessage = null;
    }, 5000);
  }

  async function handleImport() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: "ZIP Archives",
          extensions: ["zip"]
        }]
      });

      if (!selected) return;

      importing = true;
      statusMessage = null;

      const result = await importLibraryZip(selected);

      showStatus("success", `Imported "${result.library_name}": ${result.patches_imported} patches, ${result.sequences_imported} sequences`);

      await loadLibraries();
      selectedLibraryId = result.library_id;

    } catch (e) {
      console.error("Import failed:", e);
      showStatus("error", `Import failed: ${e}`);
    } finally {
      importing = false;
    }
  }

  async function handleImportDirectory() {
    try {
      const selected = await open({
        multiple: false,
        directory: true
      });

      if (!selected) return;

      importing = true;
      statusMessage = null;

      const result = await importBankDirectory(selected);

      showStatus("success", `Imported "${result.library_name}": ${result.patches_imported} patches, ${result.sequences_imported} sequences`);

      await loadLibraries();
      selectedLibraryId = result.library_id;

    } catch (e) {
      console.error("Import failed:", e);
      showStatus("error", `Import failed: ${e}`);
    } finally {
      importing = false;
    }
  }

  async function handleExport() {
    if (!selectedLibraryId) {
      showStatus("error", "Please select a library to export");
      return;
    }

    const library = libraries.find(l => l.id === selectedLibraryId);
    const defaultFilename = library ? `${library.name}.zip` : "library.zip";

    try {
      const outputPath = await save({
        defaultPath: defaultFilename,
        filters: [{
          name: "ZIP Archives",
          extensions: ["zip"]
        }]
      });

      if (!outputPath) return;

      exporting = true;
      statusMessage = null;

      const result = await exportLibrary(selectedLibraryId, outputPath);

      showStatus("success", `Exported "${result.library_name}": ${result.patches_exported} patches, ${result.sequences_exported} sequences`);

    } catch (e) {
      console.error("Export failed:", e);
      showStatus("error", `Export failed: ${e}`);
    } finally {
      exporting = false;
    }
  }

  async function handleCreateLibrary(name) {
    try {
      await createLibrary(name);
      showNewLibraryModal = false;
      showStatus("success", `Created library "${name}"`);
      await loadLibraries();
    } catch (e) {
      console.error("Failed to create library:", e);
      showStatus("error", `Failed to create library: ${e}`);
      showNewLibraryModal = false;
    }
  }

  async function handlePatchSlotDrop(slotIndex, patch) {
    if (!selectedLibraryId || selectedBankNumber === null) return;

    try {
      await assignPatchToSlot(selectedLibraryId, selectedBankNumber, slotIndex + 1, patch.id);
      showStatus("success", `Assigned "${patch.name}" to slot ${slotIndex + 1}`);
      await loadBanks(selectedLibraryId);
    } catch (e) {
      console.error("Failed to assign patch:", e);
      showStatus("error", `Failed to assign patch: ${e}`);
    }
  }

  async function handleSequenceSlotDrop(slotIndex, sequence) {
    if (!selectedLibraryId || selectedBankNumber === null) return;

    try {
      await assignSequenceToSlot(selectedLibraryId, selectedBankNumber, slotIndex + 1, sequence.id);
      showStatus("success", `Assigned "${sequence.name}" to slot ${slotIndex + 1}`);
      await loadBanks(selectedLibraryId);
    } catch (e) {
      console.error("Failed to assign sequence:", e);
      showStatus("error", `Failed to assign sequence: ${e}`);
    }
  }

  function handlePatchSlotClick(slotIndex, patch) {
    if (patch) {
      selectedBankPatch = patch;
      selectedBankSlotIndex = slotIndex;
      bankSidebarMode = 'edit';
      selectedBankPatchForCopy = null;
    }
  }

  function handleBankCopyClick(patch) {
    selectedBankPatch = patch;
    selectedBankPatchForCopy = patch;
    bankSidebarMode = 'copy';
  }

  function handleBankSidebarClose() {
    selectedBankPatch = null;
    selectedBankSlotIndex = null;
    bankSidebarMode = null;
    selectedBankPatchForCopy = null;
  }

  async function handleBankPatchDeleted() {
    selectedBankPatch = null;
    selectedBankSlotIndex = null;
    bankSidebarMode = null;
    selectedBankPatchForCopy = null;
    await loadBanks(selectedLibraryId);
  }

  async function handleBankPatchSaved() {
    await loadBanks(selectedLibraryId);
  }

  async function handleBankPatchRemovedFromSlot() {
    selectedBankPatch = null;
    selectedBankSlotIndex = null;
    bankSidebarMode = null;
    selectedBankPatchForCopy = null;
    await loadBanks(selectedLibraryId);
  }

  async function handleBankCopySubmit(libraryId, bankNumber, slotNumber) {
    await assignPatchToSlot(libraryId, bankNumber, slotNumber, selectedBankPatchForCopy.id);
    const patchName = selectedBankPatchForCopy.name;
    selectedBankPatch = null;
    selectedBankPatchForCopy = null;
    bankSidebarMode = null;
    showStatus("success", `Copied "${patchName}" to Bank ${bankNumber}, Slot ${slotNumber}`);
    await loadBanks(selectedLibraryId);
  }

  function handleSequenceSlotClick(slotIndex, sequence) {
    console.log("Sequence slot clicked:", slotIndex, sequence);
  }

  async function handleBankNameUpdate() {
    await loadBanks(selectedLibraryId);
    showStatus("success", "Bank name updated");
  }

  // Load libraries on mount
  $effect(() => {
    loadLibraries();
  });
</script>

<div class="flex h-screen bg-background text-text-primary">
  <Sidebar
    {libraries}
    bind:selectedLibraryId
    onImport={handleImport}
    onImportDirectory={handleImportDirectory}
    {importing}
    onExport={handleExport}
    {exporting}
    onNewLibrary={() => showNewLibraryModal = true}
    onLibraryNameUpdate={loadLibraries}
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

    {#if selectedLibraryId}
      <BankButtonStrip
        {banks}
        bind:selectedBankNumber
        loading={banksLoading}
        libraryName={selectedLibrary?.name || ''}
      />

      <div class="flex-1 overflow-hidden flex">
        <div class="flex-1 overflow-hidden">
          <BankDetail
            bank={selectedBank}
            libraryId={selectedLibraryId}
            selectedPatchId={selectedBankPatch?.id || null}
            onPatchSlotClick={handlePatchSlotClick}
            onSequenceSlotClick={handleSequenceSlotClick}
            onPatchSlotDrop={handlePatchSlotDrop}
            onSequenceSlotDrop={handleSequenceSlotDrop}
            onBankNameUpdate={handleBankNameUpdate}
            onCopyPatch={handleBankCopyClick}
          />
        </div>
        <div class="transition-all duration-200 ease-out overflow-hidden {bankSidebarMode ? 'w-[360px]' : 'w-0'}">
          {#if bankSidebarMode === 'edit'}
            <PatchEditorSidebar
              patch={selectedBankPatch}
              onClose={handleBankSidebarClose}
              onSaved={handleBankPatchSaved}
              onDeleted={handleBankPatchDeleted}
              bankContext={selectedBankPatch ? { libraryId: selectedLibraryId, bankNumber: selectedBankNumber, slotNumber: selectedBankSlotIndex + 1 } : null}
              onRemovedFromSlot={handleBankPatchRemovedFromSlot}
            />
          {:else if bankSidebarMode === 'copy'}
            <CopyPatchSidebar
              patch={selectedBankPatchForCopy}
              {libraries}
              onClose={handleBankSidebarClose}
              onSubmit={handleBankCopySubmit}
            />
          {/if}
        </div>
      </div>
    {:else}
      <div class="flex-1 overflow-hidden">
        <PatchList
          {selectedLibraryId}
          onLibrariesChanged={loadLibraries}
          onImportZip={handleImport}
          onImportDirectory={handleImportDirectory}
          onCreateLibrary={() => showNewLibraryModal = true}
          {importing}
        />
      </div>
    {/if}
  </main>
</div>

<NewLibraryModal
  open={showNewLibraryModal}
  onClose={() => showNewLibraryModal = false}
  onSubmit={handleCreateLibrary}
/>

<UpdateChecker />
