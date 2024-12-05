use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fmt::{self};
use std::fs::{self, OpenOptions};
use std::io::Read;

#[derive(Debug, Deserialize, Serialize)]
struct Rem {
    todos: Vec<Todo>,
}

#[derive(Debug, Deserialize, Serialize)]
enum Todo {
    Regular {
        content: String,
        done: bool,
    },
    Daily {
        content: String,
        streak: u32,
        last_marked_done: Option<String>,
        last_marked_done_backup: Option<String>,
        // deadline: Option<chrono::naive::NaiveDate>,
        #[serde(default)]
        longest_streak: u32,
    },
    Scheduled {
        content: String,
        due: String,
        done: bool,
    },
}

#[derive(Parser)]
#[command(version)]
#[command(about = "TODO CLI app", long_about = None)]
struct Cli {
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

fn main() {
    let Some(home) = home::home_dir() else {
        panic!("ERROR::Home directory not found")
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
        Some(Commands::New {
            content,
            due,
            daily,
        }) => match rem.add_todo(content, due, daily) {
            Ok(()) => {}
            Err(err) => panic!("ERROR::Failed to add todo: {err}"),
        },
        Some(Commands::Toggle { index }) => match rem.toggle_done(index) {
            Ok(()) => {}
            Err(err) => {
                panic!("ERROR::Failed to toggle todo: {err}");
            }
        },
        Some(Commands::All) => println!("{rem}"),
        Some(Commands::Pending) | None => rem.print_pending(),
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
            match todo {
                Todo::Regular { done, .. } | Todo::Scheduled { done, .. } if !done => {
                    println!("{}. {}", i + 1, todo);
                }
                Todo::Daily {
                    content,
                    streak,
                    last_marked_done,
                    longest_streak: _,
                    last_marked_done_backup: _,
                } => {
                    let is_pending = match last_marked_done {
                        Some(last_done_date) => {
                            if let Ok(last_date) =
                                chrono::NaiveDate::parse_from_str(last_done_date, "%Y-%m-%d")
                            {
                                // Check if the last done date is before today
                                last_date != chrono::Local::now().date_naive()
                            } else {
                                true // If parsing fails, consider it pending
                            }
                        }
                        None => true, // If never done, it's pending
                    };

                    if is_pending {
                        println!("{}.  {} (daily, streak: {})", i + 1, content, streak);
                    }
                }
                _ => {}
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

        match todo {
            Todo::Regular { done, .. } | Todo::Scheduled { done, .. } => {
                *done = !*done;
            }
            Todo::Daily {
                streak,
                last_marked_done,
                last_marked_done_backup,
                longest_streak,
                ..
            } => {
                use std::cmp::Ordering;
                let today = chrono::Local::now().date_naive();

                let last_done = last_marked_done
                    .as_mut()
                    .map(|date| date.parse::<chrono::NaiveDate>().unwrap());

                if let Some(last_done) = last_done {
                    match last_done.cmp(&today) {
                        Ordering::Less => {
                            if (today - last_done) == chrono::Duration::days(1) {
                                *streak += 1;
                            } else {
                                *streak = 1;
                            }
                            // *last_marked_done_backup = (*last_marked_done).clone();
                            last_marked_done_backup.clone_from(last_marked_done);
                            *last_marked_done =
                                Some(chrono::Local::now().format("%Y-%m-%d").to_string());
                        }
                        Ordering::Equal => {
                            *streak -= 1;
                            *longest_streak -= 1;
                            // *last_marked_done = (*last_marked_done_backup).clone();
                            last_marked_done.clone_from(last_marked_done_backup);
                        }
                        // should be impossible without editing the file
                        Ordering::Greater => {
                            *streak = 1;
                            *last_marked_done =
                                Some(chrono::Local::now().format("%Y-%m-%d").to_string());
                        }
                    }
                } else {
                    *streak += 1;
                    *last_marked_done = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
                }

                if *streak > *longest_streak {
                    *longest_streak = *streak;
                }
            }
        }
        Ok(())
    }

    fn add_todo(
        &mut self,
        content: String,
        due: Option<String>,
        daily: bool,
    ) -> Result<(), TodoError> {
        let todo = Todo::new(content, due, daily)?;
        self.todos.push(todo);
        Ok(())
    }
}

impl Todo {
    fn new(content: String, due: Option<String>, daily: bool) -> Result<Self, TodoError> {
        if daily {
            Ok(Todo::Daily {
                content,
                streak: 0,
                last_marked_done: None,
                last_marked_done_backup: None,
                longest_streak: 0,
            })
        } else if let Some(deadline) = due {
            let valid_date = chrono::NaiveDate::parse_from_str(&deadline, "%Y-%m-%d");
            match valid_date {
                Ok(_) => Ok(Todo::Scheduled {
                    content,
                    due: deadline,
                    done: false,
                }),
                Err(_) => Err(TodoError::InvalidDate),
            }
        } else {
            Ok(Todo::Regular {
                content,
                done: false,
            })
        }
    }
}

impl fmt::Display for Todo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Todo::Regular { content, done } => {
                write!(f, "{} {}", if *done { "" } else { "" }, content)
            }
            Todo::Daily {
                content,
                streak,
                last_marked_done,
                longest_streak,
                last_marked_done_backup: _,
            } => {
                use std::cmp::Ordering;
                let today = chrono::Local::now().date_naive();

                let mut done = false;

                let last_done = last_marked_done
                    .as_ref()
                    .map(|date| date.parse::<chrono::NaiveDate>().unwrap());

                if let Some(date) = last_done {
                    match date.cmp(&today) {
                        Ordering::Less | Ordering::Greater => {
                            done = false;
                        }
                        Ordering::Equal => {
                            done = true;
                        }
                    }
                }

                write!(
                    f,
                    "{} {} (daily, streak: {}{}){}",
                    if done { "" } else { "" },
                    content,
                    streak,
                    if *streak < *longest_streak {
                        format!(", longest: {}", *longest_streak)
                    } else {
                        String::new()
                    },
                    match last_marked_done {
                        Some(date) => format!(" (last done: {date})"),
                        None => String::new(),
                    }
                )
            }
            Todo::Scheduled {
                content,
                due: deadline,
                done,
            } => {
                write!(
                    f,
                    "{} {} (scheduled, deadline: {}){}",
                    if *done { "" } else { "" },
                    content,
                    deadline,
                    if *done { "" } else { " (pending)" }
                )
            }
        }
    }
}

fn load_or_initialize_rem(file_path: &std::path::Path) -> Rem {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(file_path)
        .expect("Failed to open or create rem.json");

    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read file");

    if contents.trim().is_empty() {
        Rem { todos: Vec::new() }
    } else {
        serde_json::from_str(&contents).expect("Failed to parse rem.json")
    }
}

fn save_rem(file_path: &std::path::Path, rem: &Rem) {
    let json = serde_json::to_string_pretty(rem).expect("Failed to serialize Rem");
    fs::write(file_path, json).expect("Failed to write to rem.json");
}

#[derive(Debug)]
enum TodoError {
    InvalidIndex { min: usize, max: usize },
    InvalidDate,
}

impl fmt::Display for TodoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TodoError::InvalidIndex { min, max } => {
                write!(f, "Invalid index, valid range is {min}-{max}")
            }
            TodoError::InvalidDate => {
                write!(f, "Invalid date format, should be YYYY-MM-DD")
            }
        }
    }
}

impl std::error::Error for TodoError {}
