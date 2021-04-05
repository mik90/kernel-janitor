use crate::{error::JanitorError, kernel::InstalledKernel};
use std::{
    io::{BufRead, BufReader},
    path::Path,
    process::Command,
    thread,
};

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

// Runs the command and prints both stdout/stderr to the console
fn exec_and_print_command(
    cmd: &mut Command,
    cmd_desc: String,
    pretend: &PretendStatus,
) -> Result<(), JanitorError> {
    if pretend == &PretendStatus::Pretend {
        println!("Pretending to run {}", cmd_desc);
        return Ok(());
    }
    println!("Running {}", cmd_desc);
    let child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.ok_or_else(|| {
        JanitorError::from(format!(
            "Could not capture standard output for {}",
            cmd_desc
        ))
    })?;
    let stderr = child.stderr.ok_or_else(|| {
        JanitorError::from(format!("Could not capture standard error for {}", cmd_desc))
    })?;

    let out_thread = thread::spawn(move || {
        let stdout_reader = BufReader::new(stdout);
        stdout_reader
            .lines()
            .filter_map(Result::ok)
            .for_each(|l| println!("stdout: {}", l));
    });
    let err_thread = thread::spawn(move || {
        let stderr_reader = BufReader::new(stderr);
        stderr_reader
            .lines()
            .filter_map(Result::ok)
            .for_each(|l| println!("stderr: {}", l));
    });
    out_thread.join().expect("Could not join out_thread");
    err_thread.join().expect("Could not join err_thread");
    Ok(())
}

pub fn build_kernel(
    pretend: &PretendStatus,
    src_dir: &Path,
    install_path: &Path,
) -> Result<(), JanitorError> {
    exec_and_print_command(
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
    exec_and_print_command(
        Command::new("make")
            .arg("-j")
            .arg(nproc)
            .current_dir(src_dir),
        format!("\'make -j{}\' in {:?}", nproc, src_dir),
        pretend,
    )?;

    // make modules_install
    exec_and_print_command(
        Command::new("make")
            .arg("modules_install")
            .current_dir(src_dir),
        format!("\'make modules_install\' in {:?}", src_dir),
        pretend,
    )?;

    // make install (with INSTALL_PATH env)
    exec_and_print_command(
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
    exec_and_print_command(
        Command::new("emerge").arg("@module-rebuild"),
        format!("\'emerge @module-rebuild\'"),
        pretend,
    )?;
    Ok(())
}

/// run grub mkconfig -o $install_path/grub/grub.cfg
pub fn gen_grub_cfg(pretend: &PretendStatus, install_path: &Path) -> Result<(), JanitorError> {
    let grub_cfg_path = install_path.join("grub").join("grub.cfg");
    exec_and_print_command(
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
        let num_version_to_delete = installed_kernels.len() - num_versions_to_keep;
        let removal_result: Result<_, _> = installed_kernels
            .into_iter()
            .take(num_version_to_delete)
            .map(|kernel| kernel.uninstall(pretend))
            .collect();
        removal_result
    }
}
