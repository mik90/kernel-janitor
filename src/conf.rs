use std::{error::Error, fmt};

#[derive(Debug)]
struct Entry {
    pub name: String,
    pub value: String,
}
/// .ini style config file but without sections
struct ConfigFile {
    entries: Vec<Entry>,
}

#[derive(PartialEq, Debug)]
/// ParseError(String) contains the line that failed to parse
/// A comment can just be ignored
enum ConfigError {
    ParseError(String),
    Comment,
}

impl ConfigError {
    fn is_parse_error(&self) -> bool {
        match self {
            ConfigError::ParseError(_) => true,
            _ => false,
        }
    }
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
    #[test]
    fn parse_conf_file() {
        let lines = vec![
            "# Hello",
            " #With space",
            "entry_0 =   value_0 # Some Comment",
            "entry_1 = value_1",
            "entry_2 = value_2#Comment",
            "entry_3=value_3",
            " entry_4= value_4",
        ];
        let (entries, errors): (Vec<_>, Vec<_>) = lines
            .into_iter()
            .map(|line| Entry::new(line))
            .partition(|entry| entry.is_ok());

        let errors: Vec<_> = errors
            .into_iter()
            .map(Result::unwrap_err)
            .filter(|e| e.is_parse_error())
            .collect();

        errors.iter().for_each(|e| match e {
            ConfigError::ParseError(e) => {
                eprintln!("{}", e);
            }
            _ => (),
        });

        let entries: Vec<_> = entries.into_iter().map(Result::unwrap).collect();

        assert!(errors.is_empty());
        assert_eq!(entries[0].name, "entry_0");
        assert_eq!(entries[0].value, "value_0");

        assert_eq!(entries[1].name, "entry_1");
        assert_eq!(entries[1].value, "value_1");

        assert_eq!(entries[2].name, "entry_2");
        assert_eq!(entries[2].value, "value_2");

        assert_eq!(entries[3].name, "entry_3");
        assert_eq!(entries[3].value, "value_3");

        assert_eq!(entries[4].name, "entry_4");
        assert_eq!(entries[4].value, "value_4");
    }
}
