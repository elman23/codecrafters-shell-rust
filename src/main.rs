#[allow(unused_imports)]
use std::io::{self, Write};

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";

const SHELL_BUILTINS: &[&str] = &[EXIT_CMD, ECHO_CMD, TYPE_CMD];


fn repl_loop() {
    loop {
        // Display prompt.
        print!("$ ");
        io::stdout().flush().unwrap();
        
        // Wait for user input.
        let mut command: String = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = String::from(command.trim());

        // Exit command.
        if command == String::from(EXIT_CMD) {
            break;
        }

        // Echo command.
        if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
            let arguments = &command[(ECHO_CMD.len() + 1)..];
            println!("{}", arguments);
            continue;
        }

        if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
            let arguments = &command[(TYPE_CMD.len() + 1)..];
            if SHELL_BUILTINS.contains(&arguments) {
                println!("{} is a shell builtin", arguments);
            } else {
                println!("{}: not found", arguments);
            }
            continue;
        }

        println!("{}: command not found", command);
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}
