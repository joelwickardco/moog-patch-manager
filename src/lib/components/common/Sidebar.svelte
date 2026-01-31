<script>
  import { updateLibrary } from "../../utils/api.js";
  import version from "../../../../version.txt?raw";

  let {
    activeTab = $bindable(),
    libraries = [],
    selectedLibraryId = $bindable(),
    onImport = () => {},
    importing = false,
    onExport = () => {},
    exporting = false,
    onNewLibrary = () => {},
    onLibraryNameUpdate = () => {}
  } = $props();

  let editingLibraryId = $state(null);
  let editedLibraryName = $state("");
  let nameInputElement = $state(null);
  let renameError = $state(null);

  const tabs = [
    { id: "library", label: "Library", icon: "folder" },
    { id: "banks", label: "Banks", icon: "bank" },
  ];

  function selectLibrary(libraryId) {
    selectedLibraryId = libraryId;
    activeTab = "library";
  }

  function getLibraryColor(library) {
    return library.color || "#6B7280";
  }

  function startEditingLibraryName(library) {
    editedLibraryName = library.name;
    editingLibraryId = library.id;
    renameError = null;
    // Focus input after it's rendered
    setTimeout(() => {
      if (nameInputElement) {
        nameInputElement.focus();
        nameInputElement.select();
      }
    }, 0);
  }

  async function saveLibraryName() {
    if (!editingLibraryId || !editedLibraryName.trim()) {
      editingLibraryId = null;
      renameError = null;
      return;
    }

    try {
      await updateLibrary(editingLibraryId, editedLibraryName.trim());
      editingLibraryId = null;
      renameError = null;
      onLibraryNameUpdate();
    } catch (e) {
      // Show error inline instead of canceling
      renameError = e.toString();
      // Keep editing mode active so user can fix the name
    }
  }

  function cancelLibraryEdit() {
    editingLibraryId = null;
    editedLibraryName = "";
    renameError = null;
  }

  function handleLibraryNameKeydown(e) {
    if (e.key === "Enter") {
      saveLibraryName();
    } else if (e.key === "Escape") {
      cancelLibraryEdit();
    }
  }
</script>

<aside class="w-64 bg-surface border-r border-border flex flex-col">
  <div class="p-4 border-b border-border">
    <h1 class="text-xl font-bold text-primary">Moog Muse</h1>
    <p class="text-sm text-text-secondary">
      Patch Manager <span class="text-xs opacity-70">v{version.trim()}</span>
    </p>
  </div>

  <nav class="flex-1 p-2 overflow-y-auto">
    {#each tabs as tab}
      <button
        class="w-full text-left px-4 py-3 rounded-lg mb-1 transition-colors {activeTab === tab.id
          ? 'bg-primary/20 text-primary'
          : 'hover:bg-border text-text-secondary'}"
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
      </button>
    {/each}

    <div class="mt-4 pt-4 border-t border-border">
      <div class="flex items-center justify-between px-4 mb-2">
        <h3 class="text-xs font-semibold text-text-secondary uppercase tracking-wider">
          Libraries
        </h3>
        <button
          onclick={onNewLibrary}
          class="w-5 h-5 flex items-center justify-center rounded hover:bg-border text-text-secondary hover:text-primary transition-colors"
          title="Create new library"
        >
          <span class="text-lg leading-none">+</span>
        </button>
      </div>
      <button
        class="w-full text-left px-4 py-2 rounded-lg transition-colors flex items-center gap-2
          {selectedLibraryId === null && activeTab === 'library'
            ? 'bg-primary/20 text-primary'
            : 'hover:bg-border text-text-secondary'}"
        onclick={() => selectLibrary(null)}
      >
        <span class="w-3 h-3 rounded-full bg-gray-500"></span>
        All Libraries
        <span class="ml-auto text-xs text-text-secondary">
          {libraries.reduce((sum, l) => sum + l.patch_count, 0)}
        </span>
      </button>
      {#each libraries as library}
        <div class="w-full px-4 py-2 rounded-lg transition-colors flex items-center gap-2 group
          {selectedLibraryId === library.id && activeTab === 'library'
            ? 'bg-primary/20 text-primary'
            : 'hover:bg-border text-text-secondary'}">

          <span
            class="w-3 h-3 rounded-full flex-shrink-0"
            style="background-color: {getLibraryColor(library)}"
          ></span>

          {#if editingLibraryId === library.id}
            <div class="flex-1 min-w-0 flex flex-col gap-1" role="presentation" onclick={(e) => e.stopPropagation()} onkeydown={(e) => e.key === 'Enter' && e.stopPropagation()}>
              <input
                type="text"
                bind:value={editedLibraryName}
                bind:this={nameInputElement}
                onkeydown={handleLibraryNameKeydown}
                onblur={saveLibraryName}
                class="w-full bg-background border border-primary rounded px-2 py-0.5 text-sm outline-none focus:ring-2 focus:ring-primary/50"
                placeholder="Library name"
              />
              {#if renameError}
                <div class="text-xs text-red-400">{renameError}</div>
              {/if}
            </div>
          {:else}
            <button
              class="min-w-0 flex-1 flex items-center gap-2"
              onclick={() => selectLibrary(library.id)}
            >
              <span class="truncate">{library.name}</span>
            </button>
          {/if}

          <span class="ml-auto text-xs text-text-secondary flex-shrink-0">
            {library.patch_count}
          </span>

          {#if editingLibraryId !== library.id}
            <button
              onclick={(e) => {
                e.stopPropagation();
                startEditingLibraryName(library);
              }}
              class="opacity-0 group-hover:opacity-100 transition-opacity p-1 hover:bg-border rounded flex-shrink-0"
              title="Rename library"
            >
              <span class="text-xs">✏️</span>
            </button>
          {/if}
        </div>
      {/each}
    </div>

    <div class="mt-4 pt-4 border-t border-border">
      <h3 class="px-4 text-xs font-semibold text-text-secondary uppercase tracking-wider mb-2">
        Quick Filters
      </h3>
      <button class="w-full text-left px-4 py-2 rounded-lg hover:bg-border text-text-secondary">
        Favorites
      </button>
    </div>
  </nav>

  <div class="p-4 border-t border-border space-y-2">
    <button
      class="w-full px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      onclick={onImport}
      disabled={importing}
    >
      {importing ? "Importing..." : "Import"}
    </button>
    <button
      class="w-full px-4 py-2 bg-secondary text-white rounded-lg hover:bg-secondary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      onclick={onExport}
      disabled={exporting || selectedLibraryId === null}
      title={selectedLibraryId === null ? "Select a library to export" : "Export selected library"}
    >
      {exporting ? "Exporting..." : "Export"}
    </button>
  </div>
</aside>
