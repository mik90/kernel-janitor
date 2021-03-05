use std::{
    borrow::Borrow, cmp::Ordering, collections::HashMap, convert::TryFrom, io, option::Option,
    path::Path, path::PathBuf,
};

use crate::dir_search;

/// A kernel version can be found as a config, vmlinuz binary, system map, or source directory.
/// Format: SomeIgnoredValue-<major>.<minor>.<patch>-gentoo
///         or SomeIgnoredValue-<major>.<minor>.<patch>-rc<release_candidate_num>-gentoo
///         or SomeIgnoredValue-<major>.<minor>.<patch>-gentoo.old
#[derive(Hash, Eq, Debug)]
struct KernelVersion {
    major: u32,
    minor: u32,
    patch: u32,
    release_candidate_num: Option<u32>,
    is_old: bool,
}
struct InstalledKernel {
    pub version: KernelVersion,
    pub module_path: Option<PathBuf>,
    pub source_path: Option<PathBuf>,
    pub vmlinuz_path: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub system_map_path: Option<PathBuf>,
}

struct KernelSearch {
    module_search_path: PathBuf,
    source_search_path: PathBuf,
    // Expect to find vmlinuz, config, and system map in this search path
    install_search_path: PathBuf,
}

impl KernelVersion {
    pub fn new(
        major: u32,
        minor: u32,
        patch: u32,
        release_candidate_num: Option<u32>,
        is_old: bool,
    ) -> KernelVersion {
        KernelVersion {
            major,
            minor,
            patch,
            release_candidate_num,
            is_old,
        }
    }
    /// Returns major, minor, patch versions
    pub fn version_triple(&self) -> (u32, u32, u32) {
        (self.major, self.minor, self.patch)
    }

    pub fn release_candidate_num(&self) -> Option<u32> {
        self.release_candidate_num
    }
    pub fn is_old(&self) -> bool {
        self.is_old
    }
}
impl TryFrom<&str> for KernelVersion {
    type Error = std::num::ParseIntError;

    fn try_from(raw_value: &str) -> Result<Self, Self::Error> {
        // linux-5.7.11-rc10-gentoo.old
        // -> ['linux', '5.7.11', 'rc10', 'gentoo.old']
        //        0        1         2          3
        let split_by_dash: Vec<&str> = raw_value.split('-').collect();

        // Collect the first 3 items or return in error
        // ['major', 'minor', 'patch']
        let version_triple: Result<Vec<_>, _> = split_by_dash[1]
            .split('.')
            .into_iter()
            .take(3)
            .map(|x| x.parse::<u32>())
            .collect();
        if version_triple.is_err() {
            return Err(version_triple.unwrap_err());
        }
        let version_triple = version_triple.unwrap();

        let is_old = raw_value.ends_with(".old");

        // release candidate
        let release_candidate_num = split_by_dash[2]
            .strip_prefix("rc")
            .unwrap_or_default()
            .parse::<u32>()
            .ok();

        Ok(KernelVersion {
            major: version_triple[0],
            minor: version_triple[1],
            patch: version_triple[2],
            release_candidate_num: release_candidate_num,
            is_old: is_old,
        })
    }
}
impl TryFrom<String> for KernelVersion {
    type Error = std::num::ParseIntError;

    fn try_from(raw_value: String) -> Result<Self, Self::Error> {
        KernelVersion::try_from(raw_value.as_str())
    }
}

/// We're just comparing versions for the sake of ordering them
impl Ord for KernelVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.major,
            self.minor,
            self.patch,
            self.release_candidate_num,
            !self.is_old,
        )
            .cmp(&(
                other.major,
                other.minor,
                other.patch,
                other.release_candidate_num,
                !other.is_old,
            ))
    }
}

impl PartialOrd for KernelVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for KernelVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.is_old == other.is_old
            // Ensure both do or don't have a release candidate
            && self.release_candidate_num.is_some() == other.release_candidate_num.is_some()
            // At this point,they both have or don't have a release candidate number.
            // Compare values, default to zero. If neither have it they'll be equal.
            // If they both have it, they'll unwrap valid values.
            && self.release_candidate_num.unwrap_or(0) == other.release_candidate_num.unwrap_or(0)
    }
}

impl InstalledKernel {
    pub fn new(
        version: KernelVersion,
        module_path: Option<PathBuf>,
        vmlinuz_path: Option<PathBuf>,
        source_path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        system_map_path: Option<PathBuf>,
    ) -> InstalledKernel {
        InstalledKernel {
            version,
            module_path,
            vmlinuz_path,
            source_path,
            config_path,
            system_map_path,
        }
    }

