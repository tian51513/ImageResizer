export interface Profile {
  name: string;
  resize: ResizeSettings;
  output: OutputSettings;
  quality: QualitySettings;
}

export interface ResizeSettings {
  width: number;
  height: number;
  unit: SizeUnit;
  mode: ResizeMode;
  keep_aspect_ratio: boolean;
}

export type SizeUnit = "Percentage" | "Pixel";
export type ResizeMode = "Fit" | "Fill" | "Stretch" | "ShrinkOnly";

export interface OutputSettings {
  operation: OutputOperation;
  custom_dir: string | null;
  format: OutputFormat;
}

export type OutputOperation = "Overwrite" | "SameDir" | "CustomDir";
export type OutputFormat = "SameAsOriginal" | "Jpeg" | "Png" | "WebP" | "Gif";

export interface QualitySettings {
  mode: QualityMode;
  quality: number;
  target_size_kb: number | null;
  adjust_dpi: boolean;
  dpi: number;
}

export type QualityMode = "Quality" | "TargetSize" | "Original";

export interface FileMetadata {
  path: string;
  size_bytes: number;
  extension: string;
}

export interface ProgressEvent {
  total: number;
  current: number;
  file: string;
  original_size: number;
  new_size: number;
  status: string;
}

export interface BatchResult {
  total_files: number;
  success: number;
  failed: number;
  total_saved_bytes: number;
  failed_files: string[];
}

export type ExtensionFilter = "all" | "jpg" | "png" | "webp" | "gif";
