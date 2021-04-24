mod cli;
mod conf;
mod error;
mod kernel;
mod update;
mod utils;
use error::JanitorError;
use update::{InteractiveStatus, PretendStatus};
fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

// Got the idea for `try_main` from https://github.com/benhoyt/countwords/blob/8553c8f600c40a4626e966bc7e7e804097e6e2f4/rust/simple/main.rs
fn try_main() -> Result<(), JanitorError> {
    let parsed_results = cli::FlagParser::new()
        .with_flag(
            "manual_edit",
            "-m",
            "--manual-edit",
            "Newest installed config file will be copied to the newest installed source directory",
        )
        .with_flag(
            "clean_only",
            "-c",
            "--clean-only",
            "Clean up the install, source, and module directories then exit",
        )
        .with_flag(
            "interactive",
            "-i",
            "--interactive",
            "Run the commands interactively",
        )
        .with_flag(
            "list",
            "-l",
            "--list",
            "List installed kernels and then exit",
        )
        .with_flag(
            "pretend",
            "-p",
            "--pretend",
            "Don't actually run the command, just print it out",
        )
        .parse_args_from_env();

    if parsed_results.flag_enabled("help") {
        println!("{}", parsed_results.help_message());
        return Ok(());
    }

    if parsed_results.flag_enabled("clean_only") {
        println!("clean only enabled");
    }

    let pretend = match parsed_results.flag_enabled("pretend") {
        true => PretendStatus::Pretend,
        false => PretendStatus::RunTheDamnThing,
    };

    let interactive = match parsed_results.flag_enabled("interactive") {
        true => InteractiveStatus::On,
        false => InteractiveStatus::Off,
    };

    let config = conf::Config::find_in_fs()?;

    /*
     * TODO move old files to trash instead of deleting them
     * I may need to implement a `mv` that copies content and deletes the old ones.
     * Either that or just use the `mv` command
     */
    let _ = config.get_path("TrashPath")?;
    let num_versions_to_keep = config.get_usize("VersionsToKeep")?;
    let regen_grub_cfg = config.get_bool("RegenerateGrubConfig")?;
    let rebuild_portage_modules = config.get_bool("RebuildPortageModules")?;

    let install_path = config.get_path("InstallPath")?;
    let module_path = config.get_path("KernelModulesPath")?;
    let src_path = config.get_path("KernelSourcePath")?;
    let installed_kernels =
        kernel::KernelSearch::new(&install_path, &src_path, &module_path).execute()?;

    if parsed_results.flag_enabled("list") {
        println!("Listing installed kernels (oldest to newest)...\n");
        for k in installed_kernels {
            println!("{}\n", k);
        }
        return Ok(());
    }
    if pretend == PretendStatus::RunTheDamnThing && !utils::user_is_root()? {
        return Err("User is not root and \'pretend\' isn\'t specified. Try running with \'-p\' or \'--pretend\' Exiting...".into());
    }

    // Grab the newest kernel that already has a config
    // The last element is the newest kernel so search in reverse
    let newest_built_kernel = match installed_kernels.iter().rfind(|k| k.config_path.is_some()) {
        Some(k) => k,
        None => {
            return Err(JanitorError::from(format!(
                "Could not find any kernels with an installed configuration file in {:?}",
                install_path
            )))
        }
    };

    let cmd_config = update::RunCmdConfig {
        pretend,
        interactive,
    };

    if parsed_results.flag_enabled("manual_edit") {
        println!("Expecting a kernel config to be present in the newest kernel source directory");
    } else {
        println!("Auto-copying config enabled");
        update::copy_config(&cmd_config, &install_path, &newest_built_kernel)?;
    }

    // Nested matches can't be the right thing to do
    let newest_source_dir = match installed_kernels.last() {
        Some(newest_kernel) => match &newest_kernel.source_path {
            Some(s) => s,
            None => {
                return Err(JanitorError::from(format!(
                    "Kernel {} doesn't have a source directory in {:?}",
                    newest_kernel.version, &install_path
                )));
            }
        },
        None => {
            return Err(JanitorError::from(format!(
                "No installed kernels were found in {:?}",
                &install_path
            )));
        }
    };

    update::build_kernel(&cmd_config, &newest_source_dir, &install_path)?;

    if rebuild_portage_modules {
        update::rebuild_portage_modules(&cmd_config)?;
    }

    if regen_grub_cfg {
        update::gen_grub_cfg(&cmd_config, &install_path)?;
    }

    update::cleanup_old_installs(&cmd_config, num_versions_to_keep, installed_kernels)?;

    Ok(())
}
