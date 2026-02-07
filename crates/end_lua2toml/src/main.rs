use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use end_lua_convert::convert_dir;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "end-lua2toml",
    about = "Convert v1 Lua recipes into v2 TOML files"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Clone, Subcommand)]
enum Command {
    /// Convert v1 input files into items.toml/facilities.toml/recipes.toml
    Convert {
        /// Input directory containing facility_power.toml and recipe/*.lua
        #[arg(short = 'i', long, default_value = "data/input")]
        input_dir: PathBuf,

        /// Output directory for generated v2 TOML files
        #[arg(short = 'o', long, default_value = "data")]
        output_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Convert {
        input_dir: PathBuf::from("data/input"),
        output_dir: PathBuf::from("data"),
    }) {
        Command::Convert {
            input_dir,
            output_dir,
        } => run_convert(input_dir, output_dir),
    }
}

fn run_convert(input_dir: PathBuf, output_dir: PathBuf) -> Result<()> {
    let output = convert_dir(&input_dir)
        .with_context(|| format!("converting v1 input from {}", input_dir.display()))?;

    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("creating output dir {}", output_dir.display()))?;

    let items_path = output_dir.join("items.toml");
    let facilities_path = output_dir.join("facilities.toml");
    let recipes_path = output_dir.join("recipes.toml");

    std::fs::write(&items_path, output.items_toml)
        .with_context(|| format!("writing {}", items_path.display()))?;
    std::fs::write(&facilities_path, output.facilities_toml)
        .with_context(|| format!("writing {}", facilities_path.display()))?;
    std::fs::write(&recipes_path, output.recipes_toml)
        .with_context(|| format!("writing {}", recipes_path.display()))?;

    eprintln!("wrote {}", items_path.display());
    eprintln!("wrote {}", facilities_path.display());
    eprintln!("wrote {}", recipes_path.display());

    Ok(())
}
