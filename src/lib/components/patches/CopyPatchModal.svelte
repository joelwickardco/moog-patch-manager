<script>
  import { getBanksForLibrary } from "../../utils/api.js";

  let {
    open = false,
    patch = null,
    libraries = [],
    onClose = () => {},
    onSubmit = () => {}
  } = $props();

  let selectedLibraryId = $state(null);
  let selectedBankNumber = $state(1);
  let slotSelectionMode = $state("auto"); // "auto" or "manual"
  let selectedSlotNumber = $state(null);
  let banks = $state([]);
  let loading = $state(false);
  let error = $state("");

  // Derived state
  let selectedBank = $derived(
    banks.find((b) => b.bank_number === selectedBankNumber) || null
  );

  let nextAvailableSlot = $derived(calculateNextAvailableSlot(selectedBank));

  let finalSlotNumber = $derived(
    slotSelectionMode === "auto" ? nextAvailableSlot : selectedSlotNumber
  );

  let canSubmit = $derived(
    selectedLibraryId && selectedBankNumber && finalSlotNumber !== null
  );

  let selectedLibrary = $derived(
    libraries.find((lib) => lib.id === selectedLibraryId) || null
  );

  // Load banks when library changes
  $effect(() => {
    if (selectedLibraryId) {
      loadBanks();
    }
  });

  // Reset state when modal opens
  $effect(() => {
    if (open) {
      error = "";
      banks = [];
      selectedBankNumber = 1;
      slotSelectionMode = "auto";
      selectedSlotNumber = null;

      // Pre-select first library if available
      if (libraries.length > 0 && !selectedLibraryId) {
        selectedLibraryId = libraries[0].id;
      }
    }
  });

  async function loadBanks() {
    try {
      loading = true;
      error = "";
      banks = await getBanksForLibrary(selectedLibraryId);
    } catch (e) {
      console.error("Failed to load banks:", e);
      error = "Failed to load banks";
      banks = [];
    } finally {
      loading = false;
    }
  }

  function calculateNextAvailableSlot(bank) {
    if (!bank || !bank.patch_slots) return null;

    for (let i = 0; i < 16; i++) {
      const slot = bank.patch_slots[i];
      if (!slot.content) {
        return i + 1; // Slots are 1-indexed
      }
    }

    return null; // Bank is full
  }

  function getSlotStatus(bank, slotNumber) {
    if (!bank || !bank.patch_slots) return "unknown";
    const slot = bank.patch_slots[slotNumber - 1];
    return slot?.content ? "filled" : "empty";
  }

  function getFilledSlotCount(bank) {
    if (!bank || !bank.patch_slots) return 0;
    return bank.patch_slots.filter((s) => s.content !== null).length;
  }

  async function handleSubmit(e) {
    e.preventDefault();

    if (!canSubmit) {
      error = "Please select a library, bank, and slot";
      return;
    }

    try {
      loading = true;
      error = "";
      await onSubmit(selectedLibraryId, selectedBankNumber, finalSlotNumber);
    } catch (e) {
      console.error("Copy failed:", e);
      error = `Failed to copy patch: ${e}`;
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e) {
    if (e.key === "Escape") {
      onClose();
    }
  }

  function handleBackdropClick(e) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }
</script>

