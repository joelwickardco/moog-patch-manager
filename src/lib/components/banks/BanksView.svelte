<script>
  import BankList from "./BankList.svelte";
  import BankDetail from "./BankDetail.svelte";
  import { getBanksForLibrary, assignPatchToBank, assignSequenceToBank } from "../../utils/api.js";

  let {
    selectedLibraryId = null,
    libraries = []
  } = $props();

  let banks = $state([]);
  let selectedBankNumber = $state(null);
  let loading = $state(false);
  let statusMessage = $state(null);

  let selectedBank = $derived(
    banks.find(b => b.bank_number === selectedBankNumber) || null
  );

  let selectedLibrary = $derived(
    libraries.find(l => l.id === selectedLibraryId) || null
  );

  // Load banks when library changes
  $effect(() => {
    if (selectedLibraryId) {
      loadBanks();
    } else {
      banks = [];
      selectedBankNumber = null;
    }
  });

  async function loadBanks() {
    loading = true;
    try {
      banks = await getBanksForLibrary(selectedLibraryId);
      // Reset selection if current bank doesn't exist
      if (selectedBankNumber !== null && !banks.find(b => b.bank_number === selectedBankNumber)) {
        selectedBankNumber = null;
      }
    } catch (e) {
      console.error("Failed to load banks:", e);
      banks = [];
      showStatus("error", `Failed to load banks: ${e}`);
    } finally {
      loading = false;
    }
  }

  function showStatus(type, text) {
    statusMessage = { type, text };
    setTimeout(() => {
      statusMessage = null;
    }, 5000);
  }

  async function handlePatchSlotDrop(slotIndex, patch) {
    if (!selectedLibraryId || selectedBankNumber === null) return;

    try {
      // Assign the patch to this slot
      await assignPatchToBank(selectedLibraryId, selectedBankNumber, slotIndex + 1, patch.id);
      showStatus("success", `Assigned "${patch.name}" to slot ${slotIndex + 1}`);
      await loadBanks();
    } catch (e) {
      console.error("Failed to assign patch:", e);
      showStatus("error", `Failed to assign patch: ${e}`);
    }
  }

  async function handleSequenceSlotDrop(slotIndex, sequence) {
    if (!selectedLibraryId || selectedBankNumber === null) return;

    try {
      await assignSequenceToBank(selectedLibraryId, selectedBankNumber, slotIndex + 1, sequence.id);
      showStatus("success", `Assigned "${sequence.name}" to slot ${slotIndex + 1}`);
      await loadBanks();
    } catch (e) {
      console.error("Failed to assign sequence:", e);
      showStatus("error", `Failed to assign sequence: ${e}`);
    }
  }

  function handlePatchSlotClick(slotIndex, patch) {
    // For now, just log - could open a context menu or detail view
    console.log("Patch slot clicked:", slotIndex, patch);
  }

  function handleSequenceSlotClick(slotIndex, sequence) {
    console.log("Sequence slot clicked:", slotIndex, sequence);
  }
</script>

<div class="h-full flex flex-col">
  {#if statusMessage}
    <div
      class="mx-4 mt-4 p-3 rounded-lg {statusMessage.type === 'success'
        ? 'bg-green-500/20 text-green-400 border border-green-500/30'
        : 'bg-red-500/20 text-red-400 border border-red-500/30'}"
    >
      {statusMessage.text}
    </div>
  {/if}

  {#if !selectedLibraryId}
    <div class="flex-1 flex items-center justify-center">
      <div class="text-center text-text-secondary">
        <p class="text-lg mb-2">No Library Selected</p>
        <p class="text-sm">Select a library from the sidebar to view its banks</p>
      </div>
    </div>
  {:else}
    <div class="p-4 border-b border-border bg-surface">
      <h1 class="text-xl font-semibold">Banks</h1>
      {#if selectedLibrary}
        <p class="text-sm text-text-secondary mt-1">
          {selectedLibrary.name}
        </p>
      {/if}
    </div>

    <div class="flex-1 flex overflow-hidden">
      <div class="w-64 flex-shrink-0">
        <BankList
          {banks}
          bind:selectedBankNumber
          {loading}
        />
      </div>

      <div class="flex-1 border-l border-border">
        <BankDetail
          bank={selectedBank}
          onPatchSlotClick={handlePatchSlotClick}
          onSequenceSlotClick={handleSequenceSlotClick}
          onPatchSlotDrop={handlePatchSlotDrop}
          onSequenceSlotDrop={handleSequenceSlotDrop}
        />
      </div>
    </div>
  {/if}
</div>
