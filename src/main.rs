use clap::Parser;
use cli::{Cli, Commands};
use std::fs::{self, OpenOptions};
use std::io::Read;
use types::Rem;
mod cli;
mod types;

fn main() {
    let file_path = get_save_file_path();

    let mut rem = load_or_initialize_rem(&file_path);

    rem.update_state();

    let cli = cli::Cli::parse();

    // Handle commands
    cli.handle_cli(&mut rem);

    save_rem(&file_path, &rem);
}

fn get_save_file_path() -> std::path::PathBuf {
    let Some(home) = home::home_dir() else {
        panic!("ERROR::Home directory not found")
    };

    let directory = home.join(".local/share/rem");
    if let Err(err) = fs::create_dir_all(&directory) {
        panic!("ERROR::Failed to create directory: {err}");
    }

    directory.join("rem.json")
}

fn load_or_initialize_rem(file_path: &std::path::Path) -> types::Rem {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(file_path)
        .expect("Failed to open or create {file_path}");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    if contents.trim().is_empty() {
        types::Rem { todos: Vec::new() }
    } else {
        serde_json::from_str(&contents).expect("Failed to parse rem.json")
    }
}

fn save_rem(file_path: &std::path::Path, rem: &types::Rem) {
    let json = serde_json::to_string_pretty(rem).expect("Failed to serialize Rem");
    fs::write(file_path, json).expect("Failed to write to rem.json");
}

impl Cli {
    pub fn handle_cli(&self, rem: &mut Rem) {
        match &self.command {
            Some(Commands::New(params)) => match rem.add_todo(params) {
                Ok(()) => {}
                Err(err) => panic!("ERROR::Failed to add todo: {err}"),
            },
            Some(Commands::Toggle { index }) => match rem.toggle_todo(*index) {
                Ok(()) => {}
                Err(err) => {
                    panic!("ERROR::Failed to toggle todo: {err}");
                }
            },
            Some(Commands::All) => println!("{rem}"),
            Some(Commands::Pending) | None => rem.print_pending(),
        }
    }
}
