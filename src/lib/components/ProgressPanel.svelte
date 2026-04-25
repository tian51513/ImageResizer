<script lang="ts">
  import { progressStore } from "../stores/progress";
  import { formatFileSize, formatSavings } from "../utils/format";

  const { isProcessing, current, total, results, batchResult, percentage, totalSaved } = progressStore;
</script>

<div class="progress-panel">
  <div class="section-title">处理进度</div>

  {#if $isProcessing || $percentage > 0}
    <div class="progress-bar-container">
      <div class="progress-bar" style="width: {$percentage}%"></div>
    </div>
    <div class="progress-text">
      {$percentage}% ({$current}/{$total})
    </div>
  {/if}

  {#if $results.length > 0}
    <div class="results-table">
      <div class="table-header">
        <span class="col-file">文件</span>
        <span class="col-size">原始</span>
        <span class="col-size">压缩后</span>
        <span class="col-saving">节省</span>
      </div>
      <div class="table-body">
        {#each $results.slice(-50) as result (result.file)}
          <div class="table-row" class:failed={result.status !== "success"}>
            <span class="col-file" title={result.file}>
              {result.file.split(/[/\\]/).slice(-2).join("/")}
            </span>
            <span class="col-size">{formatFileSize(result.original_size)}</span>
            <span class="col-size">
              {result.status === "success" ? formatFileSize(result.new_size) : result.status}
            </span>
            <span class="col-saving" class:grew={result.new_size >= result.original_size && result.status === "success"}>
              {result.status === "success" ? formatSavings(result.original_size, result.new_size) : "-"}
            </span>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if $batchResult}
    <div class="summary">
      <div class="summary-item">
        <span>成功:</span>
        <strong>{$batchResult.success}</strong>
      </div>
      <div class="summary-item">
        <span>失败:</span>
        <strong class:has-failures={$batchResult.failed > 0}>
          {$batchResult.failed}
        </strong>
      </div>
      <div class="summary-item total-saved">
        <span>总计节省:</span>
        <strong>{formatFileSize($totalSaved)}</strong>
      </div>
    </div>
  {/if}
</div>

<style>
  .progress-panel {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .section-title {
    font-weight: 600;
    font-size: 13px;
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border-color);
  }
  .progress-bar-container {
    height: 20px;
    background: var(--border-color);
    border-radius: 10px;
    overflow: hidden;
  }
  .progress-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 10px;
    transition: width 0.3s ease;
  }
  .progress-text {
    font-size: 12px;
    color: var(--text-secondary);
    text-align: center;
  }
  .results-table {
    border: 1px solid var(--border-color);
    border-radius: 4px;
    max-height: 200px;
    overflow-y: auto;
  }
  .table-header {
    display: flex;
    font-size: 12px;
    font-weight: 600;
    padding: 4px 8px;
    background: var(--bg-primary);
    border-bottom: 1px solid var(--border-color);
    position: sticky;
    top: 0;
  }
  .table-body {
    max-height: 160px;
    overflow-y: auto;
  }
  .table-row {
    display: flex;
    font-size: 12px;
    padding: 3px 8px;
    border-bottom: 1px solid var(--border-color);
  }
  .table-row.failed {
    color: var(--error);
  }
  .table-row:last-child {
    border-bottom: none;
  }
  .col-file {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    margin-right: 8px;
  }
  .col-size {
    width: 70px;
    text-align: right;
  }
  .col-saving {
    width: 50px;
    text-align: right;
    color: var(--success);
  }
  .col-saving.grew {
    color: var(--warning);
  }
  .summary {
    display: flex;
    gap: 16px;
    font-size: 13px;
    padding: 8px 0;
  }
  .summary-item {
    display: flex;
    gap: 4px;
  }
  .total-saved {
    color: var(--success);
    font-weight: 600;
  }
  .has-failures {
    color: var(--error);
  }
</style>
