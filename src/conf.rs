use std::{error::Error, fmt};

struct Entry {
    pub name: String,
    pub value: String,
}
/// .ini style config file but without sections
struct ConfigFile {
    entries: Vec<Entry>,
}

#[derive(Debug)]
/// ParseError(String) contains the line that failed to parse
/// A comment can just be ignored
enum ConfigError {
    ParseError(String),
    Comment,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::ParseError(e) => {
                write!(f, "Could not parse config line \"{}\"", e)
            }
            ConfigError::Comment => Ok(()),
        }
    }
}

fn strip_comment(text: &str) -> &str {
    if text.trim_start().starts_with('#') {
        return "";
    }
    match text.find('#') {
        Some(idx) => {
            // Ignore the comment
            let (stripped, _) = text.split_at(idx);
            stripped
        }
        None => text,
    }
}

impl Entry {
    /// Parse `some_entry_name = some_entry_value` to grab the
    /// name and the value as a string
    pub fn new(line: &str) -> Result<Entry, ConfigError> {
        if strip_comment(line).is_empty() {
            // Early return if the entire line is a comment
            return Err(ConfigError::Comment);
        }
        let equals_idx = line
            .find('=')
            .ok_or(ConfigError::ParseError(line.to_string()))?;
        let (name, value) = line.split_at(equals_idx);

        let name = name.trim();
        // The line starts with a '=' which needs to be trimmed before the whitespace
        let value = value
            .strip_prefix('=')
            .ok_or(ConfigError::ParseError(line.to_string()))?;
        if value.is_empty() {
            return Err(ConfigError::ParseError(line.to_string()));
        }
        let value = strip_comment(value).trim();
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
        let entry = entry.unwrap();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "5");
    }

    #[test]
    fn ignore_comment() {
        let line = "#Some Comment";
        let entry = Entry::new(line);

        assert!(entry.is_err());
    }

    #[test]
    fn ignore_comment_after_value_0() {
        let line = "entry_name      =   value  # Some Comment";
        let entry = Entry::new(line);

        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "value");
    }
    #[test]
    fn ignore_comment_after_value_1() {
        let line = "entry_name =   value2# Some Comment";
        let entry = Entry::new(line);

        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "value2");
    }
    #[test]
    fn ignore_comment_after_value_2() {
        let line = "entry_name=   value2# Some Comment";
        let entry = Entry::new(line);

        assert!(entry.is_ok());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "value2");
    }
}
