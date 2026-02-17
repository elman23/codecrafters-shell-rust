use std::io::{self, Write};

use crate::builtins;
use crate::executor;
use crate::parser::Command;

const PROMPT: &str = "$ ";

pub fn run() {
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        let input = read_line();
        if input.is_empty() {
            // EOF (Ctrl-D)
            break;
        }

        let Some(cmd) = Command::parse(&input) else {
            continue;
        };

        if builtins::is_builtin(&cmd.name) {
            let should_exit = builtins::execute(&cmd);
            if should_exit {
                break;
            }
        } else {
            executor::run(&cmd);
        }
    }
}

fn read_line() -> String {
    let mut buf = String::new();
    match io::stdin().read_line(&mut buf) {
        Ok(0) | Err(_) => String::new(),
        Ok(_) => buf.trim_end_matches('\n').trim_end_matches('\r').to_string(),
    }
}