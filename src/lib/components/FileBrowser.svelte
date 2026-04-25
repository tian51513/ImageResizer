<script lang="ts">
  import { filesStore } from "../stores/files";
  import { formatFileSize } from "../utils/format";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { ExtensionFilter } from "../types";

  const { sourceDir, filter, loading, error, filteredFiles, totalSize } = filesStore;

  const filterOptions: { label: string; value: ExtensionFilter }[] = [
    { label: "全部", value: "all" },
    { label: "JPG", value: "jpg" },
    { label: "PNG", value: "png" },
    { label: "WebP", value: "webp" },
    { label: "GIF", value: "gif" },
  ];

  async function pickDirectory() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "选择漫画目录",
    });
    if (selected) {
      await filesStore.scanDirectory(selected as string);
    }
  }

  function onFilterChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    filesStore.filter.set(target.value as ExtensionFilter);
  }
</script>

<div class="file-browser">
  <div class="section-header">源目录</div>
  <button class="dir-picker" onclick={pickDirectory}>
    📁 选择目录...
  </button>
  {#if $sourceDir}
    <div class="source-dir">{$sourceDir}</div>
  {/if}

  {#if $loading}
    <div class="loading">扫描中...</div>
  {:else if $error}
    <div class="error">{$error}</div>
  {:else if $filteredFiles.length > 0}
    <div class="toolbar">
      <span class="file-count">共 {$filteredFiles.length} 个文件</span>
      <select onchange={onFilterChange} value={$filter}>
        {#each filterOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>
    <div class="total-size">
      总大小: {formatFileSize($totalSize)}
    </div>
    <div class="file-list">
      {#each $filteredFiles.slice(0, 200) as file (file.path)}
        <div class="file-item">
          <span class="file-name" title={file.path}>{file.path.split(/[/\\]/).pop()}</span>
          <span class="file-size">{formatFileSize(file.size_bytes)}</span>
        </div>
      {/each}
      {#if $filteredFiles.length > 200}
        <div class="file-more">... 还有 {$filteredFiles.length - 200} 个文件</div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .file-browser {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-header {
    font-weight: 600;
    font-size: 14px;
    color: var(--text-primary);
  }
  .dir-picker {
    padding: 8px 12px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-secondary);
    cursor: pointer;
    text-align: left;
    font-size: 13px;
  }
  .dir-picker:hover {
    border-color: var(--accent);
  }
  .source-dir {
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .loading, .error {
    padding: 8px;
    font-size: 13px;
  }
  .error {
    color: var(--error);
  }
  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 12px;
  }
  .file-count {
    color: var(--text-secondary);
  }
  .total-size {
    font-size: 12px;
    color: var(--text-secondary);
  }
  .file-list {
    flex: 1;
    overflow-y: auto;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    max-height: 300px;
  }
  .file-item {
    display: flex;
    justify-content: space-between;
    padding: 4px 8px;
    font-size: 12px;
    border-bottom: 1px solid var(--border-color);
  }
  .file-item:last-child {
    border-bottom: none;
  }
  .file-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
    margin-right: 8px;
  }
  .file-size {
    color: var(--text-secondary);
    white-space: nowrap;
  }
  .file-more {
    padding: 4px 8px;
    font-size: 12px;
    color: var(--text-secondary);
    text-align: center;
  }
</style>
