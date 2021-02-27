use std::{cmp::Ordering, convert::TryFrom, option::Option, path::PathBuf};
/// Format: <major>.<minor>.<patch>-gentoo
///         or <major>.<minor>.<patch>-rc<release_candidate_num>-gentoo
///         or <major>.<minor>.<patch>-gentoo.old
#[derive(Eq)]
struct KernelVersion {
    major: u32,
    minor: u32,
    patch: u32,
    release_candidate_num: Option<u32>,
    is_old: bool,
}
struct InstalledKernel {
    version: KernelVersion,
    module_path: Option<PathBuf>,
    vmlinuz_path: Option<PathBuf>,
    source_path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    system_map_path: Option<PathBuf>,
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
            self.is_old,
        )
            .cmp(&(
                other.major,
                other.minor,
                other.patch,
                other.release_candidate_num,
                other.is_old,
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
            && self.release_candidate_num.is_some()
            && other.release_candidate_num.is_some()
            && self.release_candidate_num.unwrap() == other.release_candidate_num.unwrap()
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
}
