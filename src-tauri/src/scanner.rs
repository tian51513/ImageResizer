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

        let (width, height) = image::image_dimensions(entry.path())
            .unwrap_or((0, 0));

        files.push(FileMetadata {
            path: entry.path().to_string_lossy().to_string(),
            size_bytes: size,
            extension: ext,
            width,
            height,
        });
    }

    Ok(files)
}

#[cfg(test)]
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

#[cfg(test)]
pub fn total_size(files: &[FileMetadata]) -> u64 {
    files.iter().map(|f| f.size_bytes).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dir(test_name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("imageresizer_scanner_test_{}", test_name));
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir.join("subdir"));

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
        let dir = setup_test_dir("finds_images");
        let files = scan_directory(&dir).unwrap();
        cleanup_test_dir(&dir);

        assert_eq!(files.len(), 5);
    }

    #[test]
    fn test_scan_recurses_subdirs() {
        let dir = setup_test_dir("recurses");
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
    fn test_scan_reads_image_dimensions() {
        use image::{ImageBuffer, Rgb};

        let dir = std::env::temp_dir().join("imageresizer_scanner_test_dimensions");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);

        // Create real images with known dimensions
        let img_100x50 = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(100, 50, Rgb([255, 0, 0]));
        img_100x50.save(dir.join("photo.jpg")).unwrap();
        let img_200x300 = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(200, 300, Rgb([0, 255, 0]));
        img_200x300.save(dir.join("image.png")).unwrap();

        let files = scan_directory(&dir.to_string_lossy()).unwrap();
        cleanup_test_dir(&dir.to_string_lossy());

        assert_eq!(files.len(), 2);

        let jpg = files.iter().find(|f| f.extension == "jpg").unwrap();
        assert_eq!(jpg.width, 100);
        assert_eq!(jpg.height, 50);

        let png = files.iter().find(|f| f.extension == "png").unwrap();
        assert_eq!(png.width, 200);
        assert_eq!(png.height, 300);
    }

    #[test]
    fn test_scan_returns_zero_dimensions_for_corrupt_file() {
        let dir = std::env::temp_dir().join("imageresizer_scanner_test_corrupt");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::create_dir_all(&dir);

        // Write a valid extension but corrupt content
        fs::write(dir.join("corrupt.jpg"), "not_a_real_jpeg").unwrap();

        let files = scan_directory(&dir.to_string_lossy()).unwrap();
        cleanup_test_dir(&dir.to_string_lossy());

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].width, 0);
        assert_eq!(files[0].height, 0);
    }

    #[test]
    fn test_filter_by_extension() {
        let files = vec![
            FileMetadata {
                path: "a.jpg".to_string(),
                size_bytes: 100,
                extension: "jpg".to_string(),
                width: 0,
                height: 0,
            },
            FileMetadata {
                path: "b.png".to_string(),
                size_bytes: 200,
                extension: "png".to_string(),
                width: 0,
                height: 0,
            },
            FileMetadata {
                path: "c.gif".to_string(),
                size_bytes: 300,
                extension: "gif".to_string(),
                width: 0,
                height: 0,
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
                width: 0,
                height: 0,
            },
            FileMetadata {
                path: "b.png".to_string(),
                size_bytes: 2048,
                extension: "png".to_string(),
                width: 0,
                height: 0,
            },
        ];

        assert_eq!(total_size(&files), 3072);
    }
}
