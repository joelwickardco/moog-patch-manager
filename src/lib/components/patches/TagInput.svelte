<script>
  let {
    selectedTags = $bindable([]),
    availableTags = [],
    placeholder = "Add tags...",
    onTagsChange = () => {}
  } = $props();

  let inputValue = $state("");
  let showDropdown = $state(false);
  let inputElement;

  // Filter available tags based on input and exclude already selected
  let filteredTags = $derived(
    inputValue.trim()
      ? availableTags
          .filter(tag =>
            tag.toLowerCase().includes(inputValue.toLowerCase()) &&
            !selectedTags.includes(tag)
          )
          .slice(0, 10) // Limit to 10 suggestions
      : []
  );

  function addTag(tag) {
    if (tag && !selectedTags.includes(tag)) {
      selectedTags = [...selectedTags, tag];
      onTagsChange(selectedTags);
      inputValue = "";
      showDropdown = false;
    }
  }

  function removeTag(tagToRemove) {
    selectedTags = selectedTags.filter(tag => tag !== tagToRemove);
    onTagsChange(selectedTags);
  }

  function handleKeyDown(e) {
    if (e.key === "Enter" && inputValue.trim()) {
      e.preventDefault();
      const trimmedValue = inputValue.trim();

      // If there's a filtered suggestion, use the first one
      if (filteredTags.length > 0) {
        addTag(filteredTags[0]);
      } else {
        // Otherwise, create new tag
        addTag(trimmedValue);
      }
    } else if (e.key === "Backspace" && !inputValue && selectedTags.length > 0) {
      // Remove last tag when backspace on empty input
      removeTag(selectedTags[selectedTags.length - 1]);
    } else if (e.key === "Escape") {
      showDropdown = false;
      inputValue = "";
    }
  }

  function handleInput() {
    showDropdown = inputValue.trim().length > 0;
  }

  function handleBlur() {
    // Delay to allow click on dropdown items
    setTimeout(() => {
      showDropdown = false;
    }, 200);
  }
</script>

<div class="relative">
  <div class="flex flex-wrap gap-1 p-2 bg-surface rounded-lg border border-border focus-within:border-primary transition-colors">
    {#each selectedTags as tag}
      <span class="inline-flex items-center gap-1 text-xs px-2 py-1 rounded bg-primary/20 text-primary">
        {tag}
        <button
          type="button"
          onclick={() => removeTag(tag)}
          class="hover:text-red-400 transition-colors"
          title="Remove tag"
        >
          ×
        </button>
      </span>
    {/each}

    <input
      bind:this={inputElement}
      bind:value={inputValue}
      oninput={handleInput}
      onkeydown={handleKeyDown}
      onblur={handleBlur}
      onfocus={() => showDropdown = inputValue.trim().length > 0}
      type="text"
      class="flex-1 min-w-[120px] bg-transparent outline-none text-sm"
      placeholder={selectedTags.length === 0 ? placeholder : ""}
    />
  </div>

  {#if showDropdown && filteredTags.length > 0}
    <div class="absolute z-10 w-full mt-1 bg-surface border border-border rounded-lg shadow-lg max-h-48 overflow-y-auto">
      {#each filteredTags as tag}
        <button
          type="button"
          class="w-full text-left px-3 py-2 text-sm hover:bg-primary/10 transition-colors"
          onclick={() => addTag(tag)}
        >
          {tag}
        </button>
      {/each}
    </div>
  {/if}
</div>
