use std::error::Error;

struct Entry {
    pub name: String,
    pub value: String,
}
struct Section {
    entries: Vec<Entry>,
}
/// .ini style config file
struct ConfigFile {
    sections: Vec<Section>,
}

/// This should probably derive from an Error trait
/// `NAME`: Cannot parse entry name, and subsequently the value
/// `VALUE`: Cannot parse entry value
#[derive(Debug)]
pub enum ConfigErrorKind {
    EntryName,
    EntryValue,
}
#[derive(Debug)]
struct ConfigError {
    pub kind: ConfigErrorKind,
    pub description: String,
}

pub fn find_conf_files() {}

impl ConfigError {
    pub fn new(kind: ConfigErrorKind, desc: &str) -> ConfigError {
        ConfigError {
            kind,
            description: desc.to_string(),
        }
    }
}

impl Entry {
    /// Parse `some_entry_name = some_entry_value` to grab the
    /// name and the value as a string
    pub fn new(line: &str) -> Result<Entry, ConfigError> {
        let equals_idx = line
            .find('=')
            .ok_or(ConfigError::new(ConfigErrorKind::EntryName, line))?;
        let (name, value) = line.split_at(equals_idx);

        let name = name.trim();
        // The line starts with a '=' which needs to be trimmed before the whitespace
        let value = value
            .strip_prefix('=')
            .ok_or(ConfigError::new(ConfigErrorKind::EntryValue, line))?;
        if value.is_empty() {
            return Err(ConfigError::new(ConfigErrorKind::EntryValue, line));
        }

        Ok(Entry {
            name: name.to_string(),
            value: value.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_entry() {
        let line = "entry_name      =      5";
        let entry = Entry::new(line);

        assert!(entry.is_ok());
        /*
        let entry = entry.into_ok();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "5");
        */
    }
}
