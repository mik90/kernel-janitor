use crate::{error::JanitorError, kernel::InstalledKernel, utils};
use std::{path::Path, process::Command};

#[derive(PartialEq, Eq)]
pub enum PretendStatus {
    Pretend,
    RunTheDamnThing,
}

/// Expects the newest kernel that has already been built
pub fn copy_config(
    pretend: &PretendStatus,
    install_path: &Path,
    newest_built_kernel: &InstalledKernel,
) -> Result<(), JanitorError> {
    let newest_src_path = match &newest_built_kernel.source_path {
        Some(p) => p,
        None => {
            return Err(JanitorError::from(format!(
                "Could not find a source directory for kernel version {}",
                newest_built_kernel.version
            )))
        }
    };

    // Copy most recent kernel config over
    if let Some(installed_config) = &newest_built_kernel.config_path {
        // Install to $newest_src_path/.config
        let to = Path::new(&newest_src_path).join(".config");
        match pretend {
            PretendStatus::Pretend => {
                println!("Pretending to copy from {:?} to {:?}", installed_config, to)
            }
            PretendStatus::RunTheDamnThing => {
                println!("Copying {:?} to {:?}", installed_config, to);
                let res = std::fs::copy(installed_config, to);
                if res.is_err() {
                    return Err(res.unwrap_err().into());
                }
            }
        };
    } else {
        return Err(format!(
            "Could not find a config file in {:?} for kernel version {:?}",
            install_path, newest_built_kernel.version
        )
        .into());
    };

    Ok(())
}

pub fn build_kernel(
    pretend: &PretendStatus,
    src_dir: &Path,
    install_path: &Path,
) -> Result<(), JanitorError> {
    utils::exec_and_print_command(
        Command::new("make")
            .arg("olddefconfig")
            .current_dir(src_dir),
        format!("\'make olddefconfig\' in {:?}", src_dir),
        pretend,
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
        pretend,
    )?;

    // make modules_install
    utils::exec_and_print_command(
        Command::new("make")
            .arg("modules_install")
            .current_dir(src_dir),
        format!("\'make modules_install\' in {:?}", src_dir),
        pretend,
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
        pretend,
    )?;
    Ok(())
}

pub fn rebuild_portage_modules(pretend: &PretendStatus) -> Result<(), JanitorError> {
    // emerge @module-rebuild
    utils::exec_and_print_command(
        Command::new("emerge").arg("@module-rebuild"),
        format!("\'emerge @module-rebuild\'"),
        pretend,
    )?;
    Ok(())
}

pub fn gen_grub_cfg(pretend: &PretendStatus, install_path: &Path) -> Result<(), JanitorError> {
    // grub-mkconfig -o $install_path/grub/grub.cfg
    let grub_cfg_path = install_path.join("grub").join("grub.cfg");
    utils::exec_and_print_command(
        Command::new("grub-mkconfig").arg("-o").arg(&grub_cfg_path),
        format!("\'grub-mkconfig -o {:?}\'", grub_cfg_path),
        pretend,
    )?;
    Ok(())
}

//  cleaning up old kernels and their related installed items
pub fn cleanup_old_installs(
    pretend: &PretendStatus,
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
            .map(|kernel| kernel.uninstall(pretend))
            .collect();
        removal_result
    }
}
