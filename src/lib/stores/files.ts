import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { FileMetadata, ExtensionFilter } from "../types";

function createFilesStore() {
  const store = writable<FileMetadata[]>([]);
  const { subscribe, set, update } = store;
  const sourceDir = writable<string>("");
  const filter = writable<ExtensionFilter>("all");
  const loading = writable<boolean>(false);
  const error = writable<string | null>(null);

  // Use a regular writable for filteredFiles - subscribe to raw files
  const filteredFiles = writable<FileMetadata[]>([]);

  // Keep filteredFiles in sync when files or filter changes
  let currentFiles: FileMetadata[] = [];
  let currentFilter: ExtensionFilter = "all";

  subscribe((value) => {
    currentFiles = value;
    updateFilteredFiles();
  });

  filter.subscribe((value) => {
    currentFilter = value;
    updateFilteredFiles();
  });

  function updateFilteredFiles() {
    if (currentFilter === "all") {
      filteredFiles.set(currentFiles);
    } else if (currentFilter === "jpg") {
      filteredFiles.set(currentFiles.filter((f) => f.extension === "jpg" || f.extension === "jpeg"));
    } else {
      filteredFiles.set(currentFiles.filter((f) => f.extension === currentFilter));
    }
  }

  const totalSize = derived(store, ($files: FileMetadata[]) =>
    $files.reduce((sum: number, f: FileMetadata) => sum + f.size_bytes, 0)
  );

  async function scanDirectory(dir: string) {
    sourceDir.set(dir);
    loading.set(true);
    error.set(null);
    try {
      const files = await invoke<FileMetadata[]>("scan_directory", { path: dir });
      set(files);
    } catch (e) {
      error.set(String(e));
      set([]);
    } finally {
      loading.set(false);
    }
  }

  return {
    subscribe,
    sourceDir,
    filter,
    loading,
    error,
    filteredFiles,
    totalSize,
    scanDirectory,
  };
}

export const filesStore = createFilesStore();
