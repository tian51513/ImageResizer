<script lang="ts">
  import { progressStore } from "../stores/progress";
  import { formatFileSize, formatSavings } from "../utils/format";

  const { isProcessing, current, total, percentage, bytePercentage, totalOriginalBytes, processedBytes, results, batchResult, totalSaved } = progressStore;
</script>

<div class="progress-panel">
  <div class="section-title">处理进度</div>

  {#if $isProcessing || $percentage > 0}
    <div class="progress-group">
      <div class="progress-row">
        <span class="progress-label">文件进度:</span>
        <span class="progress-value">{$current}/{$total}</span>
      </div>
      <div class="progress-bar-container">
        <div class="progress-bar" style="width: {$percentage}%"></div>
      </div>
    </div>

    <div class="progress-group">
      <div class="progress-row">
        <span class="progress-label">总任务进度:</span>
        <span class="progress-value">{$bytePercentage}% · {formatFileSize($processedBytes)} / {formatFileSize($totalOriginalBytes)}</span>
      </div>
      <div class="progress-bar-container">
        <div class="progress-bar progress-bar-total" style="width: {$bytePercentage}%"></div>
      </div>
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
      {#each $results.slice(-100) as result (result.file)}
        <div class="table-row" class:failed={result.status !== "success"} class:skipped={result.status === "skipped"}>
          <span class="col-file" title={result.file}>
            {result.file.split(/[/\\]/).slice(-2).join("/")}
          </span>
          <span class="col-size">{formatFileSize(result.original_size)}</span>
          <span class="col-size" class:status-cell={result.status !== "success"} title={result.status !== "success" ? result.status : ''}>
            {result.status === "success" ? formatFileSize(result.new_size) : result.status}
          </span>
          <span class="col-saving" class:grew={result.new_size >= result.original_size && result.status === "success"}>
            {result.status === "success" ? formatSavings(result.original_size, result.new_size) : "-"}
          </span>
        </div>
      {/each}
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
    {#if $batchResult.failed > 0}
      <div class="log-hint" title="日志文件路径">
        日志路径: {exe所在目录}\logs\{日期}.log
      </div>
    {/if}
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
  .progress-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .progress-row {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--text-secondary);
  }
  .progress-value {
    font-variant-numeric: tabular-nums;
  }
  .progress-bar-container {
    height: 16px;
    background: var(--border-color);
    border-radius: 8px;
    overflow: hidden;
  }
  .progress-bar {
    height: 100%;
    background: var(--accent);
    border-radius: 8px;
    transition: width 0.3s ease;
  }
  .progress-bar-total {
    background: #4ade80;
  }
  .results-table {
    border: 1px solid var(--border-color);
    border-radius: 4px;
    max-height: 300px;
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
    z-index: 1;
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
  .table-row.skipped {
    color: var(--text-secondary);
  }
  .col-size.status-cell {
    cursor: help;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
  .log-hint {
    font-size: 11px;
    color: var(--text-secondary);
    padding: 4px 0;
    cursor: default;
    word-break: break-all;
  }
</style>
