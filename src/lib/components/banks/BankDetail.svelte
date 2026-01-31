<script>
  let {
    bank = null,
    libraryId = null,
    onPatchSlotClick = () => {},
    onSequenceSlotClick = () => {},
    onPatchSlotDrop = () => {},
    onSequenceSlotDrop = () => {},
    onBankNameUpdate = () => {}
  } = $props();

  import { updateBankName } from "../../utils/api.js";

  let dragOverPatchSlot = $state(null);
  let dragOverSequenceSlot = $state(null);
  let isEditingName = $state(false);
  let editedName = $state("");
  let nameInputElement = $state(null);

  function startEditingName() {
    if (!bank) return;
    editedName = bank.name;
    isEditingName = true;
    // Focus input after it's rendered
    setTimeout(() => {
      if (nameInputElement) {
        nameInputElement.focus();
        nameInputElement.select();
      }
    }, 0);
  }

  async function saveBankName() {
    if (!bank || !libraryId || !editedName.trim()) {
      isEditingName = false;
      return;
    }

    try {
      await updateBankName(libraryId, bank.bank_number, editedName.trim());
      isEditingName = false;
      onBankNameUpdate();
    } catch (e) {
      console.error("Failed to update bank name:", e);
      isEditingName = false;
    }
  }

  function cancelEdit() {
    isEditingName = false;
    editedName = "";
  }

  function handleNameKeydown(e) {
    if (e.key === "Enter") {
      saveBankName();
    } else if (e.key === "Escape") {
      cancelEdit();
    }
  }

  function handlePatchDragOver(e, slotIndex) {
    e.preventDefault();
    dragOverPatchSlot = slotIndex;
  }

  function handlePatchDragLeave() {
    dragOverPatchSlot = null;
  }

  function handlePatchDrop(e, slotIndex) {
    e.preventDefault();
    dragOverPatchSlot = null;

    const data = e.dataTransfer.getData("application/json");
    if (data) {
      try {
        const patch = JSON.parse(data);
        onPatchSlotDrop(slotIndex, patch);
      } catch (err) {
        console.error("Failed to parse drop data:", err);
      }
    }
  }

  function handleSequenceDragOver(e, slotIndex) {
    e.preventDefault();
    dragOverSequenceSlot = slotIndex;
  }

  function handleSequenceDragLeave() {
    dragOverSequenceSlot = null;
  }

  function handleSequenceDrop(e, slotIndex) {
    e.preventDefault();
    dragOverSequenceSlot = null;

    const data = e.dataTransfer.getData("application/json");
    if (data) {
      try {
        const sequence = JSON.parse(data);
        onSequenceSlotDrop(slotIndex, sequence);
      } catch (err) {
        console.error("Failed to parse drop data:", err);
      }
    }
  }

  function handlePatchDragStart(e, patch, slotIndex) {
    if (!patch) return;
    e.dataTransfer.setData("application/json", JSON.stringify({
      ...patch,
      sourceType: "bank",
      sourceBankNumber: bank.bank_number,
      sourceSlotIndex: slotIndex
    }));
    e.dataTransfer.effectAllowed = "copyMove";
  }

  function handleSequenceDragStart(e, sequence, slotIndex) {
    if (!sequence) return;
    e.dataTransfer.setData("application/json", JSON.stringify({
      ...sequence,
      sourceType: "bank",
      sourceBankNumber: bank.bank_number,
      sourceSlotIndex: slotIndex
    }));
    e.dataTransfer.effectAllowed = "copyMove";
  }
</script>

