<script>
  import TagInput from "./TagInput.svelte";
  import { updatePatchTags, updatePatchNotes, getAllTags } from "../../utils/api.js";

  let {
    open = false,
    patch = null,
    onClose = () => {},
    onSaved = () => {}
  } = $props();

  let editedTags = $state([]);
  let editedNotes = $state("");
  let availableTags = $state([]);
  let saving = $state(false);
  let error = $state(null);

  // Load tags when modal opens
  $effect(() => {
    if (open && patch) {
      editedTags = [...(patch.tags || [])];
      editedNotes = patch.notes || "";
      loadTags();
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

  async function handleSave() {
    if (!patch) return;

    saving = true;
    error = null;

    try {
      // Update tags
      await updatePatchTags(patch.id, editedTags);

      // Update notes
      await updatePatchNotes(patch.id, editedNotes);

      // Update the patch object
      patch.tags = [...editedTags];
      patch.notes = editedNotes;

      onSaved();
      onClose();
    } catch (e) {
      error = e.toString();
      console.error("Failed to save patch:", e);
    } finally {
      saving = false;
    }
  }

  function handleCancel() {
    error = null;
    onClose();
  }
</script>

{#if open && patch}
  <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4" role="presentation" onclick={handleCancel} onkeydown={(e) => e.key === 'Escape' && handleCancel()}>
    <div
      class="bg-background rounded-lg max-w-2xl w-full max-h-[90vh] overflow-y-auto shadow-xl"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
      onkeydown={(e) => e.stopPropagation()}
    >
      <div class="p-6">
        <h2 class="text-2xl font-semibold mb-4">Edit Patch</h2>

        <div class="space-y-4">
          <!-- Patch Name (read-only) -->
          <div>
            <label for="patch-name" class="block text-sm font-medium mb-1">Name</label>
            <input
              id="patch-name"
              type="text"
              value={patch.name}
              disabled
              class="w-full px-3 py-2 bg-surface border border-border rounded-lg opacity-50"
            />
          </div>

          <!-- Tags -->
          <div>
            <div class="block text-sm font-medium mb-1">Tags</div>
            <TagInput
              bind:selectedTags={editedTags}
              {availableTags}
              placeholder="Add tags..."
            />
          </div>

          <!-- Notes -->
          <div>
            <label for="patch-notes" class="block text-sm font-medium mb-1">Notes</label>
            <textarea
              id="patch-notes"
              bind:value={editedNotes}
              class="w-full px-3 py-2 bg-surface border border-border rounded-lg resize-none"
              rows="4"
              placeholder="Add notes about this patch..."
            ></textarea>
          </div>

          {#if error}
            <div class="p-3 bg-red-500/20 border border-red-500 rounded-lg text-red-400 text-sm">
              {error}
            </div>
          {/if}
        </div>

        <div class="flex gap-2 mt-6">
          <button
            onclick={handleSave}
            disabled={saving}
            class="flex-1 px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            {saving ? "Saving..." : "Save"}
          </button>
          <button
            onclick={handleCancel}
            disabled={saving}
            class="flex-1 px-4 py-2 bg-surface rounded-lg hover:bg-border disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
