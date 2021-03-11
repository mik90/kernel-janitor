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

impl Flag {
    pub fn new(name: &str, short_form: &str, long_form: &str, description: &str) -> Flag {
        Flag {
            name: name.to_owned(),
            short_form: short_form.to_owned(),
            long_form: long_form.to_owned(),
            description: description.to_owned(),
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
    pub fn add_flag<'a>(&'a mut self, flag: Flag) -> &'a mut FlagParser {
        self.flags.push(flag);
        self
    }

    /// HashSet containing whether or not a flag is present
    pub fn parse_args(&self) -> HashSet<FlagName> {
        let args: Vec<String> = std::env::args().collect();
        HashSet::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl FlagParser {
        pub fn mock_parse_args(&self, args: &[&str]) -> HashSet<FlagName> {
            for arg in args {}
            HashSet::new()
        }
    }
    #[test]
    fn arg_parse() {
        let parser = FlagParser::new();
    }
}
