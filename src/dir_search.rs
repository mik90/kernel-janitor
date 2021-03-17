use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Finds all files with a prefix in a directory
pub fn all_paths_with_prefix(prefix: &str, dir: &Path) -> io::Result<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = fs::read_dir(dir)?
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with(prefix)
        })
        .collect();
    Ok(paths)
}

/// From a full path, just get the `file_name()` as a String
pub fn filename_from_path(path: &Path) -> Option<String> {
    let os_str = match path.file_name() {
        Some(f) => f.to_string_lossy(),
        None => return None,
    };
    Some(os_str.to_string())
}

/// Finds all files with a prefix in a directory
pub fn all_paths(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let paths: Vec<PathBuf> = fs::read_dir(dir)?
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect();
    Ok(paths)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::tests::*;

    fn create_dummy_files() -> io::Result<()> {
        init_test_dir();
        let test_dir = get_test_pathbuf();

        let mut path_0 = test_dir.clone();
        path_0.push("old-file.txt");
        fs::write(path_0, "old-file")?;

        let mut path_1 = test_dir.clone();
        path_1.push("new-file.txt");
        fs::write(path_1, "new-file")?;

        Ok(())
    }
    #[test]
    fn test_setup_cleanup() {
        init_test_dir();
        cleanup_test_dir();
    }
    #[test]
    fn find_entries() {
        init_test_dir();
        let res = create_dummy_files();
        assert!(res.is_ok());

        let search_dir = get_test_pathbuf();
        let res = all_paths_with_prefix("new", &search_dir);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 1);

        cleanup_test_dir();
    }

    #[test]
    fn test_filename_from_path() {
        let path = Path::new("/tmp/some/path/a-filename.txt");
        let filename = filename_from_path(path);
        assert!(filename.is_some());
        let filename = filename.unwrap();
        assert_eq!(filename.as_str(), "a-filename.txt");
    }
}
