# ImageResizer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform desktop batch image compression tool using Tauri v2 + Svelte + Rust.

**Architecture:** Tauri v2 app with Svelte 5 frontend (left-right split panel layout) and Rust backend. Backend uses `image` crate + `rayon` for concurrent image processing, `walkdir` for directory scanning, and `serde` for JSON config persistence. Frontend communicates with backend via Tauri commands (`invoke`) and receives real-time progress via Tauri events (`emit`/`listen`).

**Tech Stack:** Tauri v2, Svelte 5, TypeScript, Rust, `image` 0.25, `rayon` 1.10, `walkdir` 2, `serde`/`serde_json`, `dirs` 5

---

## File Structure

```
ImageResizer/
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
├── src/                              # Svelte frontend
│   ├── main.ts
│   ├── app.html
│   ├── app.css
│   ├── App.svelte                    # Main layout
│   └── lib/
│       ├── types.ts                  # TypeScript interfaces
│       ├── utils/
│       │   └── format.ts             # File size formatting
│       ├── stores/
│       │   ├── files.ts              # File list store
│       │   ├── profiles.ts           # Profile config store
│       │   └── progress.ts           # Processing progress store
│       └── components/
│           ├── FileBrowser.svelte    # Directory picker + file list
│           ├── SettingsPanel.svelte  # Container for settings
│           ├── ProfileSelector.svelte
│           ├── ResizeSettings.svelte
│           ├── OutputSettings.svelte
│           ├── QualitySettings.svelte
│           └── ProgressPanel.svelte
├── src-tauri/                        # Rust backend
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── src/
│       ├── lib.rs                    # Tauri entry + builder
│       ├── state.rs                  # AppState definition
│       ├── config.rs                 # Profile types + ConfigManager
│       ├── scanner.rs                # FileScanner
│       ├── processor.rs              # ImageProcessor
│       └── commands.rs               # Tauri command handlers
├── docs/
│   └── superpowers/
│       ├── specs/
│       │   └── 2026-04-25-image-resizer-design.md
│       └── plans/
│           └── 2026-04-25-image-resizer-plan.md
└── tests/                            # Rust integration tests
    └── fixtures/                     # Test images
```

---

### Task 1: Project Initialization

**Files:**
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/src/lib.rs`, `src-tauri/build.rs`
- Create: `package.json`, `svelte.config.js`, `vite.config.ts`, `tsconfig.json`
- Create: `src/main.ts`, `src/app.html`, `src/app.css`, `src/App.svelte`

- [ ] **Step 1: Create Tauri v2 + Svelte project**

Run from `F:\project`:

```bash
pnpm create tauri-app ImageResizer --template svelte-ts
```

If the interactive prompt asks, select:
- Project name: ImageResizer
- Frontend: Svelte (TypeScript)
- Package manager: pnpm

- [ ] **Step 2: Add Rust dependencies**

Edit `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
image = { version = "0.25", features = ["jpeg", "png", "webp", "gif"] }
rayon = "1.10"
walkdir = "2"
dirs = "5"
```

- [ ] **Step 3: Add frontend dependencies**

Run:

```bash
cd F:/project/ImageResizer
pnpm add @tauri-apps/plugin-dialog
```

- [ ] **Step 4: Configure Tauri capabilities**

Write `src-tauri/capabilities/default.json`:

```json
{
  "identifier": "default",
  "description": "Default permissions for ImageResizer",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:default",
    "dialog:allow-open"
  ]
}
```

- [ ] **Step 5: Configure tauri.conf.json**

Edit `src-tauri/tauri.conf.json`, set:
- `productName`: `"ImageResizer"`
- `identifier`: `"com.imageresizer.app"`
- Window width: `1100`, height: `700`
- `minWidth`: `900`, `minHeight`: `600`
- `title`: `"ImageResizer"`

- [ ] **Step 6: Verify project compiles and launches**

```bash
cd F:/project/ImageResizer
pnpm tauri dev
```

Expected: App window opens showing the default Svelte template.

- [ ] **Step 7: Clean up default template content**

Replace `src/App.svelte` with minimal content:

```svelte
<script lang="ts">
</script>

<main>
  <h1>ImageResizer</h1>
</main>

<style>
  :global(body) {
    margin: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  }
  main {
    padding: 16px;
  }
</style>
```

Replace `src/app.css` with:

```css
:root {
  --bg-primary: #f5f5f5;
  --bg-secondary: #ffffff;
  --text-primary: #333333;
  --text-secondary: #666666;
  --border-color: #e0e0e0;
  --accent: #1890ff;
  --success: #52c41a;
  --error: #ff4d4f;
  --warning: #faad14;
}
```

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "chore: initialize Tauri v2 + Svelte project"
```

---

### Task 2: Backend Types & ConfigManager

**Files:**
- Create: `src-tauri/src/config.rs`
- Create: `src-tauri/src/state.rs`
- Test: `src-tauri/src/config.rs` (inline `#[cfg(test)]` module)
- Modify: `src-tauri/src/lib.rs` (add module declarations)

- [ ] **Step 1: Write the config types and ConfigManager with tests**

