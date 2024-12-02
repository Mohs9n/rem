use clap::{Parser, Subcommand};
use home;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Read;

#[derive(Debug, Deserialize, Serialize)]
struct Rem {
    todos: Vec<Todo>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Todo {
    content: String,
    done: bool,
    deadline: Option<String>,
    daily: Option<bool>,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo
    New {
        /// The content of the todo
        content: String,
        /// The deadline of the todo
        // #[arg(short, long)]
        deadline: Option<String>,
        /// Create a daily todo
        #[arg(short, long, default_value_t = false)]
        daily: bool,
    },
    /// Toggle the done state of a todo by its index
    Toggle {
        /// The index of the todo to toggle (1-based)
        index: usize,
    },
    /// Lists pending todos
    Pending,
    All,
}

fn main() {
    let home = match home::home_dir() {
        Some(path) => path,
        None => panic!("ERROR::Home directory not found"),
    };

    let directory = home.join(".local/share/rem");
    if let Err(err) = fs::create_dir_all(&directory) {
        panic!("ERROR::Failed to create directory: {err}");
    }
    let file_path = directory.join("remr.json");

    println!("INFO::{}", file_path.display());

    let mut rem = load_or_initialize_rem(&file_path);

    let cli = Cli::parse();

    // Handle commands
    match cli.command {
        Commands::New {
            content,
            deadline,
            daily,
        } => {
            rem.todos.push(Todo {
                content,
                done: false,
                deadline,
                daily: Some(daily),
            });
            println!("Todo added!");
        }
        Commands::Toggle { index } => {
            if index == 0 || index > rem.todos.len() {
                println!("Invalid index: {}", index);
            } else {
                let todo = &mut rem.todos[index - 1];
                todo.done = !todo.done;
                println!(
                    "Todo \"{}\" marked as {}",
                    todo.content,
                    if todo.done { "done" } else { "not done" }
                );
            }
        }
        Commands::Pending => {
            rem.print_pending();
        }
        Commands::All => {
            println!("{}", rem)
        }
    }

    save_rem(&file_path, &rem);
}

impl fmt::Display for Rem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, todo) in self.todos.iter().enumerate() {
            writeln!(
                f,
                "{}. {} {}{}",
                i + 1,
                if todo.done { "" } else { "" },
                todo.content,
                match todo.deadline.clone() {
                    Some(deadline) => format!(" ({})", deadline),
                    None => "".to_string(),
                }
            )?;
        }
        Ok(())
    }
}

impl Rem {
    fn print_pending(&self) {
        for (i, todo) in self.todos.iter().enumerate() {
            if !todo.done {
                println!(
                    "{}. {} {}{}",
                    i + 1,
                    if todo.done { "" } else { "" },
                    todo.content,
                    match todo.deadline.clone() {
                        Some(deadline) => format!(" ({})", deadline),
                        None => "".to_string(),
                    }
                );
            }
        }
    }
}

fn load_or_initialize_rem(file_path: &std::path::Path) -> Rem {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)
        .expect("Failed to open or create remr.json");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    if contents.trim().is_empty() {
        Rem { todos: Vec::new() }
    } else {
        serde_json::from_str(&contents).expect("Failed to parse remr.json")
    }
}

fn save_rem(file_path: &std::path::Path, rem: &Rem) {
    let json = serde_json::to_string_pretty(rem).expect("Failed to serialize Rem");
    fs::write(file_path, json).expect("Failed to write to remr.json");
}
