<script>
  let {
    banks = [],
    selectedBankNumber = $bindable(1),
    loading = false,
    libraryName = ''
  } = $props();

  let buttonStripRef = $state(null);

  function getBankByNumber(num) {
    return banks.find(b => b.bank_number === num) || null;
  }

  function isBankEmpty(bank) {
    if (!bank) return true;
    const patchCount = bank.patch_slots?.filter(s => s.content !== null).length || 0;
    const seqCount = bank.sequence_slots?.filter(s => s.content !== null).length || 0;
    return patchCount === 0 && seqCount === 0;
  }

  function selectBank(num) {
    selectedBankNumber = num;
  }

  function handleKeydown(e, num) {
    let targetNum = null;

    if (e.key === 'ArrowRight') {
      e.preventDefault();
      targetNum = num < 16 ? num + 1 : 1;
    } else if (e.key === 'ArrowLeft') {
      e.preventDefault();
      targetNum = num > 1 ? num - 1 : 16;
    } else if (e.key === 'Home') {
      e.preventDefault();
      targetNum = 1;
    } else if (e.key === 'End') {
      e.preventDefault();
      targetNum = 16;
    }

    if (targetNum !== null) {
      selectBank(targetNum);
      // Move focus to the newly selected button
      const btn = buttonStripRef?.querySelector(`[data-bank="${targetNum}"]`);
      btn?.focus();
    }
  }

  function getButtonClasses(num) {
    const bank = getBankByNumber(num);
    const isSelected = selectedBankNumber === num;
    const isEmpty = isBankEmpty(bank);

    const base = 'w-10 h-20 rounded-md text-sm font-bold transition-all duration-150 border flex items-center justify-center focus-visible:outline-2 focus-visible:outline-primary focus-visible:outline-offset-1';

    if (loading) {
      return `${base} bg-background border-border/30 text-transparent animate-pulse pointer-events-none`;
    }

    if (isSelected) {
      return `${base} bg-primary text-white border-primary shadow-[0_0_10px_rgba(255,107,53,0.4)] hover:bg-primary/90 hover:shadow-[0_0_12px_rgba(255,107,53,0.5)]`;
    }

    if (isEmpty) {
      return `${base} bg-background border-border/50 text-text-secondary/40 hover:bg-[#252525] hover:text-text-secondary/70`;
    }

    return `${base} bg-[#353535] border-[#4a4a4a] text-text-secondary hover:bg-[#3d3d3d] hover:border-[#555555] hover:text-text-primary`;
  }

  function getAriaLabel(num) {
    const bank = getBankByNumber(num);
    const isEmpty = isBankEmpty(bank);
    return `Bank ${num}${isEmpty ? ' (empty)' : ''}`;
  }
</script>

<div
  class="px-4 py-3 bg-surface border-b border-border flex items-center gap-1.5"
  role="toolbar"
  aria-label="Bank selector"
>
  <span class="text-sm font-medium text-text-secondary mr-3 flex-shrink-0 truncate max-w-[140px]" title={libraryName}>
    {libraryName}
  </span>

  <div class="flex gap-1.5 flex-1 justify-center" bind:this={buttonStripRef}>
    {#each Array(16) as _, i}
      {@const num = i + 1}
      <button
        class={getButtonClasses(num)}
        onclick={() => selectBank(num)}
        onkeydown={(e) => handleKeydown(e, num)}
        aria-label={getAriaLabel(num)}
        aria-pressed={selectedBankNumber === num}
        tabindex={selectedBankNumber === num ? 0 : -1}
        data-bank={num}
      >
        {num}
      </button>
    {/each}
  </div>
</div>
