use core::fmt;

use serde::{Deserialize, Serialize};

use crate::cli::NewTodoParams;

#[derive(Debug, Deserialize, Serialize)]
pub struct Rem {
    pub todos: Vec<Todo>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Todo {
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

impl fmt::Display for Rem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, todo) in self.todos.iter().enumerate() {
            writeln!(f, "{}. {}", i + 1, todo)?;
        }
        Ok(())
    }
}

impl Rem {
    pub fn print_pending(&self) {
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

    pub fn toggle_todo(&mut self, index: usize) -> Result<(), TodoError> {
        if index == 0 || index > self.todos.len() {
            return Err(TodoError::InvalidIndex {
                min: 1,
                max: self.todos.len(),
            });
        }
        let todo = &mut self.todos[index - 1];

        todo.toggle_done();
        Ok(())
    }

    pub fn add_todo(&mut self, params: &NewTodoParams) -> Result<(), TodoError> {
        let todo = Todo::new(params)?;
        self.todos.push(todo);
        Ok(())
    }

    pub fn update_state(&mut self) {
        for todo in &mut self.todos {
            match todo {
                Todo::Daily {
                    streak,
                    last_marked_done,
                    last_marked_done_backup,
                    longest_streak,
                    ..
                } => {
                    let today = chrono::Local::now().date_naive();
                    if let Some(last_done) = last_marked_done {
                        if let Ok(last_done) =
                            chrono::NaiveDate::parse_from_str(last_done, "%Y-%m-%d")
                        {
                            use std::cmp::Ordering;
                            match last_done.cmp(&today) {
                                Ordering::Less => {
                                    if (today - last_done) > chrono::Duration::days(1) {
                                        if *streak > *longest_streak {
                                            *longest_streak = *streak;
                                        }
                                        *streak = 0;
                                    }
                                    last_marked_done_backup.clone_from(last_marked_done);
                                    // *last_marked_done = None; // shouldn't be necessary
                                }
                                // should be impossible without editing the file
                                Ordering::Greater => {
                                    *streak = 1;
                                    *last_marked_done =
                                        Some(chrono::Local::now().format("%Y-%m-%d").to_string());
                                }
                                Ordering::Equal => {}
                            }
                        }
                    }
                }
                Todo::Scheduled { .. } | Todo::Regular { .. } => {}
            }
        }
    }
}

impl Todo {
    pub fn new(params: &NewTodoParams) -> Result<Self, TodoError> {
        if params.daily {
            Ok(Todo::Daily {
                content: params.content.clone(),
                streak: 0,
                last_marked_done: None,
                last_marked_done_backup: None,
                longest_streak: 0,
            })
        } else if let Some(due) = params.due.clone() {
            let valid_date = chrono::NaiveDate::parse_from_str(&due, "%Y-%m-%d");
            match valid_date {
                Ok(_) => Ok(Todo::Scheduled {
                    content: params.content.clone(),
                    due,
                    done: false,
                }),
                Err(_) => Err(TodoError::InvalidDate),
            }
        } else {
            Ok(Todo::Regular {
                content: params.content.clone(),
                done: false,
            })
        }
    }

    pub fn toggle_done(&mut self) {
        match self {
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
                let Ok(date) = chrono::NaiveDate::parse_from_str(deadline, "%Y-%m-%d") else {
                    return Err(fmt::Error);
                };

                let status = if date < chrono::Local::now().date_naive() && !*done {
                    " (overdue)"
                } else {
                    " (due)"
                };

                write!(
                    f,
                    "{} {} (scheduled, deadline: {}){}",
                    if *done { "" } else { "" },
                    content,
                    deadline,
                    status
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum TodoError {
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
