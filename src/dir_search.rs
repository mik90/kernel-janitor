use std::{
    fs,
    fs::ReadDir,
    io,
    path::{Path, PathBuf},
};

/// Finds all files with a prefix in a directory
pub fn all_entries_with_prefix(prefix: &str, dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .starts_with(prefix)
        {
            entries.push(path);
        }
    }
    Ok(entries)
}

/// Finds all files with a prefix in a directory
pub fn all_entries(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        entries.push(entry?.path());
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {

    use super::*;

    fn get_test_dir() -> PathBuf {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("UnknownThreadName");
        let thread_name_cleaned = thread_name.to_string().replace(":", "_");
        PathBuf::from(format!("unit-test-temp/{}", thread_name_cleaned))
    }

    fn setup_test_dir() -> io::Result<()> {
        let test_dir = get_test_dir();
        fs::create_dir_all(&test_dir)?;

        let mut path_0 = test_dir.clone();
        path_0.push("old-file.txt");
        fs::write(path_0, "old-file")?;

        let mut path_1 = test_dir.clone();
        path_1.push("new-file.txt");
        fs::write(path_1, "new-file")?;

        Ok(())
    }

    fn cleanup_test_dir() -> io::Result<()> {
        let test_dir = get_test_dir();
        for entry in fs::read_dir(test_dir)? {
            let path = entry?.path();

            if path.is_file() {
                println!("Attempting to delete file {:?}", path);
                fs::remove_file(path)?;
            } else if path.is_dir() {
                println!("Attempting to delete dir {:?}", path);
                fs::remove_dir_all(path)?;
            } else {
                println!("Unknown file type {:?}", path);
            }
        }
        Ok(())
    }
    #[test]
    fn test_setup_cleanup() {
        let setup_res = setup_test_dir();
        let cleanup_res = cleanup_test_dir();

        assert!(setup_res.is_ok());
        assert!(cleanup_res.is_ok());
    }
    #[test]
    fn find_entries() {
        assert!(setup_test_dir().is_ok());

        let search_dir = get_test_dir();
        let res = all_entries_with_prefix("new", &search_dir);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 1);

        assert!(cleanup_test_dir().is_ok());
    }
}