    /// True if any of the paths are empty (not found)
    /// False if all paths are Some
    pub fn files_missing(&self) -> bool {
        self.module_path.is_none()
            || self.vmlinuz_path.is_none()
            || self.source_path.is_none()
            || self.config_path.is_none()
            || self.system_map_path.is_none()
    }
}

enum InstalledItem {
    KernelImage(PathBuf),
    Config(PathBuf),
    SystemMap(PathBuf),
    SourceDir(PathBuf),
    ModuleDir(PathBuf),
}

impl KernelSearch {
    pub fn new() -> KernelSearch {
        KernelSearch {
            module_search_path: PathBuf::from("/lib/modules"),
            source_search_path: PathBuf::from("/usr/src"),
            install_search_path: PathBuf::from("/boot/EFI/Gentoo"),
        }
    }

    /// Reference: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html#non-consuming-builders-(preferred):
    pub fn module_search_path<'a>(&'a mut self, dir: PathBuf) -> &'a mut KernelSearch {
        self.module_search_path = dir;
        self
    }

    pub fn source_search_path<'a>(&'a mut self, dir: PathBuf) -> &'a mut KernelSearch {
        self.source_search_path = dir;
        self
    }

    pub fn install_search_path<'a>(&'a mut self, dir: PathBuf) -> &'a mut KernelSearch {
        self.install_search_path = dir;
        self
    }

    fn add_item_to_map(
        item: InstalledItem,
        map: &mut HashMap<KernelVersion, InstalledKernel>,
    ) -> io::Result<()> {
        // - Create a (KernelVersion, path) with each result in a search
        // - Check if that KernelVersion is already present as an InstalledKernel
        //   - If it is, add the path to the InstalledKernel
        //   - otherwise, create a new InstalledKernel with the pair
        match item {
            InstalledItem::SystemMap(pathbuf) => {
                let filename = dir_search::filename_from_path(&pathbuf)?;

                let err = io::Error::new(
                    io::ErrorKind::Other,
                    format!("Could not parse {:?} as a KernelVersion", filename),
                );
                let version = KernelVersion::try_from(filename).map_err(|_| err)?;
                match map.get_mut(&version) {
                    Some(installed_kernel) => installed_kernel.system_map_path = Some(pathbuf),
                    None => {
                        map.insert(
                            version,
                            InstalledKernel::new(version, None, None, None, None, Some(pathbuf)),
                        );
                    }
                }
            }
            InstalledItem::KernelImage(p) => {}
            InstalledItem::Config(p) => {}
            InstalledItem::SourceDir(p) => {}
            InstalledItem::ModuleDir(p) => {}
        }
        Ok(())
    }

    pub fn run(&self) -> io::Result<Vec<InstalledKernel>> {
        let mut kernel_map: HashMap<KernelVersion, InstalledKernel> = HashMap::new();

        // Search for vmlinuz
        let kernel_images: Vec<_> =
            dir_search::all_paths_with_prefix("vmlinuz-", &self.install_search_path)?
                .into_iter()
                .map(|path| InstalledItem::KernelImage(path))
                .collect();

        // Search for config
        let configs: Vec<_> =
            dir_search::all_paths_with_prefix("config-", &self.install_search_path)?
                .into_iter()
                .map(|path| InstalledItem::Config(path))
                .collect();

        // Search for system map
        let system_maps: Vec<_> =
            dir_search::all_paths_with_prefix("System.map-", &self.install_search_path)?
                .into_iter()
                .map(|path| InstalledItem::SystemMap(path))
                .collect();

        // Search for source dir
        let source_dirs: Vec<_> =
            dir_search::all_paths_with_prefix("linux", &self.source_search_path)?
                .into_iter()
                .map(|path| InstalledItem::SourceDir(path))
                .collect();

        // Search for module path
        let module_dirs: Vec<_> = dir_search::all_paths(&self.module_search_path)?
            .into_iter()
            .map(|path| InstalledItem::ModuleDir(path))
            .collect();

        Ok(Vec::new())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_kernel_version() {
        let ver = KernelVersion::try_from("linux-5.7.11-gentoo");
        assert!(ver.is_ok());

        let ver = ver.unwrap();
        assert_eq!(ver.version_triple(), (5, 7, 11));
        assert!(ver.release_candidate_num().is_none());
        assert_eq!(ver.is_old(), false);
    }

    #[test]
    fn create_kernel_version_old() {
        let ver = KernelVersion::try_from("linux-2.6.999-gentoo.old");
        assert!(ver.is_ok());

        let ver = ver.unwrap();
        assert_eq!(ver.version_triple(), (2, 6, 999));
        assert!(ver.release_candidate_num().is_none());
        assert_eq!(ver.is_old(), true);
    }
    #[test]
    fn create_kernel_version_rc() {
        let ver = KernelVersion::try_from("linux-2.6.999-rc1234-gentoo.old");
        assert!(ver.is_ok());

        let ver = ver.unwrap();
        assert_eq!(ver.version_triple(), (2, 6, 999));
        assert_eq!(ver.is_old(), true);
        assert!(ver.release_candidate_num().is_some());
        assert_eq!(ver.release_candidate_num().unwrap(), 1234);
    }

    #[test]
    fn kernel_not_equal() {
        let error_msg = "Could not construct test KernelVersion!";
        let newer = KernelVersion::try_from("linux-4.10.5-gentoo").expect(error_msg);
        let older = KernelVersion::try_from("linux-4.10.0-gentoo").expect(error_msg);
        assert_ne!(newer, older);
    }

    #[test]
    fn kernel_equal() {
        let error_msg = "Could not construct test KernelVersion!";
        let ver = KernelVersion::try_from("linux-2.6.0-gentoo").expect(error_msg);
        assert_eq!(ver, ver);
    }

    #[test]
    fn kernel_greater_than() {
        let error_msg = "Could not construct test KernelVersion!";
        let newer = KernelVersion::try_from("linux-4.10.5-gentoo").expect(error_msg);
        let older = KernelVersion::try_from("linux-4.10.0-gentoo").expect(error_msg);
        assert!(newer > older);
    }

    #[test]
    fn kernel_invalid() {
        let invalid = KernelVersion::try_from("SoYouThink-ImAKernel");
        assert!(invalid.is_err());
    }

    #[test]
    fn kernel_version_from_config() {
        let valid = KernelVersion::try_from("config-5.11.0-gentoo");
        assert!(valid.is_ok());
    }
    #[test]
    fn kernel_version_from_system_map() {
        let valid = KernelVersion::try_from("System.map-5.11.0-gentoo");
        assert!(valid.is_ok());
    }

    #[test]
    fn kernel_version_from_vmlinuz() {
        let valid = KernelVersion::try_from("vmlinuz-5.11.0-gentoo");
        assert!(valid.is_ok());
    }
    #[test]
    fn order_kernel_versions() {
        let error_msg = "Could not construct test KernelVersion!";
        let mut versions: Vec<KernelVersion> = vec![
            KernelVersion::try_from("linux-4.10.5-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-4.10.0-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-4.10.5-gentoo.old").expect(error_msg),
            KernelVersion::try_from("linux-5.11.0-rc1-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-2.6.0-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-4.10.0-rc8-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-4.10.0-rc8-gentoo.old").expect(error_msg),
            KernelVersion::try_from("linux-4.10.0-rc1-gentoo.old").expect(error_msg),
        ];

        // Ascending sort
        let sorted_versions: Vec<KernelVersion> = vec![
            KernelVersion::try_from("linux-2.6.0-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-4.10.0-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-4.10.0-rc1-gentoo.old").expect(error_msg),
            KernelVersion::try_from("linux-4.10.0-rc8-gentoo.old").expect(error_msg),
            KernelVersion::try_from("linux-4.10.0-rc8-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-4.10.5-gentoo.old").expect(error_msg),
            KernelVersion::try_from("linux-4.10.5-gentoo").expect(error_msg),
            KernelVersion::try_from("linux-5.11.0-rc1-gentoo").expect(error_msg),
        ];

        assert_eq!(versions.len(), sorted_versions.len());

        versions.sort();
        println!("versions.sort():");
        for ver in versions.iter() {
            println!("    {:?}", ver);
        }
        println!("sorted_versions:");
        for ver in sorted_versions.iter() {
            println!("    {:?}", ver);
        }

        let zipped = versions.iter().zip(sorted_versions.iter());
        for vers in zipped {
            assert_eq!(vers.0, vers.1);
        }
    }

    #[test]
    fn files_missing_true() {
        let version = KernelVersion::new(2, 6, 0, None, false);
        let installed_kernel = InstalledKernel::new(
            version,
            Some(PathBuf::from("./temp")),
            Some(PathBuf::from("./temp")),
            Some(PathBuf::from("./temp")),
            Some(PathBuf::from("./temp")),
            None,
        );
        assert_eq!(installed_kernel.files_missing(), true);
    }

    #[test]
    fn files_missing_false() {
        let version = KernelVersion::new(2, 6, 0, None, false);
        let installed_kernel = InstalledKernel::new(
            version,
            Some(PathBuf::from("./temp")),
            Some(PathBuf::from("./temp")),
            Some(PathBuf::from("./temp")),
            Some(PathBuf::from("./temp")),
            Some(PathBuf::from("./temp")),
        );
        assert_eq!(installed_kernel.files_missing(), false);
    }
}
