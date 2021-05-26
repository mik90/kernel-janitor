use crate::{error::JanitorError, kernel::InstalledKernel, utils, JanitorErrorFrom};
use std::{collections::BTreeMap, io::BufRead, path::Path, process::Command};

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
    cmd_config: &RunCmdConfig,
    newest_config: &Path,
    newest_source_dir: &Path,
) -> Result<(), JanitorError> {
    // Copy most recent kernel config over
    let to = newest_source_dir.join(".config");
    let cmd_desc = format!("copy from {:?} to {:?}", newest_config, to);
    match &cmd_config.pretend {
        PretendStatus::Pretend => {
            println!("Pretending to {}", &cmd_desc);
        }
        PretendStatus::RunTheDamnThing => {
            utils::maybe_prompt_for_confirmation(cmd_config, &cmd_desc)?;
            println!("Running {}", cmd_desc);
            let res = std::fs::copy(newest_config, to);
            if res.is_err() {
                return Err(res.unwrap_err().into());
            }
        }
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
    cmd_config: &RunCmdConfig,
    num_versions_to_keep: usize,
    installed_kernels: Vec<InstalledKernel>,
) -> Result<(), JanitorError> {
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
        utils::maybe_prompt_for_confirmation(
            cmd_config,
            &format!("Delete {} old kernels?", num_versions_to_delete),
        )?;
        let removal_result: Result<_, _> = installed_kernels
            .into_iter()
            .take(num_versions_to_delete)
            .map(|kernel| kernel.uninstall(&cmd_config.pretend))
            .collect();
        removal_result.map_err(|e| JanitorError::from(e))
    }
}

// Useful for testing interactive action
// https://stackoverflow.com/questions/28370126/how-can-i-test-stdin-and-stdout
fn prompt_for_char<R>(mut reader: R) -> Result<char, JanitorError>
where
    R: BufRead,
{
    let mut s = String::new();
    reader.read_line(&mut s)?;
    s.chars()
        .next()
        .ok_or(JanitorErrorFrom!("Could not parse input: {}", s))
}

// Interactive deletion of kernels
pub fn delete_interactive(
    cmd_config: &RunCmdConfig,
    installed_kernels: Vec<InstalledKernel>,
) -> Result<(), JanitorError> {
    // Zip up letters with kernels
    // If you have more than 26 kernels then you're kind of screwed
    let mut choice_map: BTreeMap<char, InstalledKernel> = ('a'..='z')
        .into_iter()
        .zip(installed_kernels.into_iter().rev())
        .collect();

    println!("Listing installed kernels (oldest to newest)...");
    for (letter, kernel) in choice_map.iter() {
        println!("{}) {}", letter, kernel);
    }
    let stdio = std::io::stdin();
    let input = stdio.lock();
    println!("Select a kernel to delete:");
    let choice = prompt_for_char(input)?;

    choice_map
        .remove(&choice)
        .ok_or(JanitorErrorFrom!("Could not find selected kernel"))?
        .uninstall(&cmd_config.pretend)?;
    Ok(())
}
#[cfg(test)]
mod test {
    use super::*;
    /*
    use crate::{kernel::KernelSearch, utils::tests::*};

    fn two_installed_kernels() {
        cleanup_test_dir();
        init_test_dir();

        let dummy_install = InstalledKernel::create_test_version("5.4.97", false);
        let dummy_install_old = InstalledKernel::create_test_version("5.4.97", true);
        let install_path = get_test_install_pathbuf();
        let module_path = get_test_module_pathbuf();
        let src_path = get_test_src_pathbuf();

        let installed_kernels = KernelSearch::new(&install_path, &src_path, &module_path)
            .execute()
            .unwrap();
    }
    */

    #[test]
    fn check_input_prompt() -> Result<(), JanitorError> {
        let input = b"a";
        let choice = prompt_for_char(&input[..])?;
        assert_eq!(choice, 'a');
        Ok(())
    }
}
