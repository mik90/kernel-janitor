use crate::{error::JanitorError, kernel::InstalledKernel, utils, JanitorResultErr};
use std::{path::Path, process::Command};

#[derive(PartialEq, Eq)]
pub enum PretendStatus {
    Pretend,
    RunTheDamnThing,
}
#[derive(PartialEq, Eq)]
pub enum InteractiveStatus {
    On,
    Off,
}
pub struct RunCmdConfig {
    pub pretend: PretendStatus,
    pub interactive: InteractiveStatus,
}

/// Expects the newest kernel that has already been built
pub fn copy_config(
    config: &RunCmdConfig,
    install_path: &Path,
    newest_built_kernel: &InstalledKernel,
) -> Result<(), JanitorError> {
    let newest_src_path = match &newest_built_kernel.source_path {
        Some(p) => p,
        None => {
            return JanitorResultErr!(
                "Could not find a source directory for kernel version {}",
                newest_built_kernel.version
            )
        }
    };

    // Copy most recent kernel config over
    if let Some(installed_config) = &newest_built_kernel.config_path {
        // Install to $newest_src_path/.config
        let to = Path::new(&newest_src_path).join(".config");
        let cmd_desc = format!("copy from {:?} to {:?}", installed_config, to);
        match &config.pretend {
            PretendStatus::Pretend => {
                println!("Pretending to {}", &cmd_desc);
            }
            PretendStatus::RunTheDamnThing => {
                utils::maybe_prompt_for_confirmation(config, &cmd_desc)?;
                println!("Running {}", cmd_desc);
                let res = std::fs::copy(installed_config, to);
                if res.is_err() {
                    return Err(res.unwrap_err().into());
                }
            }
        };
    } else {
        return JanitorResultErr!(
            "Could not find a config file in {:?} for kernel version {:?}",
            install_path,
            newest_built_kernel.version
        );
    };

    Ok(())
}

pub fn build_kernel(
    config: &RunCmdConfig,
    src_dir: &Path,
    install_path: &Path,
) -> Result<(), JanitorError> {
    utils::exec_and_print_command(
        Command::new("make")
            .arg("olddefconfig")
            .current_dir(src_dir),
        format!("\'make olddefconfig\' in {:?}", src_dir),
        &config,
    )?;

    // Number of processors
    let nproc_stdout = Command::new("nproc").output()?.stdout;
    // Remove whitespace and newlines
    let nproc = std::str::from_utf8(&nproc_stdout)?.trim();

    // make -j $(nproc)
    utils::exec_and_print_command(
        Command::new("make")
            .arg("-j")
            .arg(nproc)
            .current_dir(src_dir),
        format!("\'make -j{}\' in {:?}", nproc, src_dir),
        &config,
    )?;

    // make modules_install
    utils::exec_and_print_command(
        Command::new("make")
            .arg("modules_install")
            .current_dir(src_dir),
        format!("\'make modules_install\' in {:?}", src_dir),
        &config,
    )?;

    // make install (with INSTALL_PATH env)
    utils::exec_and_print_command(
        Command::new("make")
            .arg("install")
            .current_dir(src_dir)
            .env("INSTALL_PATH", install_path),
        format!(
            "\'make install\' in {:?} with env INSTALL_PATH={:?}",
            src_dir, install_path
        ),
        &config,
    )?;
    Ok(())
}

pub fn rebuild_portage_modules(config: &RunCmdConfig) -> Result<(), JanitorError> {
    // emerge @module-rebuild
    utils::exec_and_print_command(
        Command::new("emerge").arg("@module-rebuild"),
        format!("\'emerge @module-rebuild\'"),
        &config,
    )?;
    Ok(())
}

pub fn gen_grub_cfg(config: &RunCmdConfig, install_path: &Path) -> Result<(), JanitorError> {
    // grub-mkconfig -o $install_path/grub/grub.cfg
    let grub_cfg_path = install_path.join("grub").join("grub.cfg");
    utils::exec_and_print_command(
        Command::new("grub-mkconfig").arg("-o").arg(&grub_cfg_path),
        format!("\'grub-mkconfig -o {:?}\'", grub_cfg_path),
        &config,
    )?;
    Ok(())
}

//  cleaning up old kernels and their related installed items
pub fn cleanup_old_installs(
    config: &RunCmdConfig,
    num_versions_to_keep: usize,
    installed_kernels: Vec<InstalledKernel>,
) -> std::io::Result<()> {
    if installed_kernels.len() <= num_versions_to_keep {
        Ok(println!(
            "Configured to delete {} versions but there are only {} present. Skipping cleanup.",
            num_versions_to_keep,
            installed_kernels.len()
        ))
    } else {
        // There's more installed kernels than there are to keep
        // The 'pretend' handling is dealt with in `kernel.uninstall`
        let num_versions_to_delete = installed_kernels.len() - num_versions_to_keep;
        let removal_result: Result<_, _> = installed_kernels
            .into_iter()
            .take(num_versions_to_delete)
            .map(|kernel| kernel.uninstall(&config.pretend))
            .collect();
        removal_result
    }
}

pub fn interactive_delete(installed_kernels: Vec<InstalledKernel>) -> Result<(), JanitorError> {
    // List all kernels and assign them letters, then allow deletion by selection
    //let letters_and_kernels = installed_kernels.zip(["a"..].into_iter());
    let size = installed_kernels.len();
    let letters: Vec<_> = ["a"..].into_iter().take(size).collect();
    let letters_and_kernels = installed_kernels.iter().zip(letters);
    for letter_and_kernel in letters_and_kernels {
        println!("{}). {}", letter_and_kernel.0, letter_and_kernel.1);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{kernel::KernelSearch, utils::tests::*};

    fn two_installed_kernels() {

        cleanup_test_dir();
        init_test_dir();

        let dummy_install = InstalledKernel::create_test_version("5.4.97", false);
        let dummy_install_old = InstalledKernel::create_test_version("5.4.97", true);
        let install_path = get_test_install_pathbuf();
        let module_path = get_test_module_pathbuf();
        let src_path = get_test_src_pathbuf();

        let installed_kernels = KernelSearch::new(&install_path, &src_path, &module_path).execute().unwrap()
    }

    #[test]
    fn uninstall_kernels() {
        let kernels = two_installed_kernels();
    }
}
