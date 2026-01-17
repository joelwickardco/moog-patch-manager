<script>
  let { activeTab = $bindable() } = $props();

  const tabs = [
    { id: "library", label: "Library", icon: "folder" },
    { id: "banks", label: "Banks", icon: "bank" },
    { id: "categories", label: "Categories", icon: "tag" },
  ];
</script>

<aside class="w-64 bg-surface border-r border-border flex flex-col">
  <div class="p-4 border-b border-border">
    <h1 class="text-xl font-bold text-primary">Moog Muse</h1>
    <p class="text-sm text-text-secondary">Patch Manager</p>
  </div>

  <nav class="flex-1 p-2">
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
      <h3 class="px-4 text-xs font-semibold text-text-secondary uppercase tracking-wider mb-2">
        Quick Filters
      </h3>
      <button class="w-full text-left px-4 py-2 rounded-lg hover:bg-border text-text-secondary">
        Favorites
      </button>
      <button class="w-full text-left px-4 py-2 rounded-lg hover:bg-border text-text-secondary">
        All Patches
      </button>
    </div>
  </nav>

  <div class="p-4 border-t border-border space-y-2">
    <button class="w-full px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors">
      Import
    </button>
    <button class="w-full px-4 py-2 bg-secondary text-white rounded-lg hover:bg-secondary/90 transition-colors">
      Export
    </button>
  </div>
</aside>
