use std::fs::File;
use std::error::Error;
use std::path::Path;

mod config;
mod visit;


fn main() -> Result<(), Box<dyn Error>> {
    let config = config::Config::new(Path::new("config.toml"));
    let mut file = File::create("compile_commands.json")?;
    let path = Path::new(".");
    visit::visit_dirs(&path, &path, &config, &mut file).expect("Failed to read dirs");

    Ok(())

}
