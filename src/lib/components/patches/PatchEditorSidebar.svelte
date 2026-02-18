<script>
  import TagInput from "./TagInput.svelte";
  import { updatePatchTags, updatePatchNotes, toggleFavorite as toggleFavoriteApi, deletePatch, getAllTags, clearPatchSlot } from "../../utils/api.js";
  import { isNewPatch, isModifiedPatch } from "../../utils/patchStates.js";

  let {
    patch = null,
    onClose = () => {},
    onSaved = () => {},
    onDeleted = () => {},
    bankContext = null,
    onRemovedFromSlot = () => {}
  } = $props();

  let editedTags = $state([]);
  let editedNotes = $state("");
  let availableTags = $state([]);
  let saveStatus = $state(null); // null | 'saving' | 'saved' | 'error'
  let deleteConfirming = $state(false);
  let deleteConfirmTimer = $state(null);
  let deleting = $state(false);
  let removingFromSlot = $state(false);

  // Track which patch we've loaded form data for
  let loadedPatchId = $state(null);
  let closeButtonEl = $state(null);

  // Load tags on mount
  $effect(() => {
    loadTags();
  });

  // When patch changes, auto-save previous and load new
  $effect(() => {
    const currentPatch = patch;
    if (currentPatch && currentPatch.id !== loadedPatchId) {
      // Save previous patch if we had one loaded
      if (loadedPatchId !== null) {
        saveIfChanged();
      }
      // Load new patch data
      editedTags = [...(currentPatch.tags || [])];
      editedNotes = currentPatch.notes || "";
      loadedPatchId = currentPatch.id;
      deleteConfirming = false;
      clearDeleteTimer();
      saveStatus = null;
      removingFromSlot = false;
    } else if (!currentPatch) {
      loadedPatchId = null;
    }
  });

  // Focus close button when sidebar opens
  $effect(() => {
    if (patch && closeButtonEl) {
      closeButtonEl.focus();
    }
  });

  async function loadTags() {
    try {
      const tagDtos = await getAllTags();
      availableTags = tagDtos.map(t => t.name);
    } catch (e) {
      console.error("Failed to load tags:", e);
      availableTags = [];
    }
  }

  async function saveIfChanged() {
    if (!patch) return;

    const tagsChanged = JSON.stringify([...editedTags].sort()) !== JSON.stringify([...(patch.tags || [])].sort());
    const notesChanged = editedTags !== undefined && editedNotes !== (patch.notes || "");

    if (!tagsChanged && !notesChanged) return;

    saveStatus = 'saving';
    try {
      if (tagsChanged) await updatePatchTags(patch.id, editedTags);
      if (notesChanged) await updatePatchNotes(patch.id, editedNotes);
      saveStatus = 'saved';
      onSaved();
      setTimeout(() => {
        if (saveStatus === 'saved') saveStatus = null;
      }, 1500);
    } catch (e) {
      console.error("Auto-save failed:", e);
      saveStatus = 'error';
    }
  }

  function handleClose() {
    saveIfChanged();
    onClose();
  }

  // Listen for Escape key to close sidebar
  $effect(() => {
    if (!patch) return;
    function handleKeydown(e) {
      if (e.key === 'Escape') {
        handleClose();
      }
    }
    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });

  async function handleToggleFavorite() {
    if (!patch) return;
    try {
      const newValue = await toggleFavoriteApi(patch.id);
      patch.is_favorite = newValue;
      onSaved();
    } catch (e) {
      console.error("Failed to toggle favorite:", e);
    }
  }

  function clearDeleteTimer() {
    if (deleteConfirmTimer) {
      clearTimeout(deleteConfirmTimer);
      deleteConfirmTimer = null;
    }
  }

  $effect(() => {
    return () => {
      clearDeleteTimer();
    };
  });
  async function handleDelete() {
    if (!patch) return;

    if (!deleteConfirming) {
      deleteConfirming = true;
      clearDeleteTimer();
      deleteConfirmTimer = setTimeout(() => {
        deleteConfirming = false;
      }, 3000);
      return;
    }

    // Confirmed - perform delete
    clearDeleteTimer();
    deleting = true;
    try {
      await deletePatch(patch.id);
      onDeleted();
    } catch (e) {
      console.error("Failed to delete patch:", e);
      deleteConfirming = false;
      deleting = false;
    }
  }

  async function handleRemoveFromSlot() {
    if (!bankContext || !patch) return;
    removingFromSlot = true;
    try {
      await clearPatchSlot(bankContext.libraryId, bankContext.bankNumber, bankContext.slotNumber);
      onRemovedFromSlot();
    } catch (e) {
      console.error("Failed to remove from slot:", e);
      removingFromSlot = false;
    }
  }

  let isNew = $derived(patch ? isNewPatch(patch.created_at) : false);
  let isModified = $derived(patch ? isModifiedPatch(patch.created_at, patch.updated_at) : false);
  let isUntagged = $derived(patch ? (!patch.tags || patch.tags.length === 0) : false);
  let hasMultipleUses = $derived(patch ? (patch.usage_count && patch.usage_count > 1) : false);
