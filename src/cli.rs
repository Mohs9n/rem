use clap::{arg, command, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
#[command(about = "TODO CLI app", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new todo
    New(NewTodoParams),
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

#[derive(Debug, Clone, Args)]
pub struct NewTodoParams {
    /// The content of the todo
    pub content: String,
    /// The due date of the todo (for scheduled todos), valid format: YYYY-MM-DD
    #[arg(long)]
    pub due: Option<String>,
    /// make the todo a daily todo
    #[arg(short, long)]
    pub daily: bool,
}
