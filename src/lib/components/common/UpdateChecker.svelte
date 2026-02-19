<script>
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { listen } from "@tauri-apps/api/event";

  let visible = $state(false);
  let update = $state(null);
  let downloading = $state(false);
  let downloaded = $state(false);
  let progress = $state(0);
  let totalSize = $state(0);
  let error = $state(null);
  let upToDate = $state(false);
  let checkFailed = $state(false);

  // Automatic check on startup
  $effect(() => {
    const timer = setTimeout(() => checkForUpdate(false), 3000);
    return () => clearTimeout(timer);
  });

  // Listen for "Check for Updates..." from the native app menu
  $effect(() => {
    const unlisten = listen("menu:check-for-updates", () => checkForUpdate(true));
    return () => { unlisten.then(fn => fn()); };
  });

  async function checkForUpdate(isManual) {
    upToDate = false;
    checkFailed = false;
    try {
      const result = await check();
      if (result) {
        update = result;
        visible = true;
      } else if (isManual) {
        upToDate = true;
        setTimeout(() => { upToDate = false; }, 3000);
      }
    } catch (e) {
      if (isManual) {
        checkFailed = true;
        setTimeout(() => { checkFailed = false; }, 3000);
      }
      // Silently ignore automatic check failures (offline, misconfigured key, dev mode)
      console.debug("Update check skipped:", e);
    }
  }

  async function installUpdate() {
    if (!update) return;
    downloading = true;
    error = null;

    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          totalSize = event.data.contentLength ?? 0;
          progress = 0;
        } else if (event.event === "Progress") {
          progress += event.data.chunkLength;
        } else if (event.event === "Finished") {
          progress = totalSize;
        }
      });
      downloaded = true;
    } catch (e) {
      console.error("Update failed:", e);
      error = `Update failed: ${e}`;
    } finally {
      downloading = false;
    }
  }

  function dismiss() {
    visible = false;
  }

  let progressPercent = $derived(
    totalSize > 0 ? Math.min(100, Math.round((progress / totalSize) * 100)) : 0
  );
</script>

{#if upToDate}
  <div class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-surface border border-border rounded-lg px-4 py-2 shadow-lg text-sm text-text-secondary z-50">
    You're up to date
  </div>
{/if}

{#if checkFailed}
  <div class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-surface border border-border rounded-lg px-4 py-2 shadow-lg text-sm text-text-secondary z-50">
    Unable to check for updates
  </div>
{/if}

{#if visible && update}
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    role="dialog"
    aria-modal="true"
    aria-labelledby="update-title"
  >
    <div class="bg-surface rounded-lg shadow-xl w-full max-w-md mx-4 border border-border">
      <div class="p-4 border-b border-border">
        <h2 id="update-title" class="text-lg font-semibold">Update Available</h2>
        <p class="text-sm text-text-secondary mt-0.5">
          Version {update.version} is ready to install
          {#if update.currentVersion}
            <span class="text-text-secondary/60">(current: {update.currentVersion})</span>
          {/if}
        </p>
      </div>

      <div class="p-4">
        {#if update.body}
          <div class="mb-4 max-h-40 overflow-y-auto">
            <p class="text-xs text-text-secondary uppercase tracking-wide mb-1">Release Notes</p>
            <p class="text-sm text-text-primary whitespace-pre-wrap">{update.body}</p>
          </div>
        {/if}

        {#if error}
          <p class="text-red-400 text-sm mb-4">{error}</p>
        {/if}

        {#if downloading}
          <div class="mb-4">
            <div class="flex justify-between text-xs text-text-secondary mb-1">
              <span>Downloading...</span>
              <span>{progressPercent}%</span>
            </div>
            <div class="w-full bg-background rounded-full h-2">
              <div
                class="bg-primary h-2 rounded-full transition-all duration-200"
                style="width: {progressPercent}%"
              ></div>
            </div>
          </div>
        {/if}

        {#if downloaded}
          <p class="text-green-400 text-sm mb-4">
            Update downloaded. Restart the app to apply it.
          </p>
        {/if}

        <div class="flex justify-end gap-2">
          {#if !downloading && !downloaded}
            <button
              type="button"
              onclick={dismiss}
              class="px-4 py-2 rounded-lg hover:bg-border transition-colors text-sm"
            >
              Later
            </button>
            <button
              type="button"
              onclick={installUpdate}
              class="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors text-sm"
            >
              Install Update
            </button>
          {:else if downloaded}
            <button
              type="button"
              onclick={dismiss}
              class="px-4 py-2 rounded-lg hover:bg-border transition-colors text-sm"
            >
              Later
            </button>
            <button
              type="button"
              onclick={relaunch}
              class="px-4 py-2 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors text-sm"
            >
              Restart Now
            </button>
          {:else}
            <button
              type="button"
              disabled
              class="px-4 py-2 bg-primary/50 text-white rounded-lg text-sm cursor-not-allowed"
            >
              Downloading...
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}