{#if open && patch}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
  >
    <div
      class="bg-surface rounded-lg shadow-xl w-full max-w-2xl mx-4 border border-border max-h-[90vh] flex flex-col"
    >
      <!-- Header -->
      <div class="p-4 border-b border-border flex-shrink-0">
        <h2 id="modal-title" class="text-lg font-semibold">
          Copy Patch: {patch.name}
        </h2>
      </div>

      <!-- Body -->
      <form onsubmit={handleSubmit} class="flex-1 overflow-y-auto">
        <div class="p-4 space-y-6">
          <!-- Library Selector -->
          <div>
            <div class="block mb-2">
              <span class="text-sm text-text-secondary">Select Library</span>
            </div>

            {#if libraries.length === 0}
              <div class="text-sm text-text-secondary p-4 bg-background rounded-lg text-center">
                No libraries available. Please create a library first.
              </div>
            {:else}
              <div class="max-h-40 overflow-y-auto border border-border rounded-lg">
                {#each libraries as library}
                  <button
                    type="button"
                    class="w-full text-left px-3 py-2 border-b border-border/50 transition-colors
                      {selectedLibraryId === library.id
                        ? 'bg-primary/20 text-primary'
                        : 'hover:bg-border/50'}"
                    onclick={() => {
                      selectedLibraryId = library.id;
                    }}
                  >
                    <div class="flex items-center justify-between">
                      <div class="flex items-center gap-2 min-w-0 flex-1">
                        {#if library.color}
                          <div
                            class="w-2 h-2 rounded-full flex-shrink-0"
                            style="background-color: {library.color}"
                          ></div>
                        {/if}
                        <span class="truncate">{library.name}</span>
                      </div>
                      <span class="text-xs text-text-secondary ml-2"
                        >{library.patch_count}</span
                      >
                    </div>
                  </button>
                {/each}
              </div>
            {/if}
          </div>

          <!-- Bank Selector -->
          {#if selectedLibraryId}
            <div>
              <div class="block mb-2">
                <span class="text-sm text-text-secondary">Select Bank</span>
              </div>

              {#if loading}
                <div class="text-sm text-text-secondary p-4 text-center">
                  Loading banks...
                </div>
              {:else if banks.length === 0}
                <div class="text-sm text-text-secondary p-4 text-center">
                  No banks found
                </div>
              {:else}
                <div class="grid grid-cols-4 gap-2">
                  {#each banks as bank}
                    {@const filledCount = getFilledSlotCount(bank)}
                    <button
                      type="button"
                      class="p-2 rounded-lg border transition-colors text-left
                        {selectedBankNumber === bank.bank_number
                          ? 'border-primary bg-primary/20'
                          : 'border-border hover:border-primary/50'}"
                      onclick={() => {
                        selectedBankNumber = bank.bank_number;
                      }}
                    >
                      <div class="text-xs text-text-secondary mb-1">
                        #{bank.bank_number.toString().padStart(2, "0")}
                      </div>
                      <div class="text-xs font-medium truncate">{bank.name}</div>
                      <div class="text-xs text-text-secondary">{filledCount}/16</div>
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}

          <!-- Slot Selector -->
          {#if selectedBank}
            <div>
              <div class="block mb-2">
                <span class="text-sm text-text-secondary">Select Slot</span>
              </div>

              <div class="space-y-3">
                <!-- Auto mode -->
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="slot-mode"
                    value="auto"
                    bind:group={slotSelectionMode}
                    class="text-primary focus:ring-primary"
                  />
                  <span class="text-sm">
                    Next Available
                    {#if nextAvailableSlot !== null}
                      <span class="text-text-secondary"
                        >(Slot #{nextAvailableSlot
                          .toString()
                          .padStart(2, "0")})</span
                      >
                    {:else}
                      <span class="text-red-400">(Bank is full)</span>
                    {/if}
                  </span>
                </label>

                <!-- Manual mode -->
                <label class="flex items-center gap-2 cursor-pointer">
                  <input
                    type="radio"
                    name="slot-mode"
                    value="manual"
                    bind:group={slotSelectionMode}
                    class="text-primary focus:ring-primary"
                  />
                  <span class="text-sm">Choose Specific Slot</span>
                </label>

                <!-- Manual slot grid -->
                {#if slotSelectionMode === "manual"}
                  <div class="grid grid-cols-8 gap-1 mt-2 pl-6">
                    {#each Array(16).fill(0) as _, i}
                      {@const slotNum = i + 1}
                      {@const status = getSlotStatus(selectedBank, slotNum)}
                      <button
                        type="button"
                        class="aspect-square p-1 rounded text-xs border transition-colors
                          {selectedSlotNumber === slotNum
                            ? 'border-primary bg-primary/20 text-primary'
                            : status === 'filled'
                              ? 'border-border bg-border/30 text-text-secondary'
                              : 'border-border border-dashed hover:border-primary/50'}"
                        onclick={() => {
                          selectedSlotNumber = slotNum;
                        }}
                      >
                        {slotNum.toString().padStart(2, "0")}
                      </button>
                    {/each}
                  </div>
                {/if}
              </div>
            </div>
          {/if}

          <!-- Preview -->
          {#if selectedLibrary && selectedBank && finalSlotNumber}
            <div class="p-3 bg-background rounded-lg border border-border">
              <div class="text-xs text-text-secondary mb-1">Will copy to:</div>
              <div class="text-sm font-medium">
                {selectedLibrary.name}
                <span class="text-text-secondary">›</span>
                Bank #{selectedBankNumber.toString().padStart(2, "0")}
                <span class="text-text-secondary">›</span>
                Slot #{finalSlotNumber.toString().padStart(2, "0")}
              </div>
            </div>
          {/if}

          <!-- Error message -->
          {#if error}
            <div class="p-3 bg-red-500/10 border border-red-500/20 rounded-lg">
              <p class="text-red-400 text-sm">{error}</p>
            </div>
          {/if}
        </div>

        <!-- Footer -->
        <div class="p-4 border-t border-border flex justify-end gap-2 flex-shrink-0">
          <button
            type="button"
            onclick={onClose}
            class="px-4 py-2 rounded-lg hover:bg-border transition-colors"
            disabled={loading}
          >
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            disabled={!canSubmit || loading}
          >
            {loading ? "Copying..." : "Copy Patch"}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
