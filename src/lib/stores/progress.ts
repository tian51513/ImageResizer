import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ProgressBatch, BatchResult, FileMetadata, Profile, ProcessResult } from "../types";

function createProgressStore() {
  const isProcessing = writable<boolean>(false);
  const current = writable<number>(0);
  const total = writable<number>(0);
  const totalOriginalBytes = writable<number>(0);
  const processedBytes = writable<number>(0);
  const results = writable<ProcessResult[]>([]);
  const lastError = writable<string | null>(null);
  const batchResult = writable<BatchResult | null>(null);

  const percentage = derived(
    [current, total],
    ([$current, $total]) =>
      $total > 0 ? Math.round(($current / $total) * 100) : 0
  );

  const bytePercentage = derived(
    [processedBytes, totalOriginalBytes],
    ([$processed, $totalBytes]) =>
      $totalBytes > 0 ? Math.round(($processed / $totalBytes) * 100) : 0
  );

  const totalSaved = derived(batchResult, ($result) =>
    $result ? $result.total_saved_bytes : 0
  );

  let unlistenProgress: (() => void) | null = null;
  let unlistenComplete: (() => void) | null = null;

  async function startProcessing(
    files: FileMetadata[],
    profile: Profile,
    sourceDir: string
  ) {
    // 先取消上一次的事件监听器，防止重复注册
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
    if (unlistenComplete) {
      unlistenComplete();
      unlistenComplete = null;
    }

    results.set([]);
    current.set(0);
    total.set(files.length);
    totalOriginalBytes.set(0);
    processedBytes.set(0);
    batchResult.set(null);
    isProcessing.set(true);

    unlistenProgress = await listen<ProgressBatch>("progress_update", (e) => {
      const { last, results: rows } = e.payload;
      current.set(last.current);
      total.set(last.total);
      totalOriginalBytes.set(last.total_original_bytes);
      processedBytes.set(last.processed_bytes);
      // append in place — no full-array rebuild per event
      results.update((r) => {
        r.push(...rows);
        return r;
      });
    });

    unlistenComplete = await listen<BatchResult>("processing_complete", (e) => {
      batchResult.set(e.payload);
      isProcessing.set(false);
    });

    try {
      await invoke("start_processing", {
        files,
        profile,
        sourceDir,
      });
    } catch (e) {
      isProcessing.set(false);
      lastError.set(String(e));
      console.error("Failed to start processing:", e);
    }
  }

  async function stopProcessing() {
    try {
      await invoke("stop_processing");
    } catch (e) {
      lastError.set(String(e));
      console.error("Failed to stop processing:", e);
    }
  }

  function reset() {
    isProcessing.set(false);
    current.set(0);
    total.set(0);
    totalOriginalBytes.set(0);
    processedBytes.set(0);
    results.set([]);
    batchResult.set(null);
    if (unlistenProgress) {
      unlistenProgress();
      unlistenProgress = null;
    }
    if (unlistenComplete) {
      unlistenComplete();
      unlistenComplete = null;
    }
  }

  return {
    isProcessing,
    lastError,
    current,
    total,
    totalOriginalBytes,
    processedBytes,
    results,
    batchResult,
    percentage,
    bytePercentage,
    totalSaved,
    startProcessing,
    stopProcessing,
    reset,
  };
}

export const progressStore = createProgressStore();
