import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ProgressEvent, BatchResult, FileMetadata, Profile } from "../types";

function createProgressStore() {
  const isProcessing = writable<boolean>(false);
  const current = writable<number>(0);
  const total = writable<number>(0);
  const totalOriginalBytes = writable<number>(0);
  const processedBytes = writable<number>(0);
  const results = writable<Array<{file: string; original_size: number; new_size: number; status: string}>>([]);
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

    unlistenProgress = await listen<ProgressEvent>("progress_update", (e) => {
      current.set(e.payload.current);
      total.set(e.payload.total);
      totalOriginalBytes.set(e.payload.total_original_bytes);
      processedBytes.set(e.payload.processed_bytes);
      results.update((r) => [
        ...r,
        {
          file: e.payload.file,
          original_size: e.payload.original_size,
          new_size: e.payload.new_size,
          status: e.payload.status,
        },
      ]);
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
      console.error("Failed to start processing:", e);
    }
  }

  async function stopProcessing() {
    try {
      await invoke("stop_processing");
    } catch (e) {
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
