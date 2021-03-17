use std::{
    borrow::Borrow, cmp::Ordering, collections::HashMap, convert::TryFrom, fmt, io, option::Option,
    os::raw, path::Path, path::PathBuf,
};

use crate::dir_search;

/// A kernel version can be found as a config, vmlinuz binary, system map, or source directory.
/// Format: SomeIgnoredValue-<major>.<minor>.<patch>-gentoo
///         or SomeIgnoredValue-<major>.<minor>.<patch>-rc<release_candidate_num>-gentoo
///         or SomeIgnoredValue-<major>.<minor>.<patch>-gentoo.old
#[derive(Hash, Eq, Debug, Clone, Copy)]
pub struct KernelVersion {
    major: u32,
    minor: u32,
    patch: u32,
    release_candidate_num: Option<u32>,
    is_old: bool,
}
enum InstalledItemKind {
    KernelImage,
    Config,
    SystemMap,
    SourceDir,
    ModuleDir,
}

pub struct InstalledKernel {
    pub version: KernelVersion,
    pub module_path: Option<PathBuf>,
    pub source_path: Option<PathBuf>,
    pub vmlinuz_path: Option<PathBuf>,
    pub config_path: Option<PathBuf>,
    pub system_map_path: Option<PathBuf>,
}

pub struct KernelSearch {
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
    type Error = Box<dyn std::error::Error>;

    fn try_from(raw_value: &str) -> Result<Self, Self::Error> {
        let first_char = raw_value.chars().nth(0);
        let first_char_is_num = match first_char {
            Some(c) => c.is_numeric(),
            None => return Err("Could not parse empty string as KernelVersion".into()),
        };

        //  Skip the first item if the string doesn't start with a number
        let (version_triple, release_candidate_num) = if first_char_is_num {
            // Example modules dir:
            //  5.7.11-rc10-gentoo
            // -> ['5.7.11', 'rc10', 'gentoo']
            //        0        1         2
            let split_by_dash: Vec<&str> = raw_value.split('-').collect();
            if split_by_dash.len() < 2 {
                return Err(format!("Could not parse {} as KernelVersion!", raw_value).into());
            }
            (split_by_dash[0], split_by_dash[1])
        } else {
            // Example linux src dir:
            // linux-5.7.11-rc10-gentoo.old
            // -> ['linux', '5.7.11', 'rc10', 'gentoo.old']
            //        0        1         2          3
            let split_by_dash: Vec<&str> = raw_value.split('-').collect();
            if split_by_dash.len() < 3 {
                return Err(format!("Could not parse {} as KernelVersion!", raw_value).into());
            }
            (split_by_dash[1], split_by_dash[2])
        };

        // Collect the first 3 items or return in error
        // ['major', 'minor', 'patch']
        let version_triple: Result<Vec<_>, _> = version_triple
            .split('.')
            .into_iter()
            .take(3)
            .map(|x| x.parse::<u32>())
            .collect();
        if version_triple.is_err() {
            return Err(version_triple.unwrap_err().into());
        }
        let version_triple = version_triple.unwrap();

        let is_old = raw_value.ends_with(".old");

        // release candidate
        let release_candidate_num = release_candidate_num
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
    type Error = Box<dyn std::error::Error>;

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
impl fmt::Display for KernelVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut postfix = String::new();
        if let Some(n) = self.release_candidate_num {
            postfix.push_str("-rc");
            postfix.push_str(&n.to_string());
        }
        if self.is_old {
            postfix.push_str(".old");
        }
        write!(f, "{}.{}.{}{}", self.major, self.minor, self.patch, postfix)
    }
}

impl InstalledKernel {
    pub fn new(version: KernelVersion) -> InstalledKernel {
        InstalledKernel {
            version,
            module_path: None,
            source_path: None,
            vmlinuz_path: None,
            config_path: None,
            system_map_path: None,
        }
    }

