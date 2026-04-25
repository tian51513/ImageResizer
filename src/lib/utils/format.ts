export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

export function formatSavings(original: number, compressed: number): string {
  if (original === 0) return "0%";
  const saved = ((1 - compressed / original) * 100);
  if (saved <= 0) return "+0%";
  return `-${saved.toFixed(1)}%`;
}
