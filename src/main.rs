use clap::{Parser, Subcommand};
use home;
use serde::{Deserialize, Serialize};
use std::fmt::{self};
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
    /// List all todos
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
    let file_path = directory.join("rem.json");

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
        Commands::Toggle { index } => match rem.toggle_done(index) {
            Ok(_) => {}
            Err(err) => {
                panic!("ERROR::Failed to toggle todo: {err}");
            }
        },
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
            writeln!(f, "{}. {}", i + 1, todo)?;
        }
        Ok(())
    }
}

impl Rem {
    fn print_pending(&self) {
        for (i, todo) in self.todos.iter().enumerate() {
            if !todo.done {
                println!("{}. {}", i + 1, todo);
            }
        }
    }

    fn toggle_done(&mut self, index: usize) -> Result<(), TodoError> {
        if index == 0 || index > self.todos.len() {
            return Err(TodoError::InvalidIndex {
                min: 1,
                max: self.todos.len(),
            });
        }
        let todo = &mut self.todos[index - 1];
        todo.done = !todo.done;
        Ok(())
    }
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {}{}{}",
            if self.done { "" } else { "" },
            self.content,
            match self.deadline.clone() {
                Some(deadline) => format!(" ({})", deadline),
                None => "".to_string(),
            },
            match self.daily {
                Some(true) => " (daily)",
                _ => "",
            }
        )
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

#[derive(Debug)]
enum TodoError {
    InvalidIndex { min: usize, max: usize },
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TodoError::InvalidIndex { min, max } => {
                write!(f, "Invalid index, valid range is {min}-{max}")
            }
        }
    }
}

impl std::error::Error for TodoError {}
