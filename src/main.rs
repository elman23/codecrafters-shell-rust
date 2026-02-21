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
        let mut result;

        // TODO: Check if redirect
        let (redirect_stdout, redirect_stderr, index) = executor::get_stdout_redirect(&command);
        if redirect_stdout.is_some() || redirect_stderr.is_some() {
            let index = index.unwrap() - 1;
            // args = &command[command_path.len()..index];
            command = command[..index].trim().to_string();
            // TODO: Fix, dirty.
            if command.ends_with("1") || command.ends_with("2") {
                command = command[..command.len() - 1].trim().to_string();
            }
        }

        println!("Command: {command}");

        if command == String::from(EXIT_CMD) {
            break;
        } else if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
            result = builtins::handle_echo_command(&command);
        } else if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
            result = builtins::handle_type_command(&command);
        } else if command == String::from(PWD_CMD) {
            result = builtins::print_pwd();
        } else if command.starts_with(&*format!("{} ", &CD_CMD)) {
            match builtins::handle_cd_command(&command) {
                Ok(_) => {
                    result = ;
                }
                Err(e) => {
                    result = e.to_string();
                }
            }
        } else {
            result = executor::exec_command(&command);
        }
        
        if result.ends_with('\n') {
            let pos = result.len() - 1;
            result.replace_range(pos..=pos, "");
        }

        if result == "" { 
            continue;
        }

        println!("Result: {result}");

        if redirect_stdout.is_some() {
            match result {
                Ok(output) => {
                    let _ = fs::write(redirect_stdout.unwrap(), result);
                }
            }


        } else if redirect_stderr.is_some() {
            let _ = fs::write(redirect_stderr.unwrap(), result);
        } else {
            println!("{}", result);
        }
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}