use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use walkdir::WalkDir;
use zip::read::ZipArchive;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

/// Extract a ZIP file to a destination directory
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = File::open(zip_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read ZIP entry: {}", e))?;

        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create parent directory: {}", e))?;
                }
            }
            let mut outfile =
                File::create(&outpath).map_err(|e| format!("Failed to create file: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to extract file: {}", e))?;
        }
    }

    Ok(())
}

/// Create a ZIP file from a source directory
pub fn create_zip(source_dir: &Path, zip_path: &Path) -> Result<(), String> {
    let file = File::create(zip_path).map_err(|e| format!("Failed to create ZIP file: {}", e))?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(CompressionMethod::Deflated);

    let source_dir_str = source_dir.to_str().ok_or("Invalid source directory path")?;

    for entry in WalkDir::new(source_dir) {
        let entry = entry.map_err(|e| format!("Failed to walk directory: {}", e))?;
        let path = entry.path();

        // Get the relative path for the ZIP entry
        let relative_path = path
            .strip_prefix(source_dir_str)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;

        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let name = relative_path.to_str().ok_or("Invalid path encoding")?;

        if path.is_dir() {
            zip.add_directory(name, options)
                .map_err(|e| format!("Failed to add directory to ZIP: {}", e))?;
        } else {
            zip.start_file(name, options)
                .map_err(|e| format!("Failed to start file in ZIP: {}", e))?;

            let mut file =
                File::open(path).map_err(|e| format!("Failed to open file for ZIP: {}", e))?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)
                .map_err(|e| format!("Failed to read file for ZIP: {}", e))?;
            zip.write_all(&buffer)
                .map_err(|e| format!("Failed to write file to ZIP: {}", e))?;
        }
    }

    zip.finish()
        .map_err(|e| format!("Failed to finalize ZIP: {}", e))?;

    Ok(())
}
