use std::ffi::OsStr;
use std::path::Path;
use std::fs::{self, File};
use serde::Serialize;
use std::io::{self, Write};
use crate::config::Config;

#[derive(Serialize, Default)]
struct CompileCommandEntry <'a>{
    arguments: Vec<&'a str>,
    directory: &'a str,
    file: &'a str,
    output: &'a str
}

pub fn visit_dirs(root: &Path, dir: &Path, config: &Config, file: &mut File) -> io::Result<()> {
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
