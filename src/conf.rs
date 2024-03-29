use crate::{error::JanitorError, JanitorErrorFrom};
use std::{
    collections::HashMap,
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
    pub fn new(path: &Path) -> Result<Config, JanitorError> {
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
                    return Err(JanitorError::from(e));
                }
                ConfigLineKind::Comment => (),
            }
        }
        Ok(Config { entries })
    }

    /// Eh, this isn't needed. Kernel installation configuratino isn't per-user
    fn _get_xdg_config_dir() -> Option<PathBuf> {
        let home_dir = std::env::var_os("HOME")
            .ok_or("No \'HOME\' environment variable was found")
            .ok()?;
        let home_dir = home_dir
            .to_str()
            .ok_or(JanitorErrorFrom!(
                "Could not convert {:?} to a str",
                home_dir
            ))
            .ok()?;
        Some(Path::new(home_dir).join(".config"))
    }

    /// Find a Config in the filesystem
    pub fn find_in_fs() -> Result<Config, JanitorError> {
        let path_list: Vec<PathBuf> = vec![
            PathBuf::from("./kernel-janitor.conf"),
            PathBuf::from("/etc/kernel-janitor.conf"),
        ];

        for conf_path in &path_list {
            if conf_path.exists() {
                println!("Found config at {:?}", &conf_path);
                return Config::new(&conf_path);
            }
        }

        // Convert all the PathBufs to a Vec of &str
        let path_strings = path_list
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<String>>()
            .join(", ");
        Err(JanitorError::from(format!(
            "No config files found at {}",
            path_strings
        )))
    }

    // TODO use generics but they must be restricted
    pub fn get_usize(&self, name: &str) -> Result<usize, JanitorError> {
        match self.entries.get(name) {
            Some(e) => e.value.parse::<usize>().map_err(|e| JanitorError::from(e)),
            None => Err(format!("Config value with name {} was not found!", name).into()),
        }
    }
    pub fn get_bool(&self, name: &str) -> Result<bool, JanitorError> {
        match self.entries.get(name) {
            Some(e) => e.value.parse::<bool>().map_err(|e| JanitorError::from(e)),
            None => Err(format!("Config value with name {} was not found!", name).into()),
        }
    }
    pub fn get_path(&self, name: &str) -> Result<PathBuf, JanitorError> {
        match self.entries.get(name) {
            Some(e) => Ok(PathBuf::from(e.value.clone())),
            None => Err(JanitorError::from(format!(
                "Config value with name {} was not found!",
                name
            ))),
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
        let conf = Config::new(&example_conf);
        assert!(conf.is_ok());
        let conf = conf.unwrap();

        let path_value = conf.get_path("InstallPath");
        println!("{:?}", path_value);
        assert!(path_value.is_ok());
        assert_eq!(path_value.unwrap(), PathBuf::from("/boot"));

        let usize_value = conf.get_usize("VersionsToKeep");
        println!("{:?}", usize_value);
        assert!(usize_value.is_ok());
        assert_eq!(usize_value.unwrap(), 3 as usize);

        let bool_value = conf.get_bool("RegenerateGrubConfig");
        println!("{:?}", bool_value);
        assert!(bool_value.is_ok());
        assert_eq!(bool_value.unwrap(), false);
    }

    #[test]
    fn invalid_parse() {
        let example_conf = PathBuf::from("kernel-janitor-example.conf");
        let conf = Config::new(&example_conf);
        assert!(conf.is_ok());
        let conf = conf.unwrap();

        let versions_to_keep = conf.get_bool("VersionsToKeep");
        assert!(versions_to_keep.is_err());
    }
    #[test]
    fn get_home_var() {
        let home_dir = std::env::var_os("HOME");
        println!("Home dir: {:?}", home_dir);
        assert!(home_dir.is_some());
    }
    #[test]
    fn open_symlink() {
        let home_dir = std::env::var_os("HOME");
        println!("Home dir: {:?}", home_dir);
        assert!(home_dir.is_some());
    }
}