<div class="h-full flex flex-col bg-background">
  {#if !bank}
    <div class="flex-1 flex items-center justify-center text-text-secondary">
      Select a bank to view its contents
    </div>
  {:else}
    <div class="p-4 border-b border-border bg-surface">
      <div class="flex items-center justify-between">
        <div class="flex-1 mr-4">
          {#if isEditingName}
            <input
              type="text"
              bind:value={editedName}
              bind:this={nameInputElement}
              onkeydown={handleNameKeydown}
              onblur={saveBankName}
              class="text-lg font-semibold bg-background border border-primary rounded px-2 py-1 w-full outline-none focus:ring-2 focus:ring-primary/50"
              placeholder="Bank name"
            />
          {:else}
            <button
              onclick={startEditingName}
              class="text-left w-full group"
            >
              <h2 class="text-lg font-semibold group-hover:text-primary transition-colors">
                {bank.name}
                <span class="ml-2 text-xs text-text-secondary opacity-0 group-hover:opacity-100 transition-opacity">
                  (click to edit)
                </span>
              </h2>
            </button>
          {/if}
          <p class="text-sm text-text-secondary mt-1">Bank #{bank.bank_number.toString().padStart(2, '0')}</p>
        </div>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto p-4 space-y-6">
      <!-- Patches Section -->
      <div>
        <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider mb-3">
          Patches ({bank.patch_slots?.filter(s => s.content !== null).length || 0}/16)
        </h3>
        <div class="grid grid-cols-4 gap-2">
          {#each Array(16).fill(0) as _, i}
            {@const slot = bank.patch_slots?.[i]}
            {@const patch = slot?.content || null}
            {@const slotNumber = (i + 1).toString().padStart(2, '0')}
            <button
              class="relative p-3 rounded-lg border transition-all text-left
                {patch
                  ? 'bg-surface border-border hover:border-primary cursor-grab'
                  : 'bg-background border-border/50 border-dashed'}
                {dragOverPatchSlot === i ? 'border-primary bg-primary/10' : ''}"
              onclick={() => onPatchSlotClick(i, patch)}
              ondragover={(e) => handlePatchDragOver(e, i)}
              ondragleave={handlePatchDragLeave}
              ondrop={(e) => handlePatchDrop(e, i)}
              draggable={patch !== null}
              ondragstart={(e) => handlePatchDragStart(e, patch, i)}
            >
              <div class="text-xs text-text-secondary mb-1">#{slotNumber}</div>
              {#if patch}
                <div class="font-medium text-sm truncate">{patch.name}</div>
                {#if patch.source_library}
                  <div class="text-xs text-text-secondary truncate mt-0.5">{patch.source_library}</div>
                {/if}
              {:else}
                <div class="text-sm text-text-secondary/50">Empty</div>
              {/if}
            </button>
          {/each}
        </div>
      </div>

      <!-- Sequences Section -->
      <div>
        <h3 class="text-sm font-semibold text-text-secondary uppercase tracking-wider mb-3">
          Sequences ({bank.sequence_slots?.filter(s => s.content !== null).length || 0}/16)
        </h3>
        <div class="grid grid-cols-4 gap-2">
          {#each Array(16).fill(0) as _, i}
            {@const slot = bank.sequence_slots?.[i]}
            {@const sequence = slot?.content || null}
            {@const slotNumber = (i + 1).toString().padStart(2, '0')}
            <button
              class="relative p-3 rounded-lg border transition-all text-left
                {sequence
                  ? 'bg-surface border-border hover:border-secondary cursor-grab'
                  : 'bg-background border-border/50 border-dashed'}
                {dragOverSequenceSlot === i ? 'border-secondary bg-secondary/10' : ''}"
              onclick={() => onSequenceSlotClick(i, sequence)}
              ondragover={(e) => handleSequenceDragOver(e, i)}
              ondragleave={handleSequenceDragLeave}
              ondrop={(e) => handleSequenceDrop(e, i)}
              draggable={sequence !== null}
              ondragstart={(e) => handleSequenceDragStart(e, sequence, i)}
            >
              <div class="text-xs text-text-secondary mb-1">#{slotNumber}</div>
              {#if sequence}
                <div class="font-medium text-sm truncate">{sequence.name}</div>
              {:else}
                <div class="text-sm text-text-secondary/50">Empty</div>
              {/if}
            </button>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>