</script>

<aside
  class="w-[360px] min-w-[360px] h-full flex flex-col bg-background border-l border-border"
  aria-label="Patch editor"
>
  {#if patch}
    <!-- Header -->
    <div class="flex items-center justify-between p-4 border-b border-border">
      <h2 class="text-lg font-semibold">Patch Details</h2>
      <button
        bind:this={closeButtonEl}
        class="p-1.5 hover:bg-surface rounded-lg transition-colors"
        aria-label="Close patch details"
        onclick={handleClose}
      >
        <svg class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>

    <!-- Save Status -->
    {#if saveStatus === 'saving'}
      <span class="text-xs text-text-secondary px-4 pt-2">Saving...</span>
    {:else if saveStatus === 'saved'}
      <span class="text-xs text-green-400 px-4 pt-2">Saved</span>
    {:else if saveStatus === 'error'}
      <span class="text-xs text-red-400 px-4 pt-2">Failed to save</span>
    {/if}

    <!-- Patch Name -->
    <div class="px-4 pt-4">
      <h3 class="text-xl font-bold truncate" title={patch.name}>
        {#if isModified}
          <span
            class="w-1.5 h-1.5 rounded-full bg-orange-500 inline-block mr-1.5"
            title="Modified since import"
            aria-label="Modified"
          ></span>
        {/if}
        {patch.name}
      </h3>
    </div>

    <!-- Metadata Row -->
    <div class="px-4 pt-2 pb-4 flex items-center gap-3 flex-wrap">
      {#if patch.source_library}
        <span class="text-xs px-2 py-0.5 rounded bg-secondary/20 text-secondary">
          {patch.source_library}
        </span>
      {/if}
      {#if hasMultipleUses}
        <span class="text-xs px-2 py-0.5 rounded bg-state-multi-use/20 text-state-multi-use">
          Used in {patch.usage_count} banks
        </span>
      {/if}
      {#if isNew}
        <span class="px-2 py-0.5 text-xs font-medium rounded-full bg-green-500 text-white">
          New
        </span>
      {/if}
      {#if isUntagged}
        <span class="px-2 py-0.5 text-xs rounded bg-gray-600/50 text-gray-300">
          Untagged
        </span>
      {/if}
    </div>

    <!-- Divider -->
    <div class="border-t border-border"></div>

    <!-- Favorite Toggle -->
    <div class="mx-4 my-4">
      {#if patch.is_favorite}
        <button
          class="w-full px-4 py-2 rounded-lg border border-favorite bg-favorite/10 hover:bg-favorite/20 transition-colors flex items-center justify-center gap-2 text-favorite"
          onclick={handleToggleFavorite}
        >
          <span>&#9733;</span> Remove from Favorites
        </button>
      {:else}
        <button
          class="w-full px-4 py-2 rounded-lg border border-border bg-surface hover:bg-border transition-colors flex items-center justify-center gap-2 text-text-secondary"
          onclick={handleToggleFavorite}
        >
          <span>&#9734;</span> Add to Favorites
        </button>
      {/if}
    </div>

    <!-- Divider -->
    <div class="border-t border-border"></div>

    <!-- Tags Section -->
    <div class="px-4 py-4">
      <span class="block text-sm font-medium text-text-secondary mb-1.5">Tags</span>
      <TagInput
        bind:selectedTags={editedTags}
        {availableTags}
        placeholder="Add tags..."
      />
    </div>

    <!-- Notes Section -->
    <div class="px-4 pb-4">
      <label for="sidebar-patch-notes" class="block text-sm font-medium text-text-secondary mb-1.5">Notes</label>
      <textarea
        id="sidebar-patch-notes"
        bind:value={editedNotes}
        class="w-full px-3 py-2 bg-surface border border-border rounded-lg resize-none text-sm focus:border-primary focus:ring-1 focus:ring-primary/50 outline-none transition-colors"
        rows="6"
        placeholder="Add notes about this patch..."
      ></textarea>
    </div>

    <!-- Spacer -->
    <div class="flex-1"></div>

    <!-- Delete Section -->
    <div class="p-4 border-t border-border mt-auto">
      {#if bankContext}
        <button
          class="w-full px-4 py-2 rounded-lg text-sm transition-colors border border-border bg-surface hover:bg-border text-text-secondary mb-2"
          onclick={handleRemoveFromSlot}
          disabled={removingFromSlot}
        >
          {removingFromSlot ? 'Removing...' : 'Remove from Slot'}
        </button>
      {/if}
      <button
        class="w-full px-4 py-2 rounded-lg text-sm transition-colors {deleteConfirming
          ? 'border border-red-500 bg-red-500/20 text-red-400'
          : 'border border-red-500/30 text-red-400 hover:bg-red-500/10'}"
        onclick={handleDelete}
        disabled={deleting}
        aria-live="polite"
      >
        {#if deleting}
          Deleting...
        {:else if deleteConfirming}
          Are you sure? Click again to delete
        {:else}
          Delete Patch
        {/if}
      </button>
    </div>
  {/if}
</aside>
