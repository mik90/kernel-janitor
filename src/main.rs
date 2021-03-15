mod cli;
mod conf;
mod dir_search;
mod kernel;

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
            "Let the user copy over and edit the kernel configuration before building.
            Otherwise, configuration will be copied over automatically.",
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
    if parsed_results.flag_enabled("list") {
        println!("list enabled");
    }

    Ok(())
}
