<script lang="ts">
  import { profilesStore } from "../stores/profiles";
  import type { QualityMode } from "../types";

  const { activeProfile } = profilesStore;
</script>

{#if $activeProfile}
  <div class="quality-settings">
    <div class="section-title">品质与格式</div>
    <div class="quality-mode-row">
      <label>
        <input
          type="radio"
          name="quality-mode"
          checked={$activeProfile.quality.mode === "Quality"}
          onchange={() => {
            const profile = $activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: { ...profile.quality, mode: "Quality" as QualityMode },
            });
          }}
        />
        品质
      </label>
      <label>
        <input
          type="radio"
          name="quality-mode"
          checked={$activeProfile.quality.mode === "TargetSize"}
          onchange={() => {
            const profile = $activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: { ...profile.quality, mode: "TargetSize" as QualityMode },
            });
          }}
        />
        目标大小
      </label>
      <label>
        <input
          type="checkbox"
          checked={$activeProfile.quality.mode === "Original"}
          onchange={(e) => {
            const profile = $activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                mode: (e.target as HTMLInputElement).checked ? "Original" as QualityMode : "Quality" as QualityMode,
              },
            });
          }}
        />
        保持原始品质
      </label>
    </div>

    {#if $activeProfile.quality.mode === "Quality"}
      <div class="field-row">
        <label>品质:</label>
        <input
          type="range"
          min="1"
          max="100"
          value={$activeProfile.quality.quality}
          oninput={(e) => {
            const profile = $activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                quality: Number((e.target as HTMLInputElement).value),
              },
            });
          }}
        />
        <span class="value">{$activeProfile.quality.quality}%</span>
      </div>
    {/if}

    {#if $activeProfile.quality.mode === "TargetSize"}
      <div class="field-row">
        <label>大小:</label>
        <input
          type="number"
          min="1"
          value={$activeProfile.quality.target_size_kb || 100}
          oninput={(e) => {
            const profile = $activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                target_size_kb: Number((e.target as HTMLInputElement).value),
              },
            });
          }}
        />
        <span class="value">KB</span>
      </div>
    {/if}

    <div class="field-row">
      <label>
        <input
          type="checkbox"
          checked={$activeProfile.quality.adjust_dpi}
          onchange={(e) => {
            const profile = $activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                adjust_dpi: (e.target as HTMLInputElement).checked,
              },
            });
          }}
        />
        调整分辨率
      </label>
      {#if $activeProfile.quality.adjust_dpi}
        <input
          type="number"
          min="1"
          max="2400"
          value={$activeProfile.quality.dpi}
          oninput={(e) => {
            const profile = $activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                dpi: Number((e.target as HTMLInputElement).value),
              },
            });
          }}
        />
        <span class="value">DPI</span>
      {/if}
    </div>
  </div>
{/if}

<style>
  .quality-settings {
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
  .quality-mode-row {
    display: flex;
    gap: 12px;
    font-size: 13px;
    flex-wrap: wrap;
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  label {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  input[type="radio"], input[type="checkbox"] {
    margin: 0;
  }
  input[type="range"] {
    flex: 1;
    max-width: 200px;
  }
  input[type="number"] {
    width: 70px;
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
  }
  .value {
    color: var(--text-secondary);
    font-size: 12px;
    min-width: 30px;
  }
</style>
