<script lang="ts">
  import { profilesStore } from "../stores/profiles";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { OutputOperation, OutputFormat, NamingMode } from "../types";

  const { activeProfile } = profilesStore;

  const operations: { label: string; value: OutputOperation }[] = [
    { label: "重新调整原始文件", value: "Overwrite" },
    { label: "输出到与原始文件同目录", value: "SameDir" },
    { label: "输出到自定义目录", value: "CustomDir" },
  ];

  const formats: { label: string; value: OutputFormat }[] = [
    { label: "与原文件相同", value: "SameAsOriginal" },
    { label: "JPEG (.jpg)", value: "Jpeg" },
    { label: "PNG (.png)", value: "Png" },
    { label: "WebP (.webp)", value: "WebP" },
    { label: "GIF (.gif)", value: "Gif" },
  ];

  const namingModes: { label: string; value: NamingMode; desc: string }[] = [
    { label: "保持原文件名", value: "KeepOriginal", desc: "输出文件名与原文件相同（格式不同时无需后缀）" },
    { label: "添加日期后缀", value: "DateSuffix", desc: "如 photo_20260425.jpg" },
    { label: "自定义后缀", value: "CustomSuffix", desc: "如 photo_min.jpg" },
  ];

  function updateOutput(partial: Record<string, any>) {
    const profile = $activeProfile;
    if (!profile) return;
    profilesStore.updateActiveProfile({
      ...profile,
      output: { ...profile.output, ...partial },
    });
  }

  async function pickCustomDir() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择输出目录",
    });
    if (selected) {
      updateOutput({ custom_dir: selected as string });
    }
  }
</script>

{#if $activeProfile}
  <div class="output-settings">
    <div class="section-title">输出设置</div>
    <div class="field-row">
      <label>操作:</label>
      <select
        value={$activeProfile.output.operation}
        onchange={(e) => updateOutput({ operation: (e.target as HTMLSelectElement).value })}
      >
        {#each operations as op}
          <option value={op.value}>{op.label}</option>
        {/each}
      </select>
    </div>
    {#if $activeProfile.output.operation === "CustomDir"}
      <div class="field-row">
        <label>输出目录:</label>
        <input
          type="text"
          readonly
          value={$activeProfile.output.custom_dir || "点击选择..."}
          onclick={pickCustomDir}
          class="dir-input"
        />
      </div>
    {/if}
    <div class="field-row">
      <label>格式:</label>
      <select
        value={$activeProfile.output.format}
        onchange={(e) => updateOutput({ format: (e.target as HTMLSelectElement).value })}
      >
        {#each formats as f}
          <option value={f.value}>{f.label}</option>
        {/each}
      </select>
    </div>
    {#if $activeProfile.output.operation !== "Overwrite"}
      <div class="field-row">
        <label>命名:</label>
        <select
          value={$activeProfile.output.naming}
          onchange={(e) => updateOutput({ naming: (e.target as HTMLSelectElement).value })}
        >
          {#each namingModes as nm}
            <option value={nm.value} title={nm.desc}>{nm.label}</option>
          {/each}
        </select>
      </div>
      {#if $activeProfile.output.naming === "CustomSuffix"}
        <div class="field-row">
          <label>后缀:</label>
          <input
            type="text"
            placeholder="如 _min、_compressed"
            value={$activeProfile.output.custom_suffix ?? ""}
            oninput={(e) => updateOutput({ custom_suffix: (e.target as HTMLInputElement).value || null })}
            class="suffix-input"
          />
        </div>
      {/if}
    {/if}
  </div>
{/if}

<style>
  .output-settings {
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
  select {
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
    background: var(--bg-secondary);
    flex: 1;
  }
  input.suffix-input {
    flex: 1;
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
    background: var(--bg-secondary);
  }
  .dir-input {
    flex: 1;
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 12px;
    cursor: pointer;
    background: var(--bg-secondary);
  }
</style>