    pub fn with_module_path(mut self, dir: PathBuf) -> InstalledKernel {
        self.module_path = Some(dir);
        self
    }
    pub fn with_vmlinuz_path(mut self, dir: PathBuf) -> InstalledKernel {
        self.vmlinuz_path = Some(dir);
        self
    }
    pub fn with_source_path(mut self, dir: PathBuf) -> InstalledKernel {
        self.source_path = Some(dir);
        self
    }
    pub fn with_config_path(mut self, dir: PathBuf) -> InstalledKernel {
        self.config_path = Some(dir);
        self
    }
    pub fn with_system_map_path(mut self, dir: PathBuf) -> InstalledKernel {
        self.system_map_path = Some(dir);
        self
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
impl fmt::Display for InstalledKernel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Version: {}, \n
               Binary path: {:?}, Config path: {:?}, System map path: {:?},\n
               Source dir: {:?}, Module dir: {:?}",
            self.version,
            self.vmlinuz_path,
            self.config_path,
            self.system_map_path,
            self.source_path,
            self.module_path
        )
    }
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

    fn find_all_installed_items(
        &self,
    ) -> io::Result<Vec<(KernelVersion, InstalledItemKind, PathBuf)>> {
        // Search for vmlinuz
        let kernel_images: Vec<_> =
            dir_search::all_paths_with_prefix("vmlinuz-", &self.install_search_path)?
                .into_iter()
                .map(|path| (InstalledItemKind::KernelImage, path))
                .collect();

        // Search for config
        let configs: Vec<_> =
            dir_search::all_paths_with_prefix("config-", &self.install_search_path)?
                .into_iter()
                .map(|path| (InstalledItemKind::Config, path))
                .collect();

        // Search for system map
        let system_maps: Vec<_> =
            dir_search::all_paths_with_prefix("System.map-", &self.install_search_path)?
                .into_iter()
                .map(|path| (InstalledItemKind::SystemMap, path))
                .collect();

        // Search for source dir
        let source_dirs: Vec<_> =
            dir_search::all_paths_with_prefix("linux", &self.source_search_path)?
                .into_iter()
                .map(|path| (InstalledItemKind::SourceDir, path))
                .collect();

        // Search for module path
        let module_dirs: Vec<_> = dir_search::all_paths(&self.module_search_path)?
            .into_iter()
            .map(|path| (InstalledItemKind::ModuleDir, path))
            .collect();

        let all_items: Vec<(KernelVersion, InstalledItemKind, PathBuf)> = vec![
            kernel_images,
            configs,
            system_maps,
            source_dirs,
            module_dirs,
        ]
        .into_iter()
        .flatten()
        .map(|(installed_item, pathbuf)| {
            // Grab the trimmed filename so it can be used to make a KernelVersion
            (
                dir_search::filename_from_path(&pathbuf),
                installed_item,
                pathbuf,
            )
        })
        .map(|(filename, installed_item, pathbuf)| {
            // Turn the filename into a kernel version
            (
                KernelVersion::try_from(filename.unwrap_or_default()),
                installed_item,
                pathbuf,
            )
        })
        .filter_map(|(version, installed_item, pathbuf)| match version {
            // Report any errors and remove those invalid versions
            Ok(v) => Some((v, installed_item, pathbuf)),
            Err(_) => {
                eprintln!("Could not parse {:?} as a KernelVersion", &pathbuf);
                None
            }
        })
        .collect();

        Ok(all_items)
    }

    fn fold_all_to_installed_kernels(
        items: Vec<(KernelVersion, InstalledItemKind, PathBuf)>,
    ) -> Vec<InstalledKernel> {
        let mut version_map: HashMap<KernelVersion, InstalledKernel> = HashMap::new();
        // - Check if that KernelVersion is already present as an InstalledKernel
        //   - If it is, add the path to the InstalledKernel
        //   - otherwise, create a new InstalledKernel with the pair
        for (version, item_kind, path) in items {
            match item_kind {
                InstalledItemKind::KernelImage => {
                    let old_path = version_map
                        .entry(version)
                        .or_insert(InstalledKernel::new(version))
                        .vmlinuz_path
                        .replace(path);
                    if old_path.is_some() {
                        eprintln!(
                            "Overwriting previously present kernel image  {:?} for version {:?}",
                            old_path, version
                        );
                    }
                }
                InstalledItemKind::Config => {
                    let old_path = version_map
                        .entry(version)
                        .or_insert(InstalledKernel::new(version))
                        .config_path
                        .replace(path);
                    if old_path.is_some() {
                        eprintln!(
                            "Overwriting previously present config {:?} for version {:?}",
                            old_path, version
                        );
                    }
                }
                InstalledItemKind::SystemMap => {
                    let old_path = version_map
                        .entry(version)
                        .or_insert(InstalledKernel::new(version))
                        .system_map_path
                        .replace(path);
                    if old_path.is_some() {
                        eprintln!(
                            "Overwriting previously present system map {:?} for version {:?}",
                            old_path, version
                        );
                    }
                }

                InstalledItemKind::SourceDir => {
                    let old_path = version_map
                        .entry(version)
                        .or_insert(InstalledKernel::new(version))
                        .source_path
                        .replace(path);
                    if old_path.is_some() {
                        eprintln!(
                            "Overwriting previously present source path {:?} for version {:?}",
                            old_path, version
                        );
                    }
                }
                InstalledItemKind::ModuleDir => {
                    let old_path = version_map
                        .entry(version)
                        .or_insert(InstalledKernel::new(version))
                        .module_path
                        .replace(path);
                    if old_path.is_some() {
                        eprintln!(
                            "Overwriting previously present module path {:?} for version {:?}",
                            old_path, version
                        );
                    }
                }
            }
        }

        version_map
            .into_iter()
            .map(|(_, installed_kernel)| installed_kernel)
            .collect()
    }

    /// Actually run the search and return all of the found InstalledKernels
    pub fn execute(&self) -> io::Result<Vec<InstalledKernel>> {
        let all_installed_items = self.find_all_installed_items()?;

        let installed_kernels = KernelSearch::fold_all_to_installed_kernels(all_installed_items);

        Ok(installed_kernels)
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
        println!("invalid: {:?}", invalid);
        assert!(invalid.is_err());
    }
    #[test]
    fn kernel_version_from_src() {
        let valid = KernelVersion::try_from("linux-5.11.0-gentoo");
        assert!(valid.is_ok());
    }

    #[test]
    fn kernel_version_from_module() {
        let valid = KernelVersion::try_from("5.11.0-gentoo");
        assert!(valid.is_ok());
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
        let temp_path = PathBuf::from("./temp");
        let version = KernelVersion::new(2, 6, 0, None, false);
        // Only one value is given
        let installed_kernel = InstalledKernel::new(version).with_config_path(temp_path.clone());
        assert_eq!(installed_kernel.files_missing(), true);
    }

    #[test]
    fn files_missing_false() {
        let temp_path = PathBuf::from("./temp");
        let version = KernelVersion::new(2, 6, 0, None, false);
        let installed_kernel = InstalledKernel::new(version)
            .with_config_path(temp_path.clone())
            .with_module_path(temp_path.clone())
            .with_system_map_path(temp_path.clone())
            .with_vmlinuz_path(temp_path.clone())
            .with_source_path(temp_path.clone());
        assert_eq!(installed_kernel.files_missing(), false);
    }
}
