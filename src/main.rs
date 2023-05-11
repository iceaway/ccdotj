use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Write};
use std::error::Error;
use toml::map::Map;
use std::path::{Path, Component};
use serde::{Deserialize, Serialize};

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
struct ConfigEntry {
    compiler: String,
    includes: Vec<String>,
    flags: Vec<String>,
    defines: Vec<String>,
    filetypes: Vec<String>,
    include_dirs: Vec<String>,
    exclude_dirs: Vec<String>,
    gitignore: Option<bool>
}

#[derive(Deserialize, Debug)]
struct Config {
    general: ConfigEntry,
    //directories: Vec<ConfigEntry>
    directories: HashMap<String, ConfigEntry>
}

#[derive(Serialize, Default)]
struct CompileCommandEntry <'a>{
    arguments: Vec<&'a str>,
    directory: &'a str,
    file: &'a str,
    output: &'a str
}

const DEFAULT_COMPILER: &str = "/usr/bin/cc";

fn toml_to_config_entry(toml_array: &toml::Value) -> ConfigEntry {
    let cfg = ConfigEntry {
        compiler: toml_array
            .get("compiler")
            .unwrap_or(&toml::Value::String(String::from(DEFAULT_COMPILER)))
            .as_str()
            .unwrap_or(&String::from(DEFAULT_COMPILER))
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
    fn new(config_file: &Path) -> Self {
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

fn visit_dirs(root: &Path, dir: &Path, config: &Config, file: &mut File) -> io::Result<()> {
    let realdir = dir.canonicalize()?;
    let realroot = root.canonicalize()?;

    if realdir.is_dir() {
        for entry in fs::read_dir(&realdir)? {
            let path = entry?.path();

            if path.is_dir() {
                if realdir == realroot {
                    if config
                        .general
                        .include_dirs
                        .iter()
                        .any(|x| path.file_name() == Some(OsStr::new(x))) {
                        visit_dirs(root, &path, config, file)?;
                    }
                } else {
                    visit_dirs(root, &path, config, file)?;
                }
            } else {
                let Some(file_name) = path.file_name() else {
                    continue;
                };

                let Some(file_name) = file_name.to_str() else {
                    continue;
                };

                let Some(file_stem) = path.file_stem() else {
                    continue;
                };

                let Some(file_stem) = file_stem.to_str() else {
                    continue;
                };

                for ft in &config.general.filetypes {
                    if file_name.ends_with(ft) {
                        let mut cces: Vec<CompileCommandEntry> = Vec::new();
                        let mut cce = CompileCommandEntry::default();
                        cce.arguments.push(&config.general.compiler);
                        for i in &config.general.includes {
                            cce.arguments.push(&i);
                        }
                        for i in &config.general.defines {
                            cce.arguments.push(&i);
                        }
                        for i in &config.general.flags {
                            cce.arguments.push(&i);
                        }
                        cce.arguments.push("-o");
                        let object = format!("{}.o", file_stem);
                        cce.arguments.push(&object);
                        let filename = format!("{}", file_name);
                        cce.arguments.push(&filename);
                        cce.directory = realroot.to_str().unwrap_or("");
                        cce.file = path.to_str().unwrap_or("");
                        cce.output = path.to_str().unwrap_or("");
                        cces.push(cce);
                        let j = serde_json::to_string(&cces)?;
                        file.write_all(&j.as_bytes())?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::new(Path::new("config.toml"));
    let mut file = File::create("compile_commands.json")?;
    let path = Path::new(".");
    visit_dirs(&path, &path, &config, &mut file).expect("Failed to read dirs");

    Ok(())

}
