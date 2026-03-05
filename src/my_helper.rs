use std::io;
use std::io::{Write};

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{Validator, ValidationContext, ValidationResult};
use rustyline::{Context, Helper};
use std::cell::{Cell, RefCell};
use crate::path_checker::check_path;

pub struct MyHelper {
    tab_count: Cell<u32>,
    last_input: RefCell<Vec<String>>,
}

impl MyHelper {
    pub fn new() -> Self {
        Self {
            tab_count: Cell::new(0),
            last_input: RefCell::new(Vec::new()),
        }
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let mut completed_line = String::new();

        self.tab_count.set(self.tab_count.get() + 1);

        if self.tab_count.get() == 1 {
            match line {
                "ech" => {
                    completed_line = String::from("echo ");
                    print!("o ");
                    io::stdout().flush().unwrap();
                },
                "exi" => {
                    completed_line = String::from("exit ");
                    print!("t ");
                    io::stdout().flush().unwrap();
                },
                _ => {
                    let complete_command = check_path(line);
                    match complete_command {
                        Some(v) => {
                            if v.len() == 1 {
                                completed_line = v[0].clone();
                            } else {
                                completed_line.push_str(&format!("{}\x07", line));
                            }
                            *self.last_input.borrow_mut() = v;
                        },
                        None => {
                            println!("\x07");
                        }
                    }
                }
            }
        } else {
            completed_line.push_str(line);
            completed_line.push('\n');
            for s in self.last_input.borrow().iter() {
                completed_line.push_str(&format!("{}  ", s));
            }
            self.tab_count.set(0);
        }

        Ok((0, vec![Pair {display: completed_line.clone(), replacement: completed_line}]))
    }

    // fn complete(
    //     &self,
    //     line: &str,
    //     pos: usize,
    //     _ctx: &Context<'_>,
    // ) -> rustyline::Result<(usize, Vec<Pair>)> {
    //     let current_input: Vec<String> = line[..pos]
    //         .split_whitespace()
    //         .map(String::from)
    //         .collect();

    //     if *self.last_input.borrow() != current_input {
    //         self.tab_count.set(0);
    //         *self.last_input.borrow_mut() = current_input;
    //     }

    //     let count = self.tab_count.get() + 1;
    //     self.tab_count.set(count);

    //     let mut completed_line = String::new();

    //     if count == 1 {
    //         match line {
    //             "ech" => {
    //                 completed_line = String::from("echo ");
    //                 print!("o ");
    //                 io::stdout().flush().unwrap();
    //             }
    //             "exi" => {
    //                 completed_line = String::from("exit ");
    //                 print!("t ");
    //                 io::stdout().flush().unwrap();
    //             }
    //             _ => {
    //                 let complete_command = check_path(line);
    //                 match complete_command {
    //                     Some(v) => {
    //                         if v.len() == 1 {
    //                             completed_line = format!("{} ", v[0]);
    //                         } else {
    //                             // Multiple matches: ring bell, keep current input
    //                             completed_line = format!("{}\x07", line);
    //                         }
    //                     }
    //                     None => {
    //                         // No match: ring bell
    //                         print!("\x07");
    //                         io::stdout().flush().unwrap();
    //                         completed_line = line.to_string();
    //                     }
    //                 }
    //             }
    //         }
    //     } else {
    //         // Second tab: show all candidates below the prompt
    //         let complete_command = check_path(line);
    //         match complete_command {
    //             Some(v) => {
    //                 // Print all matches on a new line, then restore the input
    //                 print!("\n");
    //                 for s in &v {
    //                     print!("{}  ", s);
    //                 }
    //                 io::stdout().flush().unwrap();
    //             }
    //             None => {
    //                 print!("\x07");
    //                 io::stdout().flush().unwrap();
    //             }
    //         }
    //         // Restore the original line so the cursor position is unchanged
    //         completed_line = line.to_string();
    //     }

    //     Ok((0, vec![Pair {
    //         display: completed_line.clone(),
    //         replacement: completed_line,
    //     }]))
    // }
}

impl Hinter for MyHelper {
    type Hint = String;
}

impl Highlighter for MyHelper {}

impl Validator for MyHelper {
    fn validate(
        &self,
        _ctx: &mut ValidationContext,
    ) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for MyHelper {}

// fn main() -> rustyline::Result<()> {
//     let mut rl = Editor::new()?;
//     rl.set_helper(Some(MyHelper));
//     // rl.set_helper(Some(()));
//     // loop {    
//         // rl.readline("> ")?;
//     // }
//     rl.readline("> ")?;
//     Ok(())
// }