Write `src-tauri/src/config.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Enums ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SizeUnit {
    Percentage,
    Pixel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResizeMode {
    Fit,
    Fill,
    Stretch,
    ShrinkOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputOperation {
    Overwrite,
    SameDir,
    CustomDir,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OutputFormat {
    SameAsOriginal,
    Jpeg,
    Png,
    WebP,
    Gif,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityMode {
    Quality,
    TargetSize,
    Original,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessStatus {
    Success,
    Failed(String),
    Skipped,
}

// ── Structs ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResizeSettings {
    pub width: u32,
    pub height: u32,
    pub unit: SizeUnit,
    pub mode: ResizeMode,
    pub keep_aspect_ratio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OutputSettings {
    pub operation: OutputOperation,
    pub custom_dir: Option<String>,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QualitySettings {
    pub mode: QualityMode,
    pub quality: u8,
    pub target_size_kb: Option<u32>,
    pub adjust_dpi: bool,
    pub dpi: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile {
    pub name: String,
    pub resize: ResizeSettings,
    pub output: OutputSettings,
    pub quality: QualitySettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileMetadata {
    pub path: String,
    pub size_bytes: u64,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessResult {
    pub file: String,
    pub original_size: u64,
    pub new_size: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub total: u32,
    pub current: u32,
    pub file: String,
    pub original_size: u64,
    pub new_size: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchResult {
    pub total_files: u32,
    pub success: u32,
    pub failed: u32,
    pub total_saved_bytes: u64,
    pub failed_files: Vec<String>,
}

// ── ConfigManager ──

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Self {
        let config_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ImageResizer");
        let _ = fs::create_dir_all(&config_dir);

        ConfigManager {
            config_path: config_dir.join("profiles.json"),
        }
    }

    pub fn load_profiles(&self) -> Vec<Profile> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_else(|_| Self::default_profiles())
        } else {
            let profiles = Self::default_profiles();
            let _ = self.save_profiles(&profiles);
            profiles
        }
    }

    pub fn save_profiles(&self, profiles: &[Profile]) -> Result<(), String> {
        let content =
            serde_json::to_string_pretty(profiles).map_err(|e| format!("Serialization error: {}", e))?;
        fs::write(&self.config_path, content)
            .map_err(|e| format!("Write error: {}", e))?;
        Ok(())
    }

    pub fn default_profiles() -> Vec<Profile> {
        vec![
            Profile {
                name: "常用".to_string(),
                resize: ResizeSettings {
                    width: 100,
                    height: 100,
                    unit: SizeUnit::Percentage,
                    mode: ResizeMode::Fit,
                    keep_aspect_ratio: true,
                },
                output: OutputSettings {
                    operation: OutputOperation::SameDir,
                    custom_dir: None,
                    format: OutputFormat::SameAsOriginal,
                },
                quality: QualitySettings {
                    mode: QualityMode::Quality,
                    quality: 40,
                    target_size_kb: None,
                    adjust_dpi: false,
                    dpi: 96,
                },
            },
            Profile {
                name: "高质量".to_string(),
                resize: ResizeSettings {
                    width: 100,
                    height: 100,
                    unit: SizeUnit::Percentage,
                    mode: ResizeMode::Fit,
                    keep_aspect_ratio: true,
                },
                output: OutputSettings {
                    operation: OutputOperation::CustomDir,
                    custom_dir: None,
                    format: OutputFormat::SameAsOriginal,
                },
                quality: QualitySettings {
                    mode: QualityMode::Quality,
                    quality: 85,
                    target_size_kb: None,
                    adjust_dpi: false,
                    dpi: 96,
                },
            },
            Profile {
                name: "极限压缩".to_string(),
                resize: ResizeSettings {
                    width: 50,
                    height: 50,
                    unit: SizeUnit::Percentage,
                    mode: ResizeMode::Fit,
                    keep_aspect_ratio: true,
                },
                output: OutputSettings {
                    operation: OutputOperation::SameDir,
                    custom_dir: None,
                    format: OutputFormat::WebP,
                },
                quality: QualitySettings {
                    mode: QualityMode::Quality,
                    quality: 20,
                    target_size_kb: None,
                    adjust_dpi: false,
                    dpi: 96,
                },
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_profiles() {
        let profiles = ConfigManager::default_profiles();
        assert_eq!(profiles.len(), 3);
        assert_eq!(profiles[0].name, "常用");
        assert_eq!(profiles[0].quality.quality, 40);
        assert_eq!(profiles[1].name, "高质量");
        assert_eq!(profiles[1].quality.quality, 85);
        assert_eq!(profiles[2].name, "极限压缩");
        assert_eq!(profiles[2].quality.quality, 20);
        assert_eq!(profiles[2].output.format, OutputFormat::WebP);
    }

    #[test]
    fn test_save_and_load_profiles() {
        let dir = std::env::temp_dir().join("imageresizer_test");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test_profiles.json");

        let cm = ConfigManager {
            config_path: path.clone(),
        };

        let profiles = ConfigManager::default_profiles();
        cm.save_profiles(&profiles).unwrap();

        let loaded = cm.load_profiles();
        assert_eq!(loaded, profiles);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_profile_serialization_roundtrip() {
        let profile = Profile {
            name: "test".to_string(),
            resize: ResizeSettings {
                width: 80,
                height: 60,
                unit: SizeUnit::Pixel,
                mode: ResizeMode::Fill,
                keep_aspect_ratio: false,
            },
            output: OutputSettings {
                operation: OutputOperation::CustomDir,
                custom_dir: Some("C:\\output".to_string()),
                format: OutputFormat::Jpeg,
            },
            quality: QualitySettings {
                mode: QualityMode::TargetSize,
                quality: 0,
                target_size_kb: Some(500),
                adjust_dpi: true,
                dpi: 72,
            },
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: Profile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, deserialized);
    }
}
```

- [ ] **Step 2: Write AppState**

Write `src-tauri/src/state.rs`:

```rust
use crate::config::ConfigManager;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct AppState {
    pub config_manager: ConfigManager,
    pub stop_flag: Arc<AtomicBool>,
    pub is_processing: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            config_manager: ConfigManager::new(),
            stop_flag: Arc::new(AtomicBool::new(false)),
            is_processing: Arc::new(AtomicBool::new(false)),
        }
    }
}
```

- [ ] **Step 3: Register modules in lib.rs**

Write `src-tauri/src/lib.rs`:

```rust
mod commands;
mod config;
mod processor;
mod scanner;
mod state;

use state::AppState;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![
            commands::scan_directory,
            commands::get_profiles,
            commands::save_profile,
            commands::delete_profile,
            commands::start_processing,
            commands::stop_processing,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Create stub files for other modules**

Write `src-tauri/src/scanner.rs`:

```rust
pub fn scan_directory(_dir: &str) -> Result<Vec<crate::config::FileMetadata>, String> {
    todo!("Task 3")
}
```

Write `src-tauri/src/processor.rs`:

```rust
pub fn process_file(
    _input_path: &str,
    _output_path: &str,
    _profile: &crate::config::Profile,
) -> Result<crate::config::ProcessResult, String> {
    todo!("Task 4")
}
```

Write `src-tauri/src/commands.rs`:

```rust
use crate::config::{FileMetadata, Profile};

#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<FileMetadata>, String> {
    crate::scanner::scan_directory(&path)
}

#[tauri::command]
pub async fn get_profiles(
    state: tauri::State<'_, std::sync::Mutex<crate::state::AppState>>,
) -> Result<Vec<Profile>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.config_manager.load_profiles())
}

#[tauri::command]
pub async fn save_profile(
    state: tauri::State<'_, std::sync::Mutex<crate::state::AppState>>,
    profile: Profile,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let mut profiles = state.config_manager.load_profiles();
    if let Some(pos) = profiles.iter().position(|p| p.name == profile.name) {
        profiles[pos] = profile;
    } else {
        profiles.push(profile);
    }
    state.config_manager.save_profiles(&profiles)
}

