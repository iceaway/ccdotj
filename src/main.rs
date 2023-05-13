use std::env;
use std::fs::File;
use std::error::Error;
use std::path::Path;
use clap::Parser;

mod config;
mod visit;

const DEFAULT_CONFIG_FILE: &str = "config.toml";
const DEFAULT_OUTPUT_FILE: &str = "compile_commands.json";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to config file (default: config.toml)
    #[arg(short, long)]
    config: Option<String>,

    /// Path to root directory (default: current directory)
    #[arg(short, long)]
    root: Option<String>,

    /// Path to output file (default: compile_commands.json)
    #[arg(short, long)]
    output: Option<String>
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let cfgfile = Path::new(args.config
        .as_deref()
        .unwrap_or(DEFAULT_CONFIG_FILE));
     
    let root = Path::new(args.root
        .as_deref()
        .unwrap_or("."));

    let outfile = Path::new(args.output
        .as_deref()
        .unwrap_or(DEFAULT_OUTPUT_FILE));
    let config = config::Config::new(cfgfile);
    let mut file = File::create(outfile)?;
    println!("Indexing files under '{}'", root.to_str().expect("Invalid path"));
    visit::visit_dirs(&root, &root, &config, &mut file).expect("Failed to read dirs");

    Ok(())

}
