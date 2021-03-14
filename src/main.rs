mod cli;
mod dir_search;
mod kernel;
fn main() {
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

    if parsed_results.flag_enabled("manual_edit") {
        println!("manual edit enabled");
    }
    if parsed_results.flag_enabled("clean_only") {
        println!("clean only enabled");
    }
    if parsed_results.flag_enabled("list") {
        println!("list enabled");
    }

    if parsed_results.flag_enabled("help") {
        println!("{}", parsed_results.help_message());
    }
}
