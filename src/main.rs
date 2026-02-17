#[allow(unused_imports)]
use std::io::{self, Write};

mod executor;
mod builtins;

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";
const PROMPT: &str = "$ ";
const PWD_CMD: &str = "pwd";
const CD_CMD: &str = "cd";

fn repl_loop() {
    loop {
        // Display prompt.
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        
        // Wait for user input.
        let command = executor::read_command();

        // Exit command.
        if command == String::from(EXIT_CMD) {
            break;
        }

        // Echo command.
        if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
            builtins::handle_echo_command(&command);
            continue;
        }

        // Type command.
        if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
            builtins::handle_type_command(&command);
            continue;
        }

        // Pwd command.
        if command == String::from(PWD_CMD) {
            builtins::print_pwd();
            continue;
        }

        // Cd command.
        if command.starts_with(&*format!("{} ", &CD_CMD)) {
            builtins::handle_cd_command(&command);
            continue;
        }

        // Execute command.
        executor::exec_command(&command);
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}