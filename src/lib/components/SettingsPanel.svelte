<script lang="ts">
  import ProfileSelector from "./ProfileSelector.svelte";
  import ResizeSettings from "./ResizeSettings.svelte";
  import OutputSettings from "./OutputSettings.svelte";
  import QualitySettings from "./QualitySettings.svelte";
</script>

<div class="settings-panel">
  <ProfileSelector />
  <div class="settings-body">
    <ResizeSettings />
    <OutputSettings />
    <QualitySettings />
  </div>
</div>

<style>
  .settings-panel {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .settings-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-left: 4px;
  }
</style>
