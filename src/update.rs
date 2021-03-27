use crate::kernel::InstalledKernel;
use std::path::Path;

pub enum PretendStatus {
    Pretend,
    RunTheDamnThing,
}

pub fn copy_config(
    pretend: PretendStatus,
    install_path: &Path,
    installed_kernels: &[InstalledKernel],
) -> Result<(), Box<dyn std::error::Error>> {
    let newest_kernel = match installed_kernels.last() {
        Some(i) => i,
        None => return Err("No installed kernels found".into()),
    };

    let newest_src_path = match &newest_kernel.source_path {
        Some(p) => p,
        None => {
            return Err(format!(
                "Could not find a source directory for kernel version {:?}",
                newest_kernel.version
            )
            .into())
        }
    };

    // Copy most recent kernel config over
    if let Some(installed_config) = &newest_kernel.config_path {
        // Install to $newest_src_path/.config
        let to = Path::new(&newest_src_path).join(".config");
        if pretend {
            println!("Pretending to copy from {:?} to {:?}", installed_config, to);
        } else {
            std::fs::copy(installed_config, to)?;
        }
    } else {
        return Err(format!(
            "Could not find a config file in {:?} for kernel version {:?}",
            install_path, newest_kernel.version
        )
        .into());
    };

    Ok(())
}

pub fn build_kernel() -> Result<(), Box<dyn std::error::Error>> {
    // TODO make oldconfig

    // TODO make -j $(nproc)

    // TODO make modules_install

    // TODO make install $install_path
    Ok(())
}

// TODO func for run emerge @module-rebuild

// TODO func for run grub mkconfig -o $install_path/grub/grub.cfg

// TODO func for cleaning up old kernels
