import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ProgressEvent, BatchResult, FileMetadata, Profile } from "../types";

function createProgressStore() {
  const isProcessing = writable<boolean>(false);
  const current = writable<number>(0);
  const total = writable<number>(0);
  const results = writable<Array<{file: string; original_size: number; new_size: number; status: string}>>([]);
  const batchResult = writable<BatchResult | null>(null);

  const percentage = derived(
    [current, total],
    ([$current, $total]) =>
      $total > 0 ? Math.round(($current / $total) * 100) : 0
  );

  const totalSaved = derived(batchResult, ($result) =>
    $result ? $result.total_saved_bytes : 0
  );

  let unlisteners: Array<() => void> = [];

  async function startListening() {
    const unlisten1 = await listen<ProgressEvent>("progress_update", (event) => {
      const payload = event.payload;
      current.set(payload.current);
      total.set(payload.total);
      results.update((r) => [
        ...r,
        {
          file: payload.file,
          original_size: payload.original_size,
          new_size: payload.new_size,
          status: payload.status,
        },
      ]);
    });

    const unlisten2 = await listen<BatchResult>("processing_complete", (event) => {
      batchResult.set(event.payload);
      isProcessing.set(false);
    });

    unlisteners = [unlisten1, unlisten2];
  }

  function stopListening() {
    unlisteners.forEach((fn) => fn());
    unlisteners = [];
  }

  async function startProcessing(
    files: FileMetadata[],
    profile: Profile,
    sourceDir: string
  ) {
    results.set([]);
    current.set(0);
    total.set(files.length);
    batchResult.set(null);
    isProcessing.set(true);

    await startListening();
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
    results.set([]);
    batchResult.set(null);
    stopListening();
  }

  return {
    isProcessing,
    current,
    total,
    results,
    batchResult,
    percentage,
    totalSaved,
    startProcessing,
    stopProcessing,
    reset,
  };
}

export const progressStore = createProgressStore();
