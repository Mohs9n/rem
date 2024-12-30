use clap::CommandFactory;
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, Zsh},
};
use std::{fs, path::Path};

include!("src/cli.rs"); // Import the CLI definition

fn main() -> std::io::Result<()> {
    // let outdir = env::var("OUT_DIR").unwrap();
    let outdir = ".";

    // Use the derived Command from the CLI struct
    let mut cmd = Cli::command();

    // Create output directory for completions
    let completions_dir = Path::new(&outdir).join("completions");
    fs::create_dir_all(&completions_dir)?;

    // Generate scripts for each shell
    generate_to(Bash, &mut cmd, "rem", &completions_dir)?;
    generate_to(Zsh, &mut cmd, "rem", &completions_dir)?;
    generate_to(Fish, &mut cmd, "rem", &completions_dir)?;

    println!("cargo:rerun-if-changed=src/cli.rs");
    Ok(())
}
