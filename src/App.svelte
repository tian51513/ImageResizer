<script lang="ts">
  import { onMount } from "svelte";
  import FileBrowser from "./lib/components/FileBrowser.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";
  import ProgressPanel from "./lib/components/ProgressPanel.svelte";
  import { filesStore } from "./lib/stores/files";
  import { profilesStore } from "./lib/stores/profiles";
  import { progressStore } from "./lib/stores/progress";

  const { filteredFiles, sourceDir } = filesStore;
  const { activeProfile } = profilesStore;
  const { isProcessing } = progressStore;

  let canStart = $state(false);

  $effect(() => {
    canStart =
      $filteredFiles.length > 0 &&
      $activeProfile !== null &&
      !$isProcessing;
  });

  async function handleStart() {
    const profile = $activeProfile;
    if (!profile || $filteredFiles.length === 0) return;

    await profilesStore.saveProfile(profile);
    await progressStore.startProcessing(
      $filteredFiles,
      profile,
      $sourceDir
    );
  }

  function handleStop() {
    progressStore.stopProcessing();
  }

  onMount(async () => {
    await profilesStore.loadProfiles();
  });
</script>

<div class="app-layout">
  <header class="app-header">
    <h1>ImageResizer</h1>
  </header>

  <div class="app-body">
    <aside class="left-panel">
      <FileBrowser />
    </aside>

    <div class="right-panel">
      <div class="settings-area">
        <SettingsPanel />
      </div>

      <div class="progress-area">
        <ProgressPanel />
      </div>

      <div class="action-bar">
        <button
          class="btn-stop"
          onclick={handleStop}
          disabled={!$isProcessing}
        >
          ⏹ 停止
        </button>
        <button
          class="btn-start"
          onclick={handleStart}
          disabled={!canStart}
        >
          ▶ 开始压缩
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  .app-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }
  .app-header h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }
  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .left-panel {
    width: 280px;
    min-width: 200px;
    border-right: 1px solid var(--border-color);
    padding: 12px;
    overflow-y: auto;
  }
  .right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 12px;
    overflow-y: auto;
  }
  .settings-area {
    flex-shrink: 0;
  }
  .progress-area {
    flex: 1;
    overflow-y: auto;
  }
  .action-bar {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 0 0;
    border-top: 1px solid var(--border-color);
    margin-top: 8px;
  }
  button {
    padding: 8px 20px;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    border: 1px solid var(--border-color);
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn-stop {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }
  .btn-start {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  .btn-start:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>
