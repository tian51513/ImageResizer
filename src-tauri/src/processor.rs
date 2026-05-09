use crate::config::*;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, ImageFormat};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

pub struct ImageProcessor;

impl ImageProcessor {
    /// Process a single image file: decode -> resize -> encode -> save
    pub fn process_file(
        input_path: &str,
        output_path: &str,
        profile: &Profile,
    ) -> Result<ProcessResult, String> {
        let input = Path::new(input_path);
        log::info!("Processing file: {}", input_path);

        let img = ImageReader::open(input)
            .map_err(|e| {
                log::error!("Failed to open {}: {}", input_path, e);
                format!("Failed to open {}: {}", input_path, e)
            })?
            .with_guessed_format()
            .map_err(|e| {
                log::error!("Failed to detect format for {}: {}", input_path, e);
                format!("Failed to detect format for {}: {}", input_path, e)
            })?
            .decode()
            .map_err(|e| {
                log::error!("Failed to decode {}: {}", input_path, e);
                format!("Failed to decode {}: {}", input_path, e)
            })?;

        let (orig_w, orig_h) = img.dimensions();
        let color = match &img {
            DynamicImage::ImageRgb8(_) => "RGB8",
            DynamicImage::ImageRgba8(_) => "RGBA8",
            DynamicImage::ImageRgb16(_) => "RGB16",
            DynamicImage::ImageRgba16(_) => "RGBA16",
            DynamicImage::ImageRgb32F(_) => "RGB32F",
            DynamicImage::ImageRgba32F(_) => "RGBA32F",
            DynamicImage::ImageLuma8(_) => "Luma8",
            DynamicImage::ImageLumaA8(_) => "LumaA8",
            DynamicImage::ImageLuma16(_) => "Luma16",
            DynamicImage::ImageLumaA16(_) => "LumaA16",
            _ => "Unknown",
        };
        log::info!("  Image: {}x{}, color={}", orig_w, orig_h, color);

        let original_size = std::fs::metadata(input_path)
            .map(|m| m.len())
            .unwrap_or(0);

        // Resize
        let resized = Self::resize_image(&img, &profile.resize);

        // Ensure output directory exists
        if let Some(parent) = Path::new(output_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| {
                    log::error!("Failed to create output dir: {}", e);
                    format!("Failed to create output dir: {}", e)
                })?;
        }

        // Determine effective format
        let effective_format = Self::effective_format(input_path, &profile.output.format);
        log::info!("  Output format: {:?}, path: {}", effective_format, output_path);

        // Encode and save
        Self::encode_and_save(&resized, output_path, effective_format, &profile.quality)?;

