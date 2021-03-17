#[cfg(test)]
pub mod tests {

    use std::{fs, path::PathBuf};

    pub fn init_test_dir() {
        let test_dir = get_test_pathbuf();
        let res = fs::create_dir_all(&test_dir);
        assert!(
            res.is_ok(),
            format!("Could not init test dir {:?}", test_dir)
        );
    }

    pub fn get_test_path_string() -> String {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("UnknownThreadName");
        let thread_name_cleaned = thread_name.to_string().replace(":", "_");
        format!("unit-test-temp/{}", thread_name_cleaned)
    }

    pub fn get_test_pathbuf() -> PathBuf {
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("UnknownThreadName");
        let thread_name_cleaned = thread_name.to_string().replace(":", "_");
        PathBuf::from(format!("unit-test-temp/{}", thread_name_cleaned))
    }

    pub fn cleanup_test_dir() {
        let test_dir = get_test_pathbuf();
        if !test_dir.exists() {
            return;
        }
        let dir_entry = fs::read_dir(&test_dir);
        assert!(
            dir_entry.is_ok(),
            format!("Could not read dir {:?}", test_dir)
        );
        let dir_entry = dir_entry.unwrap();

        for entry in dir_entry {
            assert!(entry.is_ok(), format!("Could open DirEntry {:?}", entry));
            let path = entry.unwrap().path();

            if path.is_file() {
                println!("Attempting to delete file {:?}", path);
                let res = fs::remove_file(&path);
                assert!(res.is_ok(), format!("Could not delete file {:?}", path));
            } else if path.is_dir() {
                println!("Attempting to delete dir {:?}", path);
                let res = fs::remove_dir_all(&path);
                assert!(res.is_ok(), format!("Could not delete dir {:?}", path));
            } else {
                println!("Unknown file type {:?}", path);
            }
        }
    }
}
