use std::{fs::File, io, path::{Path, PathBuf}};

use anyhow::{Context, Result};
use chrono::Local;
use walkdir::WalkDir;
use zip::{ZipWriter, write::{SimpleFileOptions}};

pub fn archive_project(dir: &Path) -> Result<PathBuf> {

    let project_name = dir
        .file_name()
        .context("invalid project dir")?
        .to_string_lossy();

    let archive_name = format!("{project_name}_stems.zip");
    let archive_path = dir.join(&archive_name);

    let file = File::create(&archive_path)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let audio_dir = dir.join("Audio");

    for entry in WalkDir::new(&audio_dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().map(|e| e == "wav").unwrap_or(false) {
            let name_in_zip  = path
                .strip_prefix(dir)?
                .to_string_lossy();

            zip.start_file(name_in_zip, options)?;

            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        }
    }

    zip.finish()?;

    Ok(archive_path)
}
