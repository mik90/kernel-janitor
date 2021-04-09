pub fn user_is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

pub mod paths {
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
}

#[cfg(test)]
pub mod tests {

    use super::paths::*;
    use std::{
        fs, io,
        path::{Path, PathBuf},
    };

    fn create_dummy_files() -> io::Result<()> {
        init_test_dir();
        let test_dir = get_test_install_pathbuf();

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

        let search_dir = get_test_install_pathbuf();
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

    pub fn init_test_dir() {
        let test_dir = get_test_install_pathbuf();
        let res = fs::create_dir_all(&test_dir);
        assert!(res.is_ok(), "Could not init test dir {:?}", test_dir);
    }

    pub fn get_test_install_path_string() -> String {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("UnknownThreadName");
        let thread_name_cleaned = thread_name.to_string().replace(":", "_");
        format!("unit-test-temp/{}", thread_name_cleaned)
    }

    pub fn get_test_install_pathbuf() -> PathBuf {
        PathBuf::from(get_test_install_path_string())
    }

    pub fn get_test_module_path_string() -> String {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("UnknownThreadName");
        let thread_name_cleaned = thread_name.to_string().replace(":", "_");
        format!("unit-test-temp/{}/modules", thread_name_cleaned)
    }

    pub fn get_test_module_pathbuf() -> PathBuf {
        PathBuf::from(get_test_module_path_string())
    }

    pub fn get_test_src_path_string() -> String {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("UnknownThreadName");
        let thread_name_cleaned = thread_name.to_string().replace(":", "_");
        format!("unit-test-temp/{}/src", thread_name_cleaned)
    }
    pub fn get_test_src_pathbuf() -> PathBuf {
        PathBuf::from(get_test_src_path_string())
    }

    pub fn cleanup_test_dir() {
        let test_dir = get_test_install_pathbuf();
        if !test_dir.exists() {
            return;
        }
        let dir_entry = fs::read_dir(&test_dir);
        assert!(dir_entry.is_ok(), "Could not read dir {:?}", test_dir);
        let dir_entry = dir_entry.unwrap();

        for entry in dir_entry {
            assert!(entry.is_ok(), "Could open DirEntry {:?}", entry);
            let path = entry.unwrap().path();

            if path.is_file() {
                println!("Attempting to delete file {:?}", path);
                let res = fs::remove_file(&path);
                assert!(res.is_ok(), "Could not delete file {:?}", path);
            } else if path.is_dir() {
                println!("Attempting to delete dir {:?}", path);
                let res = fs::remove_dir_all(&path);
                assert!(res.is_ok(), "Could not delete dir {:?}", path);
            } else {
                println!("Unknown file type {:?}", path);
            }
        }
    }
}
