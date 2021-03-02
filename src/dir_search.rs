use std::{
    fs,
    fs::ReadDir,
    io,
    path::{Path, PathBuf},
};

/// Finds all files with a prefix in a directory
pub fn find_all_entries_with_prefix(prefix: &str, dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.to_string_lossy().starts_with(prefix) {
            entries.push(path);
        }
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_dir_with_entries() -> io::Result<()> {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("UnknownThreadName");
        let test_dir = PathBuf::from(format!("unit-test-run/{}", thread_name));
        fs::create_dir_all(&test_dir)?;

        // TODO Write two entries in the directory with different prefixes
        Ok(())
    }
    #[test]
    fn find_entries() {}
}
