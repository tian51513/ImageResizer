<script lang="ts">
  import { profilesStore } from "../stores/profiles";
  import type { SizeUnit, ResizeMode } from "../types";

  const { activeProfile } = profilesStore;

  const units: { label: string; value: SizeUnit }[] = [
    { label: "百分比", value: "Percentage" },
    { label: "像素", value: "Pixel" },
  ];

  const modes: { label: string; value: ResizeMode }[] = [
    { label: "合适大小", value: "Fit" },
    { label: "填充", value: "Fill" },
    { label: "拉伸", value: "Stretch" },
    { label: "仅缩小", value: "ShrinkOnly" },
  ];

  function updateResize(partial: Record<string, any>) {
    const profile = $activeProfile;
    if (!profile) return;
    profilesStore.updateActiveProfile({
      ...profile,
      resize: { ...profile.resize, ...partial },
    });
  }
</script>

{#if $activeProfile}
  <div class="resize-settings">
    <div class="section-title">调整大小</div>
    <div class="field-row">
      <label>宽度:</label>
      <input
        type="number"
        min="1"
        max="10000"
        value={$activeProfile.resize.width}
        oninput={(e) => updateResize({ width: Number((e.target as HTMLInputElement).value) })}
      />
      <select
        value={$activeProfile.resize.unit}
        onchange={(e) => updateResize({ unit: (e.target as HTMLSelectElement).value })}
      >
        {#each units as u}
          <option value={u.value}>{u.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label>高度:</label>
      <input
        type="number"
        min="1"
        max="10000"
        value={$activeProfile.resize.height}
        oninput={(e) => updateResize({ height: Number((e.target as HTMLInputElement).value) })}
      />
      <select
        value={$activeProfile.resize.unit}
        onchange={(e) => updateResize({ unit: (e.target as HTMLSelectElement).value })}
      >
        {#each units as u}
          <option value={u.value}>{u.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label>模式:</label>
      <select
        value={$activeProfile.resize.mode}
        onchange={(e) => updateResize({ mode: (e.target as HTMLSelectElement).value })}
      >
        {#each modes as m}
          <option value={m.value}>{m.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label>
        <input
          type="checkbox"
          checked={$activeProfile.resize.keep_aspect_ratio}
          onchange={(e) => updateResize({ keep_aspect_ratio: (e.target as HTMLInputElement).checked })}
        />
        保持宽高比
      </label>
    </div>
  </div>
{/if}

<style>
  .resize-settings {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .section-title {
    font-weight: 600;
    font-size: 13px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border-color);
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  label {
    min-width: 48px;
  }
  input[type="number"] {
    width: 70px;
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
  }
  select {
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
    background: var(--bg-secondary);
  }
  input[type="checkbox"] {
    margin-right: 4px;
  }
</style>
