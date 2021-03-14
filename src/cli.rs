use std::collections::HashSet;

type FlagName = String;
/// Only boolean flags for now
/// `name` is used by the programmer to refer to the Flag
#[derive(PartialEq, Eq, Hash)]
struct Flag {
    name: FlagName,
    short_form: String,
    long_form: String,
    description: String,
}

/// The parser should return a HashSet that contains the list of found flags
/// The HashSet will be indexed by the `Flag::name` member
struct FlagParser {
    flags: Vec<Flag>,
    found_flags: HashSet<FlagName>,
}

struct ParseResults {
    help_message: String,
    found_flags: HashSet<FlagName>,
}

impl Flag {
    pub fn new(name: &str, short_form: &str, long_form: &str, description: &str) -> Flag {
        Flag {
            name: name.to_owned(),
            short_form: short_form.to_owned(),
            long_form: long_form.to_owned(),
            description: description.to_owned(),
        }
    }

    pub fn matches(&self, other: &str) -> bool {
        if other == self.short_form || other == self.long_form {
            true
        } else {
            false
        }
    }
}

impl FlagParser {
    pub fn new() -> FlagParser {
        FlagParser {
            flags: Vec::new(),
            found_flags: HashSet::new(),
        }
    }

    /// Overwrites flags if they exist with the same name
    pub fn with_flag(
        mut self,
        name: &str,
        short_form: &str,
        long_form: &str,
        description: &str,
    ) -> FlagParser {
        self.flags
            .push(Flag::new(name, short_form, long_form, description));
        self
    }

    fn with_help_flag(self) -> FlagParser {
        self.with_flag(
            "help",
            "-h",
            "--help",
            "Print this message and all of the available flags",
        )
    }

    pub fn help_message(&self) -> String {
        self.flags
            .iter()
            .map(|flag| {
                format!(
                    "{},{:width$}{:}",
                    flag.short_form,
                    flag.long_form,
                    flag.description,
                    width = 25
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns a HashSet of the enabled flag names
    pub fn parse_args(mut self, args: Vec<String>) -> ParseResults {
        // nested `for` loops, yuck
        for arg in args {
            for flag in &self.flags {
                if flag.matches(&arg) {
                    self.found_flags.insert(flag.name.clone());
                }
            }
        }
        ParseResults::from(self.with_help_flag())
    }
}

impl ParseResults {
    pub fn flag_enabled(&self, name: &str) -> bool {
        self.found_flags.contains(name)
    }
    pub fn help_message(&self) -> String {
        self.help_message.clone()
    }
}
impl From<FlagParser> for ParseResults {
    fn from(parser: FlagParser) -> Self {
        ParseResults {
            help_message: parser.help_message(),
            found_flags: parser.found_flags,
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arg_parse() {
        let args = vec![
            "-e".to_string(),
            "--not-found".to_string(),
            "-t".to_string(),
            "--test".to_string(),
            "--other-flag".to_string(),
        ];
        let enabled_flag_0_name = "enabled_flag_0";
        let enabled_flag_1_name = "enabled_flag_1";
        let disabled_flag_0_name = "disabled_flag_0";
        let disabled_flag_1_name = "disabled_flag_1";

        let parse_results = FlagParser::new()
            .with_flag(enabled_flag_0_name, "-t", "--test", "enabled")
            .with_flag(enabled_flag_1_name, "-o", "--other-flag", "also enabled")
            .with_flag(disabled_flag_0_name, "-z", "--zoopies", "also disabled")
            .with_flag(disabled_flag_1_name, "-f", "--flag", "also disabled")
            .parse_args(args);

        println!("{}", parse_results.help_message());
        assert_eq!(parse_results.flag_enabled(enabled_flag_0_name), true);
        assert_eq!(parse_results.flag_enabled(enabled_flag_1_name), true);
        assert_eq!(parse_results.flag_enabled(disabled_flag_0_name), false);
        assert_eq!(parse_results.flag_enabled(disabled_flag_1_name), false);
    }
}
