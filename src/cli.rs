use clap::{arg, command, Parser, Subcommand};

use crate::types::Rem;

#[derive(Parser)]
#[command(version)]
#[command(about = "TODO CLI app", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new todo
    New {
        /// The content of the todo
        content: String,
        /// The due date of the todo (for scheduled todos), valid format: YYYY-MM-DD
        #[arg(long)]
        due: Option<String>,
        /// make the todo a daily todo
        #[arg(short, long)]
        daily: bool,
    },
    /// Toggle the done state of a todo by its index
    Toggle {
        /// The index of the todo to toggle (1-based)
        index: usize,
    },
    /// Lists pending todos (default)
    Pending,
    /// List all todos
    All,
}

impl Cli {
    pub fn handle_cli(&self, rem: &mut Rem) {
        match &self.command {
            Some(Commands::New {
                content,
                due,
                daily,
            }) => match rem.add_todo(content.clone(), due.clone(), *daily) {
                Ok(()) => {}
                Err(err) => panic!("ERROR::Failed to add todo: {err}"),
            },
            Some(Commands::Toggle { index }) => match rem.toggle_done(*index) {
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
