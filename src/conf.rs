use std::{
    collections::{hash_map::Entry, HashMap},
    path::{Path, PathBuf},
};

#[derive(PartialEq, Debug)]
pub struct ConfigEntry {
    pub name: String,
    pub value: String,
}
// A single line in a config file can be multiple things
#[derive(PartialEq, Debug)]
pub enum ConfigLineKind {
    Entry(ConfigEntry),
    Section(String),
    Comment,
    ParseError(String),
}

type EntryName = String;
pub struct Config {
    entries: HashMap<EntryName, ConfigEntry>,
}

pub fn find_conf_files() -> Vec<Config> {
    let paths = vec![
        PathBuf::from("./kernel-janitor.conf"),
        PathBuf::from("~/.config/kernel-janitor.conf"),
    ];
    Vec::new()
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

impl ConfigLineKind {
    pub fn parse(line: &str) -> ConfigLineKind {
        if strip_comment(line).is_empty() {
            return ConfigLineKind::Comment;
        }
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            return ConfigLineKind::Section(
                line.trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string(),
            );
        }

        // If not a Comment or Section title, try to parse as an entry
        match ConfigEntry::new(line) {
            Some(e) => ConfigLineKind::Entry(e),
            None => ConfigLineKind::ParseError(line.to_string()),
        }
    }
}

impl ConfigEntry {
    /// Parse `some_entry_name = some_entry_value` to grab the
    /// name and the value as a string
    pub fn new(line: &str) -> Option<ConfigEntry> {
        let equals_idx = line.find('=')?;
        let (name, value) = line.split_at(equals_idx);
        let name = name.trim();

        // The line starts with a '=' which needs to be trimmed before the whitespace
        // It'll definitely be there since it was found, otherwise just default to empty
        let value = value.strip_prefix('=')?;

        if value.is_empty() {
            // Coulnd't find a value assignedto the name :(
            return None;
        }
        let value = strip_comment(value).trim();
        Some(ConfigEntry {
            name: name.to_string(),
            value: value.to_string(),
        })
    }
}

impl Config {
    pub fn read(path: &Path) -> Result<Config, std::io::Error> {
        let contents = std::fs::read(path)?;

        let file_str = String::from_utf8_lossy(&contents);
        let lines = file_str.lines();

        let mut entries = HashMap::<EntryName, ConfigEntry>::new();
        for line in lines {
            match ConfigLineKind::parse(line) {
                ConfigLineKind::Section(_) => (), // Ignore sections for now
                ConfigLineKind::Entry(e) => {
                    entries.insert(e.name.clone(), e);
                    ()
                }
                ConfigLineKind::ParseError(e) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                }
                ConfigLineKind::Comment => (),
            }
        }
        Ok(Config { entries })
    }

    // TODO use generics but they must be restricted
    pub fn get_u32(&self, name: &str) -> Option<u32> {
        match self.entries.get(name) {
            Some(e) => e.value.parse::<u32>().ok(),
            None => None,
        }
    }
    pub fn get_bool(&self, name: &str) -> Option<bool> {
        match self.entries.get(name) {
            Some(e) => e.value.parse::<bool>().ok(),
            None => None,
        }
    }
    pub fn get_path(&self, name: &str) -> Option<PathBuf> {
        match self.entries.get(name) {
            Some(e) => Some(PathBuf::from(e.value.clone())),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_entry() {
        let line = "entry_name      =      5";
        let entry = ConfigEntry::new(line);

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "5");
    }

    #[test]
    fn ignore_comment() {
        let line = "#Some Comment";
        let entry = ConfigEntry::new(line);

        assert!(entry.is_none());
    }

    #[test]
    fn ignore_comment_after_value_0() {
        let line = "entry_name      =   value  # Some Comment";
        let entry = ConfigEntry::new(line);

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "value");
    }
    #[test]
    fn ignore_comment_after_value_1() {
        let line = "entry_name =   value2# Some Comment";
        let entry = ConfigEntry::new(line);

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "value2");
    }
    #[test]
    fn ignore_comment_after_value_2() {
        let line = "entry_name=   value2# Some Comment";
        let entry = ConfigEntry::new(line);

        assert!(entry.is_some());
        let entry = entry.unwrap();
        assert_eq!(entry.name, "entry_name");
        assert_eq!(entry.value, "value2");
    }

    #[test]
    fn parse_conf_file() {
        let example_conf = PathBuf::from("kernel-janitor-example.conf");
        let conf = Config::read(&example_conf);
        assert!(conf.is_ok());
        let conf = conf.unwrap();

        let path_value = conf.get_path("InstallPath");
        println!("{:?}", path_value);
        assert!(path_value.is_some());
        assert_eq!(path_value.unwrap(), PathBuf::from("/boot/EFI/Gentoo"));

        let u32_value = conf.get_u32("VersionsToKeep");
        println!("{:?}", u32_value);
        assert!(u32_value.is_some());
        assert_eq!(u32_value.unwrap(), 3 as u32);

        let bool_value = conf.get_bool("RegenerateGrubConfig");
        println!("{:?}", bool_value);
        assert!(bool_value.is_some());
        assert_eq!(bool_value.unwrap(), false);
    }
}
