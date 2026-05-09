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
pub enum NamingMode {
    KeepOriginal,
    CustomSuffix,
    DateSuffix,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityMode {
    Quality,
    TargetSize,
    Original,
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
    pub naming: NamingMode,
    pub custom_suffix: Option<String>,
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
    pub total_original_bytes: u64,
    pub processed_bytes: u64,
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
                    naming: NamingMode::DateSuffix,
                    custom_suffix: None,
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
                    naming: NamingMode::KeepOriginal,
                    custom_suffix: None,
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
                    naming: NamingMode::DateSuffix,
                    custom_suffix: None,
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
        let _ = fs::remove_dir_all(&dir);
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
                naming: NamingMode::CustomSuffix,
                custom_suffix: Some("_mini".to_string()),
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
