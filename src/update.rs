use crate::{error::JanitorError, kernel::InstalledKernel};
use std::{
    io::{self, Write},
    path::Path,
    process::Command,
};

#[derive(PartialEq, Eq)]
pub enum PretendStatus {
    Pretend,
    RunTheDamnThing,
}

pub fn copy_config(
    pretend: &PretendStatus,
    install_path: &Path,
    newest_kernel: &InstalledKernel,
) -> Result<(), JanitorError> {
    let newest_src_path = match &newest_kernel.source_path {
        Some(p) => p,
        None => {
            return Err(JanitorError::from(format!(
                "Could not find a source directory for kernel version {}",
                newest_kernel.version
            )))
        }
    };

    // Copy most recent kernel config over
    if let Some(installed_config) = &newest_kernel.config_path {
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
            install_path, newest_kernel.version
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
    // make oldconfig
    if pretend == &PretendStatus::Pretend {
        println!("Pretending to run \'make oldconfig\' in {:?}", src_dir);
    } else {
        println!("Running \'make oldconfig\' in {:?}", src_dir);
        let output = Command::new("make")
            .arg("oldconfig")
            .current_dir(src_dir)
            .output()?;
        io::stderr().write_all(&output.stderr)?;
        io::stdout().write_all(&output.stdout)?;
        if !output.status.success() {
            return Err("Command failed".into());
        }
    }

    // Number of processors
    let nproc_stdout = Command::new("nproc").output()?.stdout;
    // Remove whitespace and newlines
    let nproc = std::str::from_utf8(&nproc_stdout)?.trim();

    // make -j $(nproc)
    if pretend == &PretendStatus::Pretend {
        println!("Pretending to run \'make -j{}\' in {:?}", nproc, src_dir);
    } else {
        println!("Running \'make -j{}\' in {:?}", nproc, src_dir);
        let output = Command::new("make")
            .arg("-j")
            .arg(nproc)
            .current_dir(src_dir)
            .output()?;
        io::stderr().write_all(&output.stderr)?;
        io::stdout().write_all(&output.stdout)?;
        if !output.status.success() {
            return Err("Command failed".into());
        }
    }

    // make modules_install
    if pretend == &PretendStatus::Pretend {
        println!(
            "Pretending to run \'make modules_install\' in {:?}",
            src_dir
        );
    } else {
        println!("Running \'make modules_install\' in {:?}", src_dir);
        let output = Command::new("make")
            .arg("modules_install")
            .current_dir(src_dir)
            .output()?;
        io::stderr().write_all(&output.stderr)?;
        io::stdout().write_all(&output.stdout)?;
        if !output.status.success() {
            return Err("Command failed".into());
        }
    }

    // make install (with INSTALL_PATH env)
    if pretend == &PretendStatus::Pretend {
        println!(
            "Pretending to run \'make install\' in {:?} with env INSTALL_PATH={:?}",
            src_dir, install_path
        );
    } else {
        println!(
            "Running \'make install\' in {:?} with env INSTALL_PATH={:?}",
            src_dir, install_path
        );
        let output = Command::new("make")
            .arg("install")
            .current_dir(src_dir)
            .env("INSTALL_PATH", install_path)
            .output()?;
        io::stderr().write_all(&output.stderr)?;
        io::stdout().write_all(&output.stdout)?;
        if !output.status.success() {
            return Err("Command failed".into());
        }
    }
    Ok(())
}

pub fn rebuild_portage_modules(pretend: &PretendStatus) -> Result<(), JanitorError> {
    // make install (with INSTALL_PATH env)
    if pretend == &PretendStatus::Pretend {
        println!("Pretending to run \'emerge @module-rebuild\'");
    } else {
        println!("Running \'emerge @module-rebuild\'");
        let output = Command::new("emerge").arg("@module-rebuild").output()?;
        io::stderr().write_all(&output.stderr)?;
        io::stdout().write_all(&output.stdout)?;
        if !output.status.success() {
            return Err("Command failed".into());
        }
    }
    Ok(())
}

/// run grub mkconfig -o $install_path/grub/grub.cfg
pub fn gen_grub_cfg(pretend: &PretendStatus, install_path: &Path) -> Result<(), JanitorError> {
    // make install (with INSTALL_PATH env)
    let grub_cfg_path = install_path.join("grub").join("grub.cfg");
    if pretend == &PretendStatus::Pretend {
        println!("Pretending to run \'grub-mkconfig -o {:?}\'", grub_cfg_path);
    } else {
        println!("Running \'grub-mkconfig -o {:?}\'", grub_cfg_path);
        let output = Command::new("grub-mkconfig")
            .arg("-o")
            .arg(grub_cfg_path)
            .output()?;
        io::stderr().write_all(&output.stderr)?;
        io::stdout().write_all(&output.stdout)?;
        if !output.status.success() {
            return Err("Command failed".into());
        }
    }
    Ok(())
}

//  cleaning up old kernels and their related installed items
pub fn cleanup_old_installs(
    pretend: &PretendStatus,
    num_versions_to_keep: usize,
    installed_kernels: Vec<InstalledKernel>,
) -> std::io::Result<()> {
    if installed_kernels.len() < num_versions_to_keep {
        let err = std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Configured to delete {} versions but there are only {} present, skipping.",
                num_versions_to_keep,
                installed_kernels.len()
            ),
        );
        return Err(err);
    }

    let removal_result: Result<_, _> = installed_kernels
        .into_iter()
        .take(num_versions_to_keep)
        .map(|kernel| kernel.uninstall(pretend))
        .collect();
    removal_result
}