        let new_size = std::fs::metadata(output_path)
            .map(|m| m.len())
            .unwrap_or(0);
        log::info!(
            "  Done: {} -> {} bytes (saved {} bytes)",
            original_size,
            new_size,
            original_size.saturating_sub(new_size)
        );

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
            SizeUnit::Pixel => (settings.width.max(1), settings.height.max(1)),
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
                let compatible = Self::ensure_compatible_color(img, format);
                log::info!("  Saving with original quality as {:?}", format);
                compatible
                    .save_with_format(path, format)
                    .map_err(|e| {
                        log::error!("Save error ({}): {}", path, e);
                        format!("Save error: {}", e)
                    })?;
                return Ok(());
            }
            QualityMode::Quality => quality_settings.quality.max(1).min(100),
            QualityMode::TargetSize => {
                let target_kb = quality_settings.target_size_kb.unwrap_or(100);
                log::info!("  Saving with target size: {}KB", target_kb);
                return Self::save_with_target_size(img, path, format, target_kb);
            }
        };

        match format {
            ImageFormat::Jpeg => {
                // JPEG does not support alpha — convert RGBA to RGB first
                let rgb_img = img.to_rgb8();
                log::info!("  Encoding JPEG quality={}", quality);
                let mut buf = Vec::new();
                let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
                rgb_img
                    .write_with_encoder(encoder)
                    .map_err(|e| {
                        log::error!("JPEG encode error: {}", e);
                        format!("JPEG encode error: {}", e)
                    })?;
                std::fs::write(path, buf).map_err(|e| {
                    log::error!("JPEG write error: {}", e);
                    format!("JPEG write error: {}", e)
                })?;
            }
            ImageFormat::Png => {
                let compatible = Self::ensure_compatible_color(img, format);
                log::info!("  Encoding PNG");
                compatible
                    .save_with_format(path, ImageFormat::Png)
                    .map_err(|e| {
                        log::error!("PNG save error: {}", e);
                        format!("PNG save error: {}", e)
                    })?;
            }
            ImageFormat::WebP => {
                // WebP encoder needs 8-bit color — downconvert if needed
                let compatible = Self::ensure_compatible_color(img, format);
                log::info!("  Encoding WebP");
                compatible
                    .save_with_format(path, ImageFormat::WebP)
                    .map_err(|e| {
                        log::error!("WebP save error: {}", e);
                        format!("WebP save error: {}", e)
                    })?;
            }
            ImageFormat::Gif => {
                // GIF needs RGB8 or RGBA8 — downconvert 16-bit images
                let compatible = Self::ensure_compatible_color(img, format);
                log::info!("  Encoding GIF");
                compatible
                    .save_with_format(path, ImageFormat::Gif)
                    .map_err(|e| {
                        log::error!("GIF save error: {}", e);
                        format!("GIF save error: {}", e)
                    })?;
            }
            _ => {
                let compatible = Self::ensure_compatible_color(img, format);
                compatible
                    .save_with_format(path, format)
                    .map_err(|e| {
                        log::error!("Save error: {}", e);
                        format!("Save error: {}", e)
                    })?;
            }
        }
        Ok(())
    }

    /// Ensure image color type is compatible with the target format.
    /// Downconverts 16-bit to 8-bit and removes alpha for JPEG.
    fn ensure_compatible_color(img: &DynamicImage, format: ImageFormat) -> DynamicImage {
        // JPEG needs RGB (no alpha, no 16-bit)
        if format == ImageFormat::Jpeg {
            return img.to_rgb8().into();
        }
        // For all other formats, ensure 8-bit (WebP/GIF/PNG encoders may fail on 16-bit)
        match img {
            DynamicImage::ImageRgb16(_) => img.to_rgb8().into(),
            DynamicImage::ImageRgba16(_) => img.to_rgba8().into(),
            DynamicImage::ImageRgb32F(_) => img.to_rgb8().into(),
            DynamicImage::ImageRgba32F(_) => img.to_rgba8().into(),
            DynamicImage::ImageLuma16(_) => img.to_luma8().into(),
            DynamicImage::ImageLumaA16(_) => img.to_luma_alpha8().into(),
            _ => img.clone(),
        }
    }

    /// Binary search for quality that meets target file size
    fn save_with_target_size(
        img: &DynamicImage,
        path: &str,
        format: ImageFormat,
        target_kb: u32,
    ) -> Result<(), String> {
        let target_bytes = (target_kb as u64) * 1024;
        let rgb_img = img.to_rgb8();

        match format {
            ImageFormat::Jpeg => {
                let mut low: u8 = 1;
                let mut high: u8 = 100;
                let mut best_buf = Vec::new();

                while low <= high {
                    let mid = (low + high) / 2;
                    let mut buf = Vec::new();
                    let encoder =
                        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, mid);
                    let _ = rgb_img.write_with_encoder(encoder);

                    if buf.len() as u64 > target_bytes {
                        high = mid - 1;
                    } else {
                        best_buf = buf;
                        low = mid + 1;
                    }
                }

                if best_buf.is_empty() {
                    let mut buf = Vec::new();
                    let encoder =
                        image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 1);
                    rgb_img
                        .write_with_encoder(encoder)
                        .map_err(|e| {
                            log::error!("JPEG encode error: {}", e);
                            format!("JPEG encode error: {}", e)
                        })?;
                    std::fs::write(path, buf).map_err(|e| {
                        log::error!("JPEG write error: {}", e);
                        format!("JPEG write error: {}", e)
                    })?;
                } else {
                    std::fs::write(path, best_buf).map_err(|e| {
                        log::error!("JPEG write error: {}", e);
                        format!("JPEG write error: {}", e)
                    })?;
                }
            }
            _ => {
                let compatible = Self::ensure_compatible_color(img, format);
                compatible
                    .save_with_format(path, format)
                    .map_err(|e| {
                        log::error!("Save error (target size): {}", e);
                        format!("Save error: {}", e)
                    })?;
            }
        }
        Ok(())
    }

    /// Build the filename stem (without extension) with appropriate suffix applied.
    fn build_filename_stem(
        stem: &str,
        _original_ext: &str,
        _target_ext: &str,
        naming: &NamingMode,
        custom_suffix: &Option<String>,
    ) -> String {
        match naming {
            NamingMode::CustomSuffix => {
                let suffix = custom_suffix.as_deref().unwrap_or("");
                if suffix.is_empty() {
                    stem.to_string()
                } else {
                    format!("{}{}", stem, suffix)
                }
            }
            NamingMode::DateSuffix => {
                let date_suffix = chrono::Local::now().format("_%Y%m%d").to_string();
                format!("{}{}", stem, date_suffix)
            }
            NamingMode::KeepOriginal => stem.to_string(),
        }
    }

    /// Compute output file path based on operation mode
    pub fn compute_output_path(
        file_path: &str,
        source_dir: &str,
        output_settings: &OutputSettings,
    ) -> String {
        let path = Path::new(file_path);
        let original_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let target_ext = match &output_settings.format {
            OutputFormat::SameAsOriginal => {
                if original_ext.is_empty() {
                    "jpg".to_string()
                } else {
                    original_ext.clone()
                }
            }
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
                let new_stem = Self::build_filename_stem(
                    &stem,
                    &original_ext,
                    &target_ext,
                    &output_settings.naming,
                    &output_settings.custom_suffix,
                );
                let new_name = format!("{}.{}", new_stem, target_ext);
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
                let new_stem = Self::build_filename_stem(
                    &stem,
                    &original_ext,
                    &target_ext,
                    &output_settings.naming,
                    &output_settings.custom_suffix,
                );
                let new_relative = relative
                    .parent()
                    .map(|p| p.join(format!("{}.{}", new_stem, target_ext)))
                    .unwrap_or_else(|| PathBuf::from(format!("{}.{}", new_stem, target_ext)));
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
        let total_original_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();
        let processed_bytes = Arc::new(AtomicU64::new(0));

        let results: Vec<ProcessResult> = files
            .par_iter()
            .enumerate()
            .filter_map(|(i, file)| {
                if stop_flag.load(Ordering::Relaxed) {
                    processed_bytes.fetch_add(file.size_bytes, Ordering::Relaxed);
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
                    Err(e) => {
                        log::error!("FAILED [{}]: {}", file.path, e);
                        ProcessResult {
                            file: file.path.clone(),
                            original_size: file.size_bytes,
                            new_size: 0,
                            status: format!("failed: {}", e),
                        }
                    }
                };

                let bytes = processed_bytes.fetch_add(file.size_bytes, Ordering::Relaxed) + file.size_bytes;

                progress_callback(ProgressEvent {
                    total,
                    current: (i + 1) as u32,
                    file: file.path.clone(),
                    original_size: result.original_size,
                    new_size: result.new_size,
                    status: result.status.clone(),
                    total_original_bytes,
                    processed_bytes: bytes,
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
        let img = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(width, height, Rgb([255u8, 0, 0]));
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

        assert_eq!(w, 50);
        assert_eq!(h, 50);
    }

    #[test]
    fn test_compute_output_path_same_dir() {
        let settings = OutputSettings {
            operation: OutputOperation::SameDir,
            custom_dir: None,
            format: OutputFormat::SameAsOriginal,
            naming: NamingMode::DateSuffix,
            custom_suffix: None,
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert!(!output.contains("_compressed"));
        assert!(output.ends_with(".jpg"));
        let filename = std::path::Path::new(&output)
            .file_name()
            .unwrap()
            .to_string_lossy();
        assert_ne!(filename.to_string(), "001.jpg");
    }

    #[test]
    fn test_compute_output_path_same_dir_date_suffix() {
        let settings = OutputSettings {
            operation: OutputOperation::SameDir,
            custom_dir: None,
            format: OutputFormat::SameAsOriginal,
            naming: NamingMode::DateSuffix,
            custom_suffix: None,
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert!(output.contains("001_20260425") || output.contains("001_2026"));
        assert!(output.ends_with(".jpg"));
        assert!(!output.contains("_compressed"));
    }

    #[test]
    fn test_compute_output_path_same_dir_custom_suffix() {
        let settings = OutputSettings {
            operation: OutputOperation::SameDir,
            custom_dir: None,
            format: OutputFormat::SameAsOriginal,
            naming: NamingMode::CustomSuffix,
            custom_suffix: Some("_mini".to_string()),
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert!(output.contains("001_mini"));
        assert!(output.ends_with(".jpg"));
    }

    #[test]
    fn test_compute_output_path_same_dir_keep_original() {
        let settings = OutputSettings {
            operation: OutputOperation::SameDir,
            custom_dir: None,
            format: OutputFormat::Jpeg,
            naming: NamingMode::KeepOriginal,
            custom_suffix: None,
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.png",
            "C:\\comics",
            &settings,
        );

        // KeepOriginal + format change → different extension, no suffix needed
        assert!(output.contains("001.jpg"));
        assert!(!output.contains("_compressed"));
    }

    #[test]
    fn test_compute_output_path_same_dir_keep_original_no_suffix() {
        let settings = OutputSettings {
            operation: OutputOperation::SameDir,
            custom_dir: None,
            format: OutputFormat::SameAsOriginal,
            naming: NamingMode::KeepOriginal,
            custom_suffix: None,
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert_eq!(output, "C:\\comics\\vol1\\001.jpg", "KeepOriginal should keep original filename without suffix");
    }

    #[test]
    fn test_compute_output_path_custom_dir_date_suffix() {
        let settings = OutputSettings {
            operation: OutputOperation::CustomDir,
            custom_dir: Some("C:\\output".to_string()),
            format: OutputFormat::WebP,
            naming: NamingMode::DateSuffix,
            custom_suffix: None,
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert!(output.starts_with("C:\\output"));
        assert!(output.contains("vol1"));
        assert!(output.ends_with(".webp"));
        assert!(output.contains("001_20260425") || output.contains("001_2026"));
    }

    #[test]
    fn test_compute_output_path_overwrite_no_suffix() {
        let settings = OutputSettings {
            operation: OutputOperation::Overwrite,
            custom_dir: None,
            format: OutputFormat::SameAsOriginal,
            naming: NamingMode::KeepOriginal,
            custom_suffix: None,
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert_eq!(output, "C:\\comics\\vol1\\001.jpg");
    }

    #[test]
    fn test_compute_output_path_empty_custom_suffix_keeps_original() {
        let settings = OutputSettings {
            operation: OutputOperation::SameDir,
            custom_dir: None,
            format: OutputFormat::SameAsOriginal,
            naming: NamingMode::CustomSuffix,
            custom_suffix: Some("".to_string()),
        };

        let output = ImageProcessor::compute_output_path(
            "C:\\comics\\vol1\\001.jpg",
            "C:\\comics",
            &settings,
        );

        assert_eq!(output, "C:\\comics\\vol1\\001.jpg", "Empty custom suffix should keep original filename");
    }

    #[test]
    fn test_compute_output_path_custom_dir() {
        let settings = OutputSettings {
            operation: OutputOperation::CustomDir,
            custom_dir: Some("C:\\output".to_string()),
            format: OutputFormat::WebP,
            naming: NamingMode::KeepOriginal,
            custom_suffix: None,
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
                naming: NamingMode::KeepOriginal,
                custom_suffix: None,
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
                naming: NamingMode::KeepOriginal,
                custom_suffix: None,
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
