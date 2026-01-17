<script>
  let {
    activeTab = $bindable(),
    libraries = [],
    selectedLibraryId = $bindable(),
    onImport = () => {},
    importing = false,
    onExport = () => {},
    exporting = false,
    onNewLibrary = () => {}
  } = $props();

  const tabs = [
    { id: "library", label: "Library", icon: "folder" },
    { id: "banks", label: "Banks", icon: "bank" },
    { id: "categories", label: "Categories", icon: "tag" },
  ];

  function selectLibrary(libraryId) {
    selectedLibraryId = libraryId;
    activeTab = "library";
  }

  function getLibraryColor(library) {
    return library.color || "#6B7280";
  }
</script>

<aside class="w-64 bg-surface border-r border-border flex flex-col">
  <div class="p-4 border-b border-border">
    <h1 class="text-xl font-bold text-primary">Moog Muse</h1>
    <p class="text-sm text-text-secondary">Patch Manager</p>
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
        <button
          class="w-full text-left px-4 py-2 rounded-lg transition-colors flex items-center gap-2
            {selectedLibraryId === library.id && activeTab === 'library'
              ? 'bg-primary/20 text-primary'
              : 'hover:bg-border text-text-secondary'}"
          onclick={() => selectLibrary(library.id)}
        >
          <span
            class="w-3 h-3 rounded-full flex-shrink-0"
            style="background-color: {getLibraryColor(library)}"
          ></span>
          <span class="truncate">{library.name}</span>
          <span class="ml-auto text-xs text-text-secondary flex-shrink-0">
            {library.patch_count}
          </span>
        </button>
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
