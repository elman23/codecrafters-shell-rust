#[allow(unused_imports)]
use std::fs;
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
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        
        let mut command = executor::read_command();
        let mut output;

        // TODO: Check if redirect
        let (redirect, index) = executor::get_stdout_redirect(&command);
        if redirect.is_some() {
            let index = index.unwrap() - 1;
            // args = &command[command_path.len()..index];
            command = command[..index].trim().to_string();
            // TODO: Fix, dirty.
            if command.ends_with("1") {
                command = command[..command.len() - 1].trim().to_string();
            }
        }

        if command == String::from(EXIT_CMD) {
            break;
        } else if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
            output = builtins::handle_echo_command(&command);
        } else if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
            output = builtins::handle_type_command(&command);
        } else if command == String::from(PWD_CMD) {
            output = builtins::print_pwd();
        } else if command.starts_with(&*format!("{} ", &CD_CMD)) {
            match builtins::handle_cd_command(&command) {
                Ok(_) => {
                    output = "".to_string();
                }
                Err(e) => {
                    output = e.to_string();
                }
            }
        } else {
            output = executor::exec_command(&command).expect("Failure");
        }
        
        if output.ends_with('\n') {
            let pos = output.len() - 1;
            output.replace_range(pos..=pos, "");
        }

        if output == "" { 
            continue;
        }

        if redirect.is_some() {
            let _ = fs::write(redirect.unwrap(), output);
        } else {
            println!("{}", output);
        }
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}