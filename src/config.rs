use serde::Deserialize;
use std::fs;
use std::collections::HashMap;
use toml::map::Map;
use std::path::Path;

// Config file structure
// [general]
// compiler: String
// includes: Array(String)
// flags: Array(String)
// defines: Array(String)
// include_dirs = Array(String)
// exclude_dirs = Array(String)
// filetypes = Array(String)
// gitignore = bool
//
// for each include_dirs there can be one section with the same name
// that defines parameters specific to that directory:
// [dir]
// compiler: String
// includes: Array(String)
// flags: Array(String)
// defines: Array(String)
// include_dirs = Array(String)
// exclude_dirs = Array(String)
// gitignore = bool
//
//
//

#[derive(Deserialize, Debug)]
pub struct ConfigEntry {
    pub compiler: String,
    pub includes: Vec<String>,
    pub flags: Vec<String>,
    pub defines: Vec<String>,
    pub filetypes: Vec<String>,
    pub include_dirs: Vec<String>,
    pub exclude_dirs: Vec<String>,
    pub gitignore: Option<bool>
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub general: ConfigEntry,
    //directories: Vec<ConfigEntry>
    pub directories: HashMap<String, ConfigEntry>
}

const DEFAULT_COMPILER: &str = "/usr/bin/cc";

fn toml_to_config_entry(toml_array: &toml::Value) -> ConfigEntry {
    let cfg = ConfigEntry {
        compiler: toml_array
            .get("compiler")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| DEFAULT_COMPILER)
            .to_string(),
        flags: toml_array
            .get("flags")
            .and_then(|f| f.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .map(|x| String::from(x.as_str().unwrap_or("")))
            .collect(),
        includes: toml_array
            .get("includes")
            .and_then(|f| f.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .map(|x| String::from(x.as_str().unwrap_or("")))
            .collect(),
        defines: toml_array
            .get("defines")
            .and_then(|f| f.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .map(|x| String::from(x.as_str().unwrap_or("")))
            .collect(),
        filetypes: toml_array
            .get("filetypes")
            .and_then(|f| f.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .map(|x| String::from(x.as_str().unwrap_or("")))
            .collect(),
        include_dirs: toml_array
            .get("include_dirs")
            .and_then(|f| f.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .map(|x| String::from(x.as_str().unwrap_or("")))
            .collect(),
        exclude_dirs: toml_array
            .get("exclude_dirs")
            .and_then(|f| f.as_array())
            .unwrap_or(&Vec::new())
            .iter()
            .map(|x| String::from(x.as_str().unwrap_or("")))
            .collect(),
        gitignore: toml_array.get("gitignore").and_then(|f| f.as_bool())
    };
    cfg
}

impl Config {
    pub fn new(config_file: &Path) -> Self {
        let contents = fs::read_to_string(config_file)
            .expect("No config file found!");
        let config = contents.parse::<toml::Value>().ok().and_then(|r| match r {
            toml::Value::Table(table) => Some(table),
            _ => None,
        }).unwrap_or(Map::new());
        let mut cfg = Config {
            general: ConfigEntry { 
                compiler: String::from("/usr/bin/cc"),
                includes: Vec::new(),
                flags: Vec::new(),
                defines: Vec::new(),
                filetypes: Vec::new(),
                include_dirs: Vec::new(),
                exclude_dirs: Vec::new(),
                gitignore: None
            },
            directories: HashMap::new()
        };

        if let Some(general) = config.get("general") {
            cfg.general = toml_to_config_entry(general);

            for d in &cfg.general.include_dirs {
                if let Some(subdir) = config.get(d) {
                    let tmp = toml_to_config_entry(subdir);
                    cfg.directories.insert(d.to_string(), tmp);
                }
            }
        }
        cfg
    }
}


