<script>
  import { getBanksForLibrary } from '../../utils/api.js';

  let {
    patch = null,
    libraries = [],
    onClose = () => {},
    onSubmit = () => {}
  } = $props();

  let selectedLibraryId = $state(null);
  let selectedBankNumber = $state(1);
  let slotSelectionMode = $state('auto'); // 'auto' or 'manual'
  let selectedSlotNumber = $state(null);
  let banks = $state([]);
  let loading = $state(false);
  let error = $state('');

  // Derived state
  let selectedBank = $derived(
    banks.find((b) => b.bank_number === selectedBankNumber) || null
  );

  let nextAvailableSlot = $derived(calculateNextAvailableSlot(selectedBank));

  let finalSlotNumber = $derived(
    slotSelectionMode === 'auto' ? nextAvailableSlot : selectedSlotNumber
  );

  let canSubmit = $derived(
    selectedLibraryId && selectedBankNumber && finalSlotNumber !== null
  );

  let selectedLibrary = $derived(
    libraries.find((lib) => lib.id === selectedLibraryId) || null
  );

  // Load banks when library selection changes
  $effect(() => {
    if (selectedLibraryId) {
      loadBanks();
    }
  });

  // Reset state when patch changes (sidebar opens with a new patch)
  $effect(() => {
    const currentPatchId = patch?.id;
    if (currentPatchId !== undefined && currentPatchId !== null) {
      error = '';
      banks = [];
      selectedBankNumber = 1;
      slotSelectionMode = 'auto';
      selectedSlotNumber = null;

      // Pre-select first library if available
      if (libraries.length > 0) {
        selectedLibraryId = libraries[0].id;
      } else {
        selectedLibraryId = null;
      }
    }
  });

  // Listen for Escape key to close sidebar
  $effect(() => {
    if (!patch) return;
    function handleKeydown(e) {
      if (e.key === 'Escape') {
        onClose();
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  async function loadBanks() {
    try {
      loading = true;
      error = '';
      banks = await getBanksForLibrary(selectedLibraryId);
    } catch (e) {
      console.error('Failed to load banks:', e);
      error = 'Failed to load banks';
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
    if (!bank || !bank.patch_slots) return 'unknown';
    const slot = bank.patch_slots[slotNumber - 1];
    return slot?.content ? 'filled' : 'empty';
  }

  function getFilledSlotCount(bank) {
    if (!bank || !bank.patch_slots) return 0;
    return bank.patch_slots.filter((s) => s.content !== null).length;
  }

  async function handleSubmit(e) {
    e.preventDefault();

    if (!canSubmit) {
      error = 'Please select a library, bank, and slot';
      return;
    }

    try {
      loading = true;
      error = '';
      await onSubmit(selectedLibraryId, selectedBankNumber, finalSlotNumber);
    } catch (e) {
      console.error('Copy failed:', e);
      error = `Failed to copy patch: ${e}`;
    } finally {
      loading = false;
    }
  }
</script>

<aside
  class="w-[360px] min-w-[360px] h-full flex flex-col bg-background border-l border-border"
  aria-label="Copy patch"
>
  {#if patch}
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-border flex-shrink-0">
      <div class="min-w-0 flex-1 pr-2">
        <h2 class="text-lg font-semibold">Copy Patch</h2>
        <p class="text-sm text-text-secondary truncate" title={patch.name}>{patch.name}</p>
      </div>
      <button
        class="p-1.5 hover:bg-surface rounded-lg transition-colors flex-shrink-0"
        aria-label="Close copy patch"
        onclick={onClose}
      >
        <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>

    <!-- Scrollable body -->
    <form onsubmit={handleSubmit} class="flex-1 overflow-y-auto flex flex-col">
      <div class="p-4 space-y-6 flex-1">

        <!-- Library Selector -->
        <div>
          <span class="block text-sm text-text-secondary mb-2">Select Library</span>

          {#if libraries.length === 0}
            <div class="text-sm text-text-secondary p-4 bg-surface rounded-lg text-center">
              No libraries available. Please create a library first.
            </div>
          {:else}
            <div class="max-h-40 overflow-y-auto border border-border rounded-lg">
              {#each libraries as library}
                <button
                  type="button"
                  class="w-full text-left px-3 py-2 border-b border-border/50 transition-colors last:border-b-0
                    {selectedLibraryId === library.id
                      ? 'bg-primary/20 text-primary'
                      : 'hover:bg-border/50'}"
                  onclick={() => { selectedLibraryId = library.id; }}
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
                    <span class="text-xs text-text-secondary ml-2">{library.patch_count}</span>
                  </div>
                </button>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Bank Selector -->
        {#if selectedLibraryId}
          <div>
            <span class="block text-sm text-text-secondary mb-2">Select Bank</span>

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
                    onclick={() => { selectedBankNumber = bank.bank_number; }}
                  >
                    <div class="text-xs text-text-secondary mb-1">
                      #{bank.bank_number.toString().padStart(2, '0')}
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
            <span class="block text-sm text-text-secondary mb-2">Select Slot</span>

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
                    <span class="text-text-secondary">
                      (Slot #{nextAvailableSlot.toString().padStart(2, '0')})
                    </span>
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
              {#if slotSelectionMode === 'manual'}
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
                      onclick={() => { selectedSlotNumber = slotNum; }}
                    >
                      {slotNum.toString().padStart(2, '0')}
                    </button>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Preview -->
        {#if selectedLibrary && selectedBank && finalSlotNumber}
          <div class="p-3 bg-surface rounded-lg border border-border">
            <div class="text-xs text-text-secondary mb-1">Will copy to:</div>
            <div class="text-sm font-medium">
              {selectedLibrary.name}
              <span class="text-text-secondary">›</span>
              Bank #{selectedBankNumber.toString().padStart(2, '0')}
              <span class="text-text-secondary">›</span>
              Slot #{finalSlotNumber.toString().padStart(2, '0')}
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
      <div class="p-4 border-t border-border flex gap-2 flex-shrink-0">
        <button
          type="button"
          onclick={onClose}
          class="flex-1 px-4 py-2 rounded-lg border border-border hover:bg-border transition-colors text-sm"
          disabled={loading}
        >
          Cancel
        </button>
        <button
          type="submit"
          class="flex-1 px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm"
          disabled={!canSubmit || loading}
        >
          {loading ? 'Copying...' : 'Copy Patch'}
        </button>
      </div>
    </form>
  {/if}
</aside>
