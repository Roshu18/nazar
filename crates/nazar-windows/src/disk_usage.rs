use std::fs;
use std::path::{Path, PathBuf};

pub fn measure_readable_size(path: &Path) -> u64 {
    let mut total_size_in_bytes = 0;
    let mut folders_to_visit: Vec<PathBuf> = Vec::new();
    add_entry(path, &mut total_size_in_bytes, &mut folders_to_visit);
    while let Some(folder) = folders_to_visit.pop() {
        let Ok(entries) = fs::read_dir(&folder) else {
            continue;
        };
        for entry in entries.flatten() {
            add_entry(
                &entry.path(),
                &mut total_size_in_bytes,
                &mut folders_to_visit,
            );
        }
    }
    total_size_in_bytes
}

fn add_entry(path: &Path, total_size_in_bytes: &mut u64, folders_to_visit: &mut Vec<PathBuf>) {
    let Ok(metadata) = path.symlink_metadata() else {
        return;
    };
    if metadata.file_type().is_symlink() {
        return;
    }
    if metadata.is_dir() {
        folders_to_visit.push(path.to_owned());
    } else {
        *total_size_in_bytes += metadata.len();
    }
}