#[tauri::command]
pub async fn delete_profile(
    state: tauri::State<'_, std::sync::Mutex<crate::state::AppState>>,
    name: String,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let mut profiles = state.config_manager.load_profiles();
    profiles.retain(|p| p.name != name);
    if profiles.is_empty() {
        return Err("Cannot delete the last profile".to_string());
    }
    state.config_manager.save_profiles(&profiles)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn start_processing(
    _app: tauri::AppHandle,
    _state: tauri::State<'_, std::sync::Mutex<crate::state::AppState>>,
    _files: Vec<FileMetadata>,
    _profile: Profile,
    _source_dir: String,
) -> Result<(), String> {
    todo!("Task 6")
}

#[tauri::command]
pub async fn stop_processing(
    state: tauri::State<'_, std::sync::Mutex<crate::state::AppState>>,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    state.stop_flag.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}
```

- [ ] **Step 5: Run tests**

```bash
cd F:/project/ImageResizer/src-tauri && cargo test
```

Expected: All 3 config tests pass. Other modules are stubs so no other tests.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add backend types, ConfigManager, and AppState"
```

---

### Task 3: FileScanner

**Files:**
- Modify: `src-tauri/src/scanner.rs` (replace stub with implementation + tests)

- [ ] **Step 1: Implement FileScanner with tests**

Replace `src-tauri/src/scanner.rs`:

```rust
use crate::config::FileMetadata;
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp"];

pub fn scan_directory(dir: &str) -> Result<Vec<FileMetadata>, String> {
    let path = std::path::Path::new(dir);
    if !path.exists() {
        return Err(format!("Directory not found: {}", dir));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", dir));
    }

    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }

        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

        files.push(FileMetadata {
            path: entry.path().to_string_lossy().to_string(),
            size_bytes: size,
            extension: ext,
        });
    }

    Ok(files)
}

pub fn filter_by_extension(files: &[FileMetadata], extensions: &[&str]) -> Vec<FileMetadata> {
    files
        .iter()
        .filter(|f| {
            extensions
                .iter()
                .any(|ext| f.extension.eq_ignore_ascii_case(ext))
        })
        .cloned()
        .collect()
}

pub fn total_size(files: &[FileMetadata]) -> u64 {
    files.iter().map(|f| f.size_bytes).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dir() -> String {
        let dir = std::env::temp_dir().join("imageresizer_scanner_test");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir.join("subdir"));

        // Create test files
        fs::write(dir.join("photo.jpg"), "fake_jpg").unwrap();
        fs::write(dir.join("image.png"), "fake_png").unwrap();
        fs::write(dir.join("anim.gif"), "fake_gif").unwrap();
        fs::write(dir.join("pic.webp"), "fake_webp").unwrap();
        fs::write(dir.join("subdir/nested.jpg"), "fake_jpg2").unwrap();
        fs::write(dir.join("readme.txt"), "not_image").unwrap();
        fs::write(dir.join("data.json"), "not_image2").unwrap();

        dir.to_string_lossy().to_string()
    }

    fn cleanup_test_dir(dir: &str) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_scan_finds_image_files() {
        let dir = setup_test_dir();
        let files = scan_directory(&dir).unwrap();
        cleanup_test_dir(&dir);

        // Should find 5 image files, not txt/json
        assert_eq!(files.len(), 5);
    }

    #[test]
    fn test_scan_recurses_subdirs() {
        let dir = setup_test_dir();
        let files = scan_directory(&dir).unwrap();
        cleanup_test_dir(&dir);

        let nested: Vec<_> = files.iter().filter(|f| f.path.contains("subdir")).collect();
        assert_eq!(nested.len(), 1);
    }

    #[test]
    fn test_scan_nonexistent_dir() {
        let result = scan_directory("/nonexistent/path/12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_filter_by_extension() {
        let files = vec![
            FileMetadata {
                path: "a.jpg".to_string(),
                size_bytes: 100,
                extension: "jpg".to_string(),
            },
            FileMetadata {
                path: "b.png".to_string(),
                size_bytes: 200,
                extension: "png".to_string(),
            },
            FileMetadata {
                path: "c.gif".to_string(),
                size_bytes: 300,
                extension: "gif".to_string(),
            },
        ];

        let jpg_only = filter_by_extension(&files, &["jpg"]);
        assert_eq!(jpg_only.len(), 1);

        let jpg_png = filter_by_extension(&files, &["jpg", "png"]);
        assert_eq!(jpg_png.len(), 2);
    }

    #[test]
    fn test_total_size() {
        let files = vec![
            FileMetadata {
                path: "a.jpg".to_string(),
                size_bytes: 1024,
                extension: "jpg".to_string(),
            },
            FileMetadata {
                path: "b.png".to_string(),
                size_bytes: 2048,
                extension: "png".to_string(),
            },
        ];

        assert_eq!(total_size(&files), 3072);
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd F:/project/ImageResizer/src-tauri && cargo test scanner
```

Expected: All 5 scanner tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: implement FileScanner with recursive directory walking"
```

---

### Task 4: ImageProcessor

**Files:**
- Modify: `src-tauri/src/processor.rs` (replace stub with implementation + tests)

- [ ] **Step 1: Implement ImageProcessor core**

Replace `src-tauri/src/processor.rs`:

```rust
use crate::config::*;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageFormat};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct ImageProcessor;

impl ImageProcessor {
    /// Process a single image file: decode → resize → encode → save
    pub fn process_file(
        input_path: &str,
        output_path: &str,
        profile: &Profile,
    ) -> Result<ProcessResult, String> {
        let input = Path::new(input_path);
        let img = image::open(input)
            .map_err(|e| format!("Failed to open {}: {}", input_path, e))?;

        let original_size = std::fs::metadata(input_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Resize
        let resized = Self::resize_image(&img, &profile.resize);

        // Ensure output directory exists
        if let Some(parent) = Path::new(output_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create output dir: {}", e))?;
        }

        // Determine effective format
        let effective_format = Self::effective_format(input_path, &profile.output.format);

        // Encode and save
        Self::encode_and_save(&resized, output_path, effective_format, &profile.quality)?;

        let new_size = std::fs::metadata(output_path)
            .map(|m| m.len())
            .unwrap_or(0);

        Ok(ProcessResult {
            file: input_path.to_string(),
            original_size,
            new_size,
            status: "success".to_string(),
        })
    }

    /// Resize image according to settings
    fn resize_image(img: &DynamicImage, settings: &ResizeSettings) -> DynamicImage {
        let (orig_w, orig_h) = img.dimensions();
        if orig_w == 0 || orig_h == 0 {
            return img.clone();
        }

        let (target_w, target_h) = match settings.unit {
            SizeUnit::Percentage => {
                let w = orig_w * settings.width as u32 / 100;
                let h = orig_h * settings.height as u32 / 100;
                (w.max(1), h.max(1))
            }
            SizeUnit::Pixel => (
                settings.width.max(1),
                settings.height.max(1),
            ),
        };

        if settings.keep_aspect_ratio {
            match settings.mode {
                ResizeMode::Fit => {
                    let ratio = (target_w as f64 / orig_w as f64)
                        .min(target_h as f64 / orig_h as f64);
                    let new_w = (orig_w as f64 * ratio).max(1.0) as u32;
                    let new_h = (orig_h as f64 * ratio).max(1.0) as u32;
                    img.resize(new_w, new_h, FilterType::Lanczos3)
                }
                ResizeMode::Fill => {
                    let ratio = (target_w as f64 / orig_w as f64)
                        .max(target_h as f64 / orig_h as f64);
                    let new_w = (orig_w as f64 * ratio).max(1.0) as u32;
                    let new_h = (orig_h as f64 * ratio).max(1.0) as u32;
                    let resized = img.resize(new_w, new_h, FilterType::Lanczos3);
                    let x = (new_w.saturating_sub(target_w)) / 2;
                    let y = (new_h.saturating_sub(target_h)) / 2;
                    resized.crop_imm(x, y, target_w, target_h)
                }
                ResizeMode::Stretch => {
                    img.resize_exact(target_w, target_h, FilterType::Lanczos3)
                }
                ResizeMode::ShrinkOnly => {
                    if target_w >= orig_w && target_h >= orig_h {
                        return img.clone();
                    }
                    let ratio = (target_w as f64 / orig_w as f64)
                        .min(target_h as f64 / orig_h as f64);
                    let new_w = (orig_w as f64 * ratio).max(1.0) as u32;
                    let new_h = (orig_h as f64 * ratio).max(1.0) as u32;
                    img.resize(new_w, new_h, FilterType::Lanczos3)
                }
            }
        } else {
            img.resize_exact(target_w, target_h, FilterType::Lanczos3)
        }
    }

    /// Determine effective output format from settings and original file
    fn effective_format(input_path: &str, format: &OutputFormat) -> ImageFormat {
        match format {
            OutputFormat::SameAsOriginal => {
                let ext = Path::new(input_path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.to_lowercase())
                    .unwrap_or_default();
                match ext.as_str() {
                    "jpg" | "jpeg" => ImageFormat::Jpeg,
                    "png" => ImageFormat::Png,
                    "webp" => ImageFormat::WebP,
                    "gif" => ImageFormat::Gif,
                    _ => ImageFormat::Jpeg,
                }
            }
            OutputFormat::Jpeg => ImageFormat::Jpeg,
            OutputFormat::Png => ImageFormat::Png,
            OutputFormat::WebP => ImageFormat::WebP,
            OutputFormat::Gif => ImageFormat::Gif,
        }
    }

    /// Encode image and save to disk
    fn encode_and_save(
        img: &DynamicImage,
        path: &str,
        format: ImageFormat,
        quality_settings: &QualitySettings,
    ) -> Result<(), String> {
        let quality = match quality_settings.mode {
            QualityMode::Original => {
                // Save with default quality for the format
                img.save_with_format(path, format)
                    .map_err(|e| format!("Save error: {}", e))?;
                return Ok(());
            }
            QualityMode::Quality => quality_settings.quality.max(1).min(100),
            QualityMode::TargetSize => {
                let target_kb = quality_settings.target_size_kb.unwrap_or(100);
                return Self::save_with_target_size(img, path, format, target_kb);
            }
        };

        match format {
            ImageFormat::Jpeg => {
                let mut buf = Vec::new();
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
                img.write_with_encoder(encoder)
                    .map_err(|e| format!("JPEG encode error: {}", e))?;
                std::fs::write(path, buf)
                    .map_err(|e| format!("JPEG write error: {}", e))?;
            }
            ImageFormat::Png => {
                img.save_with_format(path, ImageFormat::Png)
                    .map_err(|e| format!("PNG save error: {}", e))?;
            }
            ImageFormat::WebP => {
                let mut buf = Vec::new();
                let encoder = image::codecs::webp::WebPEncoder::new_lossy(&mut buf);
                img.write_with_encoder(encoder)
                    .map_err(|e| format!("WebP encode error: {}", e))?;
                std::fs::write(path, buf)
                    .map_err(|e| format!("WebP write error: {}", e))?;
            }
            ImageFormat::Gif => {
                img.save_with_format(path, ImageFormat::Gif)
                    .map_err(|e| format!("GIF save error: {}", e))?;
            }
            _ => {
                img.save_with_format(path, format)
                    .map_err(|e| format!("Save error: {}", e))?;
            }
        }
        Ok(())
    }

    /// Binary search for quality that meets target file size
    fn save_with_target_size(
        img: &DynamicImage,
        path: &str,
        format: ImageFormat,
        target_kb: u32,
    ) -> Result<(), String> {
        let target_bytes = (target_kb as u64) * 1024;

        match format {
            ImageFormat::Jpeg => {
                let mut low: u8 = 1;
                let mut high: u8 = 100;
                let mut best_quality = low;
                let mut best_buf = Vec::new();

                while low <= high {
                    let mid = (low + high) / 2;
                    let mut buf = Vec::new();
                    let encoder =
                        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, mid);
                    let _ = img.write_with_encoder(encoder);

                    if buf.len() as u64 > target_bytes {
                        high = mid - 1;
                    } else {
                        best_quality = mid;
                        best_buf = buf;
                        low = mid + 1;
                    }
                }

                if best_buf.is_empty() {
                    let mut buf = Vec::new();
                    let encoder =
                        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 1);
                    img.write_with_encoder(encoder)
                        .map_err(|e| format!("JPEG encode error: {}", e))?;
                    std::fs::write(path, buf)
                        .map_err(|e| format!("JPEG write error: {}", e))?;
                } else {
                    std::fs::write(path, best_buf)
                        .map_err(|e| format!("JPEG write error: {}", e))?;
                }
            }
            _ => {
                // Fallback: save with quality mode 75
                img.save_with_format(path, format)
                    .map_err(|e| format!("Save error: {}", e))?;
            }
        }
        Ok(())
    }

    /// Compute output file path based on operation mode
    pub fn compute_output_path(
        file_path: &str,
        source_dir: &str,
        output_settings: &OutputSettings,
    ) -> String {
        let path = Path::new(file_path);
        let ext = match &output_settings.format {
            OutputFormat::SameAsOriginal => path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("jpg")
                .to_string(),
            OutputFormat::Jpeg => "jpg".to_string(),
            OutputFormat::Png => "png".to_string(),
            OutputFormat::WebP => "webp".to_string(),
            OutputFormat::Gif => "gif".to_string(),
        };

        match output_settings.operation {
            OutputOperation::Overwrite => file_path.to_string(),
            OutputOperation::SameDir => {
                let stem = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let new_name = format!("{}_compressed.{}", stem, ext);
                path.parent()
                    .unwrap_or(path)
                    .join(new_name)
                    .to_string_lossy()
                    .to_string()
            }
            OutputOperation::CustomDir => {
                let custom_dir = output_settings
                    .custom_dir
                    .as_deref()
                    .unwrap_or(source_dir);
                let source = Path::new(source_dir);
                let relative = path.strip_prefix(source).unwrap_or(path);
                let stem = path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let new_relative = relative
                    .parent()
                    .map(|p| p.join(format!("{}.{}", stem, ext)))
                    .unwrap_or_else(|| PathBuf::from(format!("{}.{}", stem, ext)));
                Path::new(custom_dir)
                    .join(new_relative)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }

    /// Batch process multiple files in parallel
    pub fn batch_process(
        files: &[FileMetadata],
        profile: &Profile,
        source_dir: &str,
        stop_flag: &Arc<AtomicBool>,
        progress_callback: impl Fn(ProgressEvent) + Send + Sync,
    ) -> BatchResult {
        let total = files.len() as u32;
        let results: Vec<ProcessResult> = files
            .par_iter()
            .enumerate()
            .filter_map(|(i, file)| {
                if stop_flag.load(Ordering::Relaxed) {
                    return Some(ProcessResult {
                        file: file.path.clone(),
                        original_size: file.size_bytes,
                        new_size: 0,
                        status: "skipped".to_string(),
                    });
                }

                let output_path =
                    Self::compute_output_path(&file.path, source_dir, &profile.output);

                let result = match Self::process_file(&file.path, &output_path, profile) {
                    Ok(r) => r,
                    Err(e) => ProcessResult {
                        file: file.path.clone(),
                        original_size: file.size_bytes,
                        new_size: 0,
                        status: format!("failed: {}", e),
                    },
                };

                progress_callback(ProgressEvent {
                    total,
                    current: (i + 1) as u32,
                    file: file.path.clone(),
                    original_size: result.original_size,
                    new_size: result.new_size,
                    status: result.status.clone(),
                });

                Some(result)
            })
            .collect();

        let success = results
            .iter()
            .filter(|r| r.status == "success")
            .count() as u32;
        let failed = results
            .iter()
            .filter(|r| r.status.starts_with("failed"))
            .count() as u32;
        let total_saved: u64 = results
            .iter()
            .filter_map(|r| {
                if r.status == "success" && r.new_size < r.original_size {
                    Some(r.original_size - r.new_size)
                } else {
                    None
                }
            })
            .sum();

        let failed_files: Vec<String> = results
            .iter()
            .filter(|r| r.status.starts_with("failed"))
            .map(|r| r.file.clone())
            .collect();

        BatchResult {
            total_files: total,
            success,
            failed,
            total_saved_bytes: total_saved,
            failed_files,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};
    use std::fs;

    fn create_test_image(path: &str, width: u32, height: u32) {
        let img = ImageBuffer::from_pixel(width, height, Rgb([255, 0, 0]));
        img.save(path).unwrap();
    }

    fn test_output_dir() -> String {
        let dir = std::env::temp_dir().join("imageresizer_processor_test");
        let _ = fs::create_dir_all(&dir);
        dir.to_string_lossy().to_string()
    }

    #[test]
    fn test_resize_fit_preserves_aspect_ratio() {
        let img = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(200, 100, Rgb([255, 0, 0])));
        let settings = ResizeSettings {
            width: 100,
            height: 100,
            unit: SizeUnit::Percentage,
            mode: ResizeMode::Fit,
            keep_aspect_ratio: true,
        };

        let resized = ImageProcessor::resize_image(&img, &settings);
        let (w, h) = resized.dimensions();

        // 200x100 at 100% should stay 200x100
        assert_eq!(w, 200);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_resize_fit_downscale() {
        let img = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(200, 100, Rgb([255, 0, 0])));
        let settings = ResizeSettings {
            width: 50,
            height: 50,
            unit: SizeUnit::Percentage,
            mode: ResizeMode::Fit,
            keep_aspect_ratio: true,
        };

        let resized = ImageProcessor::resize_image(&img, &settings);
        let (w, h) = resized.dimensions();

        assert_eq!(w, 100);
        assert_eq!(h, 50);
    }

    #[test]
    fn test_resize_pixel_absolute() {
        let img = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(200, 100, Rgb([255, 0, 0])));
        let settings = ResizeSettings {
            width: 80,
            height: 80,
            unit: SizeUnit::Pixel,
            mode: ResizeMode::Fit,
            keep_aspect_ratio: true,
        };

        let resized = ImageProcessor::resize_image(&img, &settings);
        let (w, h) = resized.dimensions();

        // Fit: should fit within 80x80, aspect ratio 2:1
        assert_eq!(w, 80);
        assert_eq!(h, 40);
    }

    #[test]
    fn test_resize_shrink_only() {
        let img = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(50, 50, Rgb([255, 0, 0])));
        let settings = ResizeSettings {
            width: 200,
            height: 200,
            unit: SizeUnit::Pixel,
            mode: ResizeMode::ShrinkOnly,
            keep_aspect_ratio: true,
        };

        let resized = ImageProcessor::resize_image(&img, &settings);
        let (w, h) = resized.dimensions();

        // Should not upscale
        assert_eq!(w, 50);
        assert_eq!(h, 50);
    }

    #[test]
    fn test_compute_output_path_same_dir() {
        let settings = OutputSettings {
            operation: OutputOperation::SameDir,
            custom_dir: None,
            format: OutputFormat::SameAsOriginal,
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert!(output.contains("_compressed"));
        assert!(output.ends_with(".jpg"));
    }

    #[test]
    fn test_compute_output_path_custom_dir() {
        let settings = OutputSettings {
            operation: OutputOperation::CustomDir,
            custom_dir: Some("C:\\output".to_string()),
            format: OutputFormat::WebP,
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert!(output.starts_with("C:\\output"));
        assert!(output.contains("vol1"));
        assert!(output.ends_with(".webp"));
    }

    #[test]
    fn test_process_file_jpeg_quality() {
        let dir = test_output_dir();
        let input = format!("{}\\test_input.jpg", dir);
        let output = format!("{}\\test_output.jpg", dir);

        create_test_image(&input, 100, 100);

        let profile = Profile {
            name: "test".to_string(),
            resize: ResizeSettings {
                width: 50,
                height: 50,
                unit: SizeUnit::Percentage,
                mode: ResizeMode::Fit,
                keep_aspect_ratio: true,
            },
            output: OutputSettings {
                operation: OutputOperation::SameDir,
                custom_dir: None,
                format: OutputFormat::Jpeg,
            },
            quality: QualitySettings {
                mode: QualityMode::Quality,
                quality: 40,
                target_size_kb: None,
                adjust_dpi: false,
                dpi: 96,
            },
        };

        let result = ImageProcessor::process_file(&input, &output, &profile).unwrap();
        assert_eq!(result.status, "success");

        // Output should exist and be smaller than original (compressed)
        assert!(std::path::Path::new(&output).exists());

        let _ = fs::remove_file(&input);
        let _ = fs::remove_file(&output);
    }

    #[test]
    fn test_process_nonexistent_file() {
        let profile = Profile {
            name: "test".to_string(),
            resize: ResizeSettings {
                width: 100,
                height: 100,
                unit: SizeUnit::Percentage,
                mode: ResizeMode::Fit,
                keep_aspect_ratio: true,
            },
            output: OutputSettings {
                operation: OutputOperation::SameDir,
                custom_dir: None,
                format: OutputFormat::Jpeg,
            },
            quality: QualitySettings {
                mode: QualityMode::Quality,
                quality: 80,
                target_size_kb: None,
                adjust_dpi: false,
                dpi: 96,
            },
        };

        let result = ImageProcessor::process_file("/nonexistent/file.jpg", "/tmp/out.jpg", &profile);
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd F:/project/ImageResizer/src-tauri && cargo test processor
```

Expected: All 8 processor tests pass.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: implement ImageProcessor with resize, encode, and batch processing"
```

---

### Task 5: Tauri Commands & Events

**Files:**
- Modify: `src-tauri/src/commands.rs` (replace stubs with full implementation)
- Modify: `src-tauri/src/state.rs` (update if needed)

- [ ] **Step 1: Implement full commands**

Replace `src-tauri/src/commands.rs`:

```rust
use crate::config::*;
use crate::processor::ImageProcessor;
use crate::state::AppState;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn scan_directory(path: String) -> Result<Vec<FileMetadata>, String> {
    crate::scanner::scan_directory(&path)
}

#[tauri::command]
pub async fn get_profiles(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<Profile>, String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    Ok(state.config_manager.load_profiles())
}

#[tauri::command]
pub async fn save_profile(
    state: tauri::State<'_, Mutex<AppState>>,
    profile: Profile,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let mut profiles = state.config_manager.load_profiles();
    if let Some(pos) = profiles.iter().position(|p| p.name == profile.name) {
        profiles[pos] = profile;
    } else {
        profiles.push(profile);
    }
    state.config_manager.save_profiles(&profiles)
}

#[tauri::command]
pub async fn delete_profile(
    state: tauri::State<'_, Mutex<AppState>>,
    name: String,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    let mut profiles = state.config_manager.load_profiles();
    profiles.retain(|p| p.name != name);
    if profiles.is_empty() {
        return Err("Cannot delete the last profile".to_string());
    }
    state.config_manager.save_profiles(&profiles)
}

#[tauri::command(rename_all = "camelCase")]
pub async fn start_processing(
    app: AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    files: Vec<FileMetadata>,
    profile: Profile,
    source_dir: String,
) -> Result<(), String> {
    {
        let state = state.lock().map_err(|e| e.to_string())?;
        if state.is_processing.load(Ordering::Relaxed) {
            return Err("Processing is already in progress".to_string());
        }
        state.is_processing.store(true, Ordering::Relaxed);
        state.stop_flag.store(false, Ordering::Relaxed);
    }

    // Get stop_flag Arc
    let stop_flag = {
        let state = state.lock().map_err(|e| e.to_string())?;
        Arc::clone(&state.stop_flag)
    };
    let is_processing = {
        let state = state.lock().map_err(|e| e.to_string())?;
        Arc::clone(&state.is_processing)
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = ImageProcessor::batch_process(
            &files,
            &profile,
            &source_dir,
            &stop_flag,
            move |event| {
                let _ = app_handle.emit("progress_update", &event);
            },
        );

        let _ = app_handle.emit("processing_complete", &result);
        is_processing.store(false, Ordering::Relaxed);
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_processing(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let state = state.lock().map_err(|e| e.to_string())?;
    if !state.is_processing.load(Ordering::Relaxed) {
        return Err("No processing in progress".to_string());
    }
    state.stop_flag.store(true, Ordering::Relaxed);
    Ok(())
}
```

- [ ] **Step 2: Verify build compiles**

```bash
cd F:/project/ImageResizer/src-tauri && cargo build
```

Expected: Compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: implement Tauri commands with async processing and progress events"
```

---

### Task 6: Frontend Foundation

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/utils/format.ts`
- Create: `src/lib/stores/files.ts`
- Create: `src/lib/stores/profiles.ts`
- Create: `src/lib/stores/progress.ts`

- [ ] **Step 1: Create TypeScript types**

Write `src/lib/types.ts`:

```typescript
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

export interface ProcessResult {
  file: string;
  original_size: number;
  new_size: number;
  status: string;
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
```

- [ ] **Step 2: Create utility functions**

Write `src/lib/utils/format.ts`:

```typescript
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
```

- [ ] **Step 3: Create files store**

Write `src/lib/stores/files.ts`:

```typescript
import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { FileMetadata, ExtensionFilter } from "../types";

function createFilesStore() {
  const { subscribe, set, update } = writable<FileMetadata[]>([]);
  const sourceDir = writable<string>("");
  const filter = writable<ExtensionFilter>("all");
  const loading = writable<boolean>(false);
  const error = writable<string | null>(null);

  const filteredFiles = derived(
    [subscribe as never, filter],
    ([$files, $filter]: [FileMetadata[], ExtensionFilter]) => {
      if ($filter === "all") return $files;
      if ($filter === "jpg") return $files.filter((f) => f.extension === "jpg" || f.extension === "jpeg");
      return $files.filter((f) => f.extension === $filter);
    }
  );

  const totalSize = derived(subscribe as never, ($files) =>
    $files.reduce((sum, f) => sum + f.size_bytes, 0)
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
```

- [ ] **Step 4: Create profiles store**

Write `src/lib/stores/profiles.ts`:

```typescript
import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { Profile } from "../types";

function createProfilesStore() {
  const { subscribe, set, update } = writable<Profile[]>([]);
  const activeProfileName = writable<string>("常用");
  const loading = writable<boolean>(false);

  const activeProfile = derived(
    [subscribe as never, activeProfileName],
    ([$profiles, $name]: [Profile[], string]) =>
      $profiles.find((p) => p.name === $name) ?? $profiles[0] ?? null
  );

  // Capture activeProfileName for closures
  let $activeProfileName = "常用";
  activeProfileName.subscribe((v) => ($activeProfileName = v));

  async function loadProfiles() {
    loading.set(true);
    try {
      const profiles = await invoke<Profile[]>("get_profiles");
      set(profiles);
      if (profiles.length > 0 && !profiles.find((p) => p.name === $activeProfileName)) {
        activeProfileName.set(profiles[0].name);
      }
    } catch (e) {
      console.error("Failed to load profiles:", e);
    } finally {
      loading.set(false);
    }
  }

  async function saveProfile(profile: Profile) {
    await invoke("save_profile", { profile });
    await loadProfiles();
  }

  async function deleteProfile(name: string) {
    await invoke("delete_profile", { name });
    await loadProfiles();
  }

  function updateActiveProfile(partial: Partial<Profile>) {
    update((profiles) =>
      profiles.map((p) =>
        p.name === $activeProfileName ? { ...p, ...partial } : p
      )
    );
  }

  return {
    subscribe,
    activeProfile,
    activeProfileName,
    loading,
    loadProfiles,
    saveProfile,
    deleteProfile,
    updateActiveProfile,
  };
}

export const profilesStore = createProfilesStore();
```

- [ ] **Step 5: Create progress store**

Write `src/lib/stores/progress.ts`:

```typescript
import { writable, derived } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ProgressEvent, BatchResult, ProcessResult, FileMetadata, Profile } from "../types";

function createProgressStore() {
  const isProcessing = writable<boolean>(false);
  const current = writable<number>(0);
  const total = writable<number>(0);
  const results = writable<ProcessResult[]>([]);
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
```

- [ ] **Step 6: Verify frontend compiles**

```bash
cd F:/project/ImageResizer && pnpm dev
```

Expected: No TypeScript errors in terminal.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add frontend types, utilities, and state stores"
```

---

### Task 7: FileBrowser Component

**Files:**
- Create: `src/lib/components/FileBrowser.svelte`

- [ ] **Step 1: Implement FileBrowser component**

Write `src/lib/components/FileBrowser.svelte`:

```svelte
<script lang="ts">
  import { filesStore } from "../stores/files";
  import { formatFileSize } from "../utils/format";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { ExtensionFilter } from "../types";

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
  {#if $filesStore.sourceDir}
    <div class="source-dir">{$filesStore.sourceDir}</div>
  {/if}

  {#if $filesStore.loading}
    <div class="loading">扫描中...</div>
  {:else if $filesStore.error}
    <div class="error">{$filesStore.error}</div>
  {:else if $filesStore.filteredFiles.length > 0}
    <div class="toolbar">
      <span class="file-count">共 {$filesStore.filteredFiles.length} 个文件</span>
      <select onchange={onFilterChange} value={$filesStore.filter}>
        {#each filterOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>
    <div class="total-size">
      总大小: {formatFileSize($filesStore.totalSize)}
    </div>
    <div class="file-list">
      {#each $filesStore.filteredFiles.slice(0, 200) as file (file.path)}
        <div class="file-item">
          <span class="file-name" title={file.path}>{file.path.split(/[/\\]/).pop()}</span>
          <span class="file-size">{formatFileSize(file.size_bytes)}</span>
        </div>
      {/each}
      {#if $filesStore.filteredFiles.length > 200}
        <div class="file-more">... 还有 {$filesStore.filteredFiles.length - 200} 个文件</div>
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
```

- [ ] **Step 2: Verify component renders in App.svelte**

Temporarily add to `src/App.svelte`:

```svelte
<script lang="ts">
  import FileBrowser from "./lib/components/FileBrowser.svelte";
</script>

<main>
  <FileBrowser />
</main>
```

Run `pnpm tauri dev` and verify the file browser renders.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: implement FileBrowser component with directory picker and file list"
```

---

### Task 8: SettingsPanel Components

**Files:**
- Create: `src/lib/components/ProfileSelector.svelte`
- Create: `src/lib/components/ResizeSettings.svelte`
- Create: `src/lib/components/OutputSettings.svelte`
- Create: `src/lib/components/QualitySettings.svelte`
- Create: `src/lib/components/SettingsPanel.svelte`

- [ ] **Step 1: Implement ProfileSelector**

Write `src/lib/components/ProfileSelector.svelte`:

```svelte
<script lang="ts">
  import { profilesStore } from "../stores/profiles";

  function onProfileChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    profilesStore.activeProfileName.set(target.value);
  }

  function addProfile() {
    const name = prompt("输入新方案名称:");
    if (!name) return;
    const current = $profilesStore.activeProfile;
    if (!current) return;
    profilesStore.saveProfile({
      ...current,
      name,
    });
  }

  function renameProfile() {
    const current = $profilesStore.activeProfile;
    if (!current) return;
    const newName = prompt("输入新名称:", current.name);
    if (!newName) return;
    profilesStore.saveProfile({
      ...current,
      name: newName,
    });
    profilesStore.activeProfileName.set(newName);
  }

  function deleteProfile() {
    const current = $profilesStore.activeProfile;
    if (!current) return;
    if (!confirm(`确定要删除方案 "${current.name}" 吗？`)) return;
    profilesStore.deleteProfile(current.name);
  }
</script>

<div class="profile-selector">
  <label>配置方案:</label>
  <div class="profile-row">
    <select onchange={onProfileChange} value={$profilesStore.activeProfileName}>
      {#each $profilesStore as profile}
        <option value={profile.name}>{profile.name}</option>
      {/each}
    </select>
    <button onclick={addProfile} title="新建方案">+</button>
    <button onclick={renameProfile} title="重命名">✎</button>
    <button onclick={deleteProfile} title="删除">✕</button>
  </div>
</div>

<style>
  .profile-selector {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .profile-selector label {
    font-size: 13px;
    white-space: nowrap;
  }
  .profile-row {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  select {
    padding: 4px 8px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
    background: var(--bg-secondary);
  }
  button {
    padding: 2px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    background: var(--bg-secondary);
    cursor: pointer;
    font-size: 12px;
  }
  button:hover {
    border-color: var(--accent);
  }
</style>
```

- [ ] **Step 2: Implement ResizeSettings**

Write `src/lib/components/ResizeSettings.svelte`:

```svelte
<script lang="ts">
  import { profilesStore } from "../stores/profiles";
  import type { SizeUnit, ResizeMode } from "../types";

  const units: { label: string; value: SizeUnit }[] = [
    { label: "百分比", value: "Percentage" },
    { label: "像素", value: "Pixel" },
  ];

  const modes: { label: string; value: ResizeMode }[] = [
    { label: "合适大小", value: "Fit" },
    { label: "填充", value: "Fill" },
    { label: "拉伸", value: "Stretch" },
    { label: "仅缩小", value: "ShrinkOnly" },
  ];

  function updateResize(partial: Record<string, any>) {
    const profile = $profilesStore.activeProfile;
    if (!profile) return;
    profilesStore.updateActiveProfile({
      ...profile,
      resize: { ...profile.resize, ...partial },
    });
  }
</script>

{#if $profilesStore.activeProfile}
  <div class="resize-settings">
    <div class="section-title">调整大小</div>
    <div class="field-row">
      <label>宽度:</label>
      <input
        type="number"
        min="1"
        max="10000"
        value={$profilesStore.activeProfile.resize.width}
        oninput={(e) => updateResize({ width: Number((e.target as HTMLInputElement).value) })}
      />
      <select
        value={$profilesStore.activeProfile.resize.unit}
        onchange={(e) => updateResize({ unit: (e.target as HTMLSelectElement).value })}
      >
        {#each units as u}
          <option value={u.value}>{u.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label>高度:</label>
      <input
        type="number"
        min="1"
        max="10000"
        value={$profilesStore.activeProfile.resize.height}
        oninput={(e) => updateResize({ height: Number((e.target as HTMLInputElement).value) })}
      />
      <select
        value={$profilesStore.activeProfile.resize.unit}
        onchange={(e) => updateResize({ unit: (e.target as HTMLSelectElement).value })}
      >
        {#each units as u}
          <option value={u.value}>{u.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label>模式:</label>
      <select
        value={$profilesStore.activeProfile.resize.mode}
        onchange={(e) => updateResize({ mode: (e.target as HTMLSelectElement).value })}
      >
        {#each modes as m}
          <option value={m.value}>{m.label}</option>
        {/each}
      </select>
    </div>
    <div class="field-row">
      <label>
        <input
          type="checkbox"
          checked={$profilesStore.activeProfile.resize.keep_aspect_ratio}
          onchange={(e) => updateResize({ keep_aspect_ratio: (e.target as HTMLInputElement).checked })}
        />
        保持宽高比
      </label>
    </div>
  </div>
{/if}

<style>
  .resize-settings {
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
  input[type="number"] {
    width: 70px;
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
  }
  select {
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
    background: var(--bg-secondary);
  }
  input[type="checkbox"] {
    margin-right: 4px;
  }
</style>
```

- [ ] **Step 3: Implement OutputSettings**

Write `src/lib/components/OutputSettings.svelte`:

```svelte
<script lang="ts">
  import { profilesStore } from "../stores/profiles";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { OutputOperation, OutputFormat } from "../types";

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

  function updateOutput(partial: Record<string, any>) {
    const profile = $profilesStore.activeProfile;
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

{#if $profilesStore.activeProfile}
  <div class="output-settings">
    <div class="section-title">输出设置</div>
    <div class="field-row">
      <label>操作:</label>
      <select
        value={$profilesStore.activeProfile.output.operation}
        onchange={(e) => updateOutput({ operation: (e.target as HTMLSelectElement).value })}
      >
        {#each operations as op}
          <option value={op.value}>{op.label}</option>
        {/each}
      </select>
    </div>
    {#if $profilesStore.activeProfile.output.operation === "CustomDir"}
      <div class="field-row">
        <label>输出目录:</label>
        <input
          type="text"
          readonly
          value={$profilesStore.activeProfile.output.custom_dir || "点击选择..."}
          onclick={pickCustomDir}
          class="dir-input"
        />
      </div>
    {/if}
    <div class="field-row">
      <label>格式:</label>
      <select
        value={$profilesStore.activeProfile.output.format}
        onchange={(e) => updateOutput({ format: (e.target as HTMLSelectElement).value })}
      >
        {#each formats as f}
          <option value={f.value}>{f.label}</option>
        {/each}
      </select>
    </div>
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
```

- [ ] **Step 4: Implement QualitySettings**

Write `src/lib/components/QualitySettings.svelte`:

```svelte
<script lang="ts">
  import { profilesStore } from "../stores/profiles";
  import type { QualityMode } from "../types";
</script>

{#if $profilesStore.activeProfile}
  <div class="quality-settings">
    <div class="section-title">品质与格式</div>
    <div class="quality-mode-row">
      <label>
        <input
          type="radio"
          name="quality-mode"
          checked={$profilesStore.activeProfile.quality.mode === "Quality"}
          onchange={() => {
            const profile = $profilesStore.activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: { ...profile.quality, mode: "Quality" as QualityMode },
            });
          }}
        />
        品质
      </label>
      <label>
        <input
          type="radio"
          name="quality-mode"
          checked={$profilesStore.activeProfile.quality.mode === "TargetSize"}
          onchange={() => {
            const profile = $profilesStore.activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: { ...profile.quality, mode: "TargetSize" as QualityMode },
            });
          }}
        />
        目标大小
      </label>
      <label>
        <input
          type="checkbox"
          checked={$profilesStore.activeProfile.quality.mode === "Original"}
          onchange={(e) => {
            const profile = $profilesStore.activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                mode: (e.target as HTMLInputElement).checked ? "Original" as QualityMode : "Quality" as QualityMode,
              },
            });
          }}
        />
        保持原始品质
      </label>
    </div>

    {#if $profilesStore.activeProfile.quality.mode === "Quality"}
      <div class="field-row">
        <label>品质:</label>
        <input
          type="range"
          min="1"
          max="100"
          value={$profilesStore.activeProfile.quality.quality}
          oninput={(e) => {
            const profile = $profilesStore.activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                quality: Number((e.target as HTMLInputElement).value),
              },
            });
          }}
        />
        <span class="value">{$profilesStore.activeProfile.quality.quality}%</span>
      </div>
    {/if}

    {#if $profilesStore.activeProfile.quality.mode === "TargetSize"}
      <div class="field-row">
        <label>大小:</label>
        <input
          type="number"
          min="1"
          value={$profilesStore.activeProfile.quality.target_size_kb || 100}
          oninput={(e) => {
            const profile = $profilesStore.activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                target_size_kb: Number((e.target as HTMLInputElement).value),
              },
            });
          }}
        />
        <span class="value">KB</span>
      </div>
    {/if}

    <div class="field-row">
      <label>
        <input
          type="checkbox"
          checked={$profilesStore.activeProfile.quality.adjust_dpi}
          onchange={(e) => {
            const profile = $profilesStore.activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                adjust_dpi: (e.target as HTMLInputElement).checked,
              },
            });
          }}
        />
        调整分辨率
      </label>
      {#if $profilesStore.activeProfile.quality.adjust_dpi}
        <input
          type="number"
          min="1"
          max="2400"
          value={$profilesStore.activeProfile.quality.dpi}
          oninput={(e) => {
            const profile = $profilesStore.activeProfile!;
            profilesStore.updateActiveProfile({
              ...profile,
              quality: {
                ...profile.quality,
                dpi: Number((e.target as HTMLInputElement).value),
              },
            });
          }}
        />
        <span class="value">DPI</span>
      {/if}
    </div>
  </div>
{/if}

<style>
  .quality-settings {
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
  .quality-mode-row {
    display: flex;
    gap: 12px;
    font-size: 13px;
    flex-wrap: wrap;
  }
  .field-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
  }
  label {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  input[type="radio"], input[type="checkbox"] {
    margin: 0;
  }
  input[type="range"] {
    flex: 1;
    max-width: 200px;
  }
  input[type="number"] {
    width: 70px;
    padding: 3px 6px;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    font-size: 13px;
  }
  .value {
    color: var(--text-secondary);
    font-size: 12px;
    min-width: 30px;
  }
</style>
```

- [ ] **Step 5: Implement SettingsPanel container**

Write `src/lib/components/SettingsPanel.svelte`:

```svelte
<script lang="ts">
  import ProfileSelector from "./ProfileSelector.svelte";
  import ResizeSettings from "./ResizeSettings.svelte";
  import OutputSettings from "./OutputSettings.svelte";
  import QualitySettings from "./QualitySettings.svelte";
</script>

<div class="settings-panel">
  <ProfileSelector />
  <div class="settings-body">
    <ResizeSettings />
    <OutputSettings />
    <QualitySettings />
  </div>
</div>

<style>
  .settings-panel {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .settings-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-left: 4px;
  }
</style>
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: implement SettingsPanel with profile selector and all setting components"
```

---

### Task 9: ProgressPanel Component

**Files:**
- Create: `src/lib/components/ProgressPanel.svelte`

- [ ] **Step 1: Implement ProgressPanel**

Write `src/lib/components/ProgressPanel.svelte`:

```svelte
<script lang="ts">
  import { progressStore } from "../stores/progress";
  import { formatFileSize, formatSavings } from "../utils/format";
</script>

<div class="progress-panel">
  <div class="section-title">处理进度</div>

  {#if $progressStore.isProcessing || $progressStore.percentage > 0}
    <div class="progress-bar-container">
      <div class="progress-bar" style="width: {$progressStore.percentage}%"></div>
    </div>
    <div class="progress-text">
      {$progressStore.percentage}% ({$progressStore.current}/{$progressStore.total})
    </div>
  {/if}

  {#if $progressStore.results.length > 0}
    <div class="results-table">
      <div class="table-header">
        <span class="col-file">文件</span>
        <span class="col-size">原始</span>
        <span class="col-size">压缩后</span>
        <span class="col-saving">节省</span>
      </div>
      <div class="table-body">
        {#each $progressStore.results.slice(-50) as result (result.file)}
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

  {#if $progressStore.batchResult}
    <div class="summary">
      <div class="summary-item">
        <span>成功:</span>
        <strong>{$progressStore.batchResult.success}</strong>
      </div>
      <div class="summary-item">
        <span>失败:</span>
        <strong class:has-failures={$progressStore.batchResult.failed > 0}>
          {$progressStore.batchResult.failed}
        </strong>
      </div>
      <div class="summary-item total-saved">
        <span>总计节省:</span>
        <strong>{formatFileSize($progressStore.totalSaved)}</strong>
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
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "feat: implement ProgressPanel with progress bar, file table, and statistics"
```

---

### Task 10: App Integration & Build

**Files:**
- Modify: `src/App.svelte` (final layout)
- Modify: `src/app.css` (layout styles)

- [ ] **Step 1: Write final App.svelte layout**

Replace `src/App.svelte`:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import FileBrowser from "./lib/components/FileBrowser.svelte";
  import SettingsPanel from "./lib/components/SettingsPanel.svelte";
  import ProgressPanel from "./lib/components/ProgressPanel.svelte";
  import { filesStore } from "./lib/stores/files";
  import { profilesStore } from "./lib/stores/profiles";
  import { progressStore } from "./lib/stores/progress";

  let canStart = false;

  $: canStart =
    $filesStore.filteredFiles.length > 0 &&
    $profilesStore.activeProfile !== null &&
    !$progressStore.isProcessing;

  async function handleStart() {
    const profile = $profilesStore.activeProfile;
    if (!profile || $filesStore.filteredFiles.length === 0) return;

    await profilesStore.saveProfile(profile);
    await progressStore.startProcessing(
      $filesStore.filteredFiles,
      profile,
      $filesStore.sourceDir
    );
  }

  function handleStop() {
    progressStore.stopProcessing();
  }

  onMount(async () => {
    await profilesStore.loadProfiles();
  });
</script>

<div class="app-layout">
  <header class="app-header">
    <h1>ImageResizer</h1>
  </header>

  <div class="app-body">
    <aside class="left-panel">
      <FileBrowser />
    </aside>

    <div class="right-panel">
      <div class="settings-area">
        <SettingsPanel />
      </div>

      <div class="progress-area">
        <ProgressPanel />
      </div>

      <div class="action-bar">
        <button
          class="btn-stop"
          onclick={handleStop}
          disabled={!$progressStore.isProcessing}
        >
          ⏹ 停止
        </button>
        <button
          class="btn-start"
          onclick={handleStart}
          disabled={!canStart}
        >
          ▶ 开始压缩
        </button>
      </div>
    </div>
  </div>
</div>

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--bg-primary);
    color: var(--text-primary);
  }
  .app-header {
    padding: 8px 16px;
    border-bottom: 1px solid var(--border-color);
    background: var(--bg-secondary);
  }
  .app-header h1 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }
  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .left-panel {
    width: 280px;
    min-width: 200px;
    border-right: 1px solid var(--border-color);
    padding: 12px;
    overflow-y: auto;
  }
  .right-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 12px;
    overflow-y: auto;
  }
  .settings-area {
    flex-shrink: 0;
  }
  .progress-area {
    flex: 1;
    overflow-y: auto;
  }
  .action-bar {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 0 0;
    border-top: 1px solid var(--border-color);
    margin-top: 8px;
  }
  button {
    padding: 8px 20px;
    border-radius: 4px;
    font-size: 14px;
    cursor: pointer;
    border: 1px solid var(--border-color);
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .btn-stop {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }
  .btn-start {
    background: var(--accent);
    color: white;
    border-color: var(--accent);
  }
  .btn-start:hover:not(:disabled) {
    opacity: 0.9;
  }
</style>
```

- [ ] **Step 2: Update app.css with layout styles**

Ensure `src/app.css` contains:

```css
:root {
  --bg-primary: #f5f5f5;
  --bg-secondary: #ffffff;
  --text-primary: #333333;
  --text-secondary: #666666;
  --border-color: #e0e0e0;
  --accent: #1890ff;
  --success: #52c41a;
  --error: #ff4d4f;
  --warning: #faad14;
}

* {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  font-size: 14px;
}
```

- [ ] **Step 3: Verify app builds and runs**

```bash
cd F:/project/ImageResizer && pnpm tauri dev
```

Expected: Full app renders with left panel (file browser), right panel (settings + progress), and action buttons.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: integrate all components into main app layout"
```

---

### Task 11: Manual Verification & Polish

**Files:**
- Modify: any files that need fixes discovered during testing

- [ ] **Step 1: Test directory scanning**

1. Launch app
2. Click "选择目录..." and select a folder with images
3. Verify: file list populates, file count shows, total size displays
4. Test format filter dropdown: JPG, PNG, etc.

- [ ] **Step 2: Test profile management**

1. Switch between default profiles (常用, 高质量, 极限压缩)
2. Verify settings update when switching
3. Create a new profile: click "+", enter name
4. Verify new profile appears and settings carry over
5. Rename a profile: click "✎", enter new name
6. Delete the test profile: click "✕"
7. Verify deletion fails when only one profile remains

- [ ] **Step 3: Test settings modification**

1. Change resize width/height and verify values update
2. Switch between 百分比 and 像素 units
3. Switch resize modes (合适大小, 填充, 拉伸, 仅缩小)
4. Toggle "保持宽高比"
5. Change output operation, verify custom directory picker appears
6. Change output format
7. Change quality slider, verify percentage displays
8. Switch to "目标大小" mode, verify KB input appears
9. Toggle "调整分辨率", verify DPI input appears

- [ ] **Step 4: Test image processing**

1. Select a directory with test images
2. Choose "常用" profile (100%, quality 40%, same dir)
3. Click "开始压缩"
4. Verify: progress bar advances, file results appear, statistics update
5. After completion, verify `_compressed` files exist in original directory
6. Verify compressed files are smaller than originals
7. Click "开始压缩" again with same settings — verify idempotent

- [ ] **Step 5: Test stop processing**

1. Select a large directory (or use high-resolution images)
2. Start processing
3. Click "停止" during processing
4. Verify: processing stops, partial results displayed

- [ ] **Step 6: Fix any issues discovered**

Address bugs found during manual testing. Common issues to watch for:
- Path separator handling on Windows (backslash vs forward slash)
- File locking (can't overwrite a file that's open)
- Large directory scanning performance
- Error display for failed files

- [ ] **Step 7: Production build**

```bash
cd F:/project/ImageResizer && pnpm tauri build
```

Expected: Build completes, installer produced in `src-tauri/target/release/bundle/`

- [ ] **Step 8: Final commit**

```bash
git add -A
git commit -m "fix: polish and fix issues from manual testing"
```
