mod cli;
mod conf;
mod dir_search;
mod kernel;
mod utils;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

// Got the idea for `try_main` from https://github.com/benhoyt/countwords/blob/8553c8f600c40a4626e966bc7e7e804097e6e2f4/rust/simple/main.rs
fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let parsed_results = cli::FlagParser::new()
        .with_flag(
            "manual_edit",
            "-m",
            "--manual-edit",
            "Let the user manually copy over and edit the kernel configuration before building.",
        )
        .with_flag(
            "clean_only",
            "-c",
            "--clean-only",
            "Clean up the install, source, and module directories then exit",
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

    if parsed_results.flag_enabled("manual_edit") {
        println!("manual edit enabled");
    }
    if parsed_results.flag_enabled("clean_only") {
        println!("clean only enabled");
    }

    let config = conf::Config::find_in_fs()?;

    let trash_path = config.get_path("TrashPath")?;
    let versions_to_keep = config.get_u32("VersionsToKeep")?;
    let regen_grub_cfg = config.get_bool("RegenerateGrubConfig")?;
    let emerge_pres_rebuild = config.get_bool("EmergePreservedRebuild")?;

    let install_path = config.get_path("InstallPath")?;
    let module_path = config.get_path("KernelModulesPath")?;
    let src_path = config.get_path("KernelSourcePath")?;
    let installed_kernels =
        kernel::KernelSearch::new(install_path, src_path, module_path).execute()?;

    if parsed_results.flag_enabled("list") {
        println!("Listing installed kernels...");
        for k in installed_kernels {
            println!("  {}", k);
        }
    }

    Ok(())
}
