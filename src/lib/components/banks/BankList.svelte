<script>
  let {
    banks = [],
    selectedBankNumber = $bindable(null),
    loading = false
  } = $props();

  function selectBank(bankNumber) {
    selectedBankNumber = bankNumber;
  }

  function getFilledSlotCount(bank) {
    const patchCount = bank.patch_slots?.filter(s => s.content !== null).length || 0;
    const seqCount = bank.sequence_slots?.filter(s => s.content !== null).length || 0;
    return { patchCount, seqCount };
  }
</script>

<div class="h-full flex flex-col bg-surface border-r border-border">
  <div class="p-3 border-b border-border">
    <h2 class="text-sm font-semibold text-text-secondary uppercase tracking-wider">Banks</h2>
  </div>

  <div class="flex-1 overflow-y-auto">
    {#if loading}
      <div class="p-4 text-center text-text-secondary">
        Loading banks...
      </div>
    {:else if banks.length === 0}
      <div class="p-4 text-center text-text-secondary">
        No banks found
      </div>
    {:else}
      {#each banks as bank}
        {@const slots = getFilledSlotCount(bank)}
        <button
          class="w-full text-left px-3 py-2 border-b border-border/50 transition-colors
            {selectedBankNumber === bank.bank_number
              ? 'bg-primary/20 text-primary'
              : 'hover:bg-border/50 text-text-primary'}"
          onclick={() => selectBank(bank.bank_number)}
        >
          <div class="flex items-center justify-between">
            <div class="min-w-0 flex-1">
              <div class="font-medium truncate">{bank.name}</div>
              <div class="text-xs text-text-secondary mt-0.5">
                {slots.patchCount}/16 patches, {slots.seqCount}/16 sequences
              </div>
            </div>
            <div class="text-xs text-text-secondary ml-2">
              #{bank.bank_number.toString().padStart(2, '0')}
            </div>
          </div>
        </button>
      {/each}
    {/if}
  </div>
</div>
