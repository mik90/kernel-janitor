use std::{convert::TryFrom, option::Option, path::PathBuf};
/// Format: <major>.<minor>.<patch>-gentoo
///         or <major>.<minor>.<patch>-rc<release_candidate_num>-gentoo
///         or <major>.<minor>.<patch>-gentoo.old
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
    ) -> KernelVersion {
        KernelVersion {
            major,
            minor,
            patch,
            release_candidate_num,
        }
    }
}
impl TryFrom<String> for KernelVersion {
    // TOOD Get error information in returned error type
    // Maybe use std::num::ParseIntError?
    type Error = ();

    fn try_from(raw_value: String) -> Result<Self, Self::Error> {
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
            return Err(());
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
            patch: version_triple[2],,
            release_candidate_num: release_candidate_num,
            is_old: is_old,
        })
    }
}
