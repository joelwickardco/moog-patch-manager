<script>
  let {
    open = false,
    onClose = () => {},
    onSubmit = () => {}
  } = $props();

  let libraryName = $state("");
  let error = $state("");
  let inputElement = $state(null);

  // Focus input when modal opens
  $effect(() => {
    if (open && inputElement) {
      libraryName = "";
      error = "";
      setTimeout(() => inputElement?.focus(), 50);
    }
  });

  function handleSubmit(e) {
    e.preventDefault();

    const trimmedName = libraryName.trim();
    if (!trimmedName) {
      error = "Library name is required";
      return;
    }

    onSubmit(trimmedName);
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

{#if open}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    tabindex="-1"
    onclick={handleBackdropClick}
    onkeydown={handleKeydown}
  >
    <div class="bg-surface rounded-lg shadow-xl w-full max-w-md mx-4 border border-border">
      <div class="p-4 border-b border-border">
        <h2 id="modal-title" class="text-lg font-semibold">New Library</h2>
      </div>

      <form onsubmit={handleSubmit} class="p-4">
        <label class="block mb-4">
          <span class="text-sm text-text-secondary mb-1 block">Library Name</span>
          <input
            bind:this={inputElement}
            bind:value={libraryName}
            type="text"
            placeholder="Enter library name..."
            class="w-full px-3 py-2 bg-background border border-border rounded-lg
                   focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary
                   placeholder:text-text-secondary/50"
          />
        </label>

        {#if error}
          <p class="text-red-400 text-sm mb-4">{error}</p>
        {/if}

        <div class="flex justify-end gap-2">
          <button
            type="button"
            onclick={onClose}
            class="px-4 py-2 rounded-lg hover:bg-border transition-colors"
          >
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors"
          >
            Create
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
