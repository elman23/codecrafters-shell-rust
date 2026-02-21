#[allow(unused_imports)]
use std::fs;
use std::io::{self, Write};

mod executor;
mod builtins;
mod output;

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";
const PROMPT: &str = "$ ";
const PWD_CMD: &str = "pwd";
const CD_CMD: &str = "cd";

fn clean_last_newline(mut s: String) -> String {
    if s.starts_with('\n') {
        s.replace_range(0..=0, "");
    }
    if s.ends_with('\n') {
        let pos = s.len() - 1;
        s.replace_range(pos..=pos, "");
    }
    s
}

fn repl_loop() {
    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        
        let mut command = executor::read_command();
        let result;

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

        if command == String::from(EXIT_CMD) {
            break;
        } else if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
            result = builtins::handle_echo_command(&command);
        } else if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
            result = builtins::handle_type_command(&command);
        } else if command == String::from(PWD_CMD) {
            result = builtins::print_pwd();
        } else if command.starts_with(&*format!("{} ", &CD_CMD)) {
            result = builtins::handle_cd_command(&command);
        } else {
            result = executor::exec_command(&command);
        }

        if redirect_stdout.is_some() {
            // match result {
            //     Ok(output) => {
            //         if output != "" { 
            //             let _ = fs::write(redirect_stdout.unwrap(), clean_last_newline(output)); 
            //         }
            //     }
            //     Err(error) => {
            //         if error != "" {
            //             println!("{}", clean_last_newline(error));
            //         }
            //     }
            // }
            match result.output {
                Some(output) => {
                    let _ = fs::write(redirect_stdout.unwrap(), clean_last_newline(output)); 
                }, 
                None => { }
            }
            match result.error {
                Some(error) => {
                    println!("{}", clean_last_newline(error));
                }, 
                None => { }
            }
        } else if redirect_stderr.is_some() {
            // match result {
            //     Ok(output) => {
            //         if output != "" {
            //             println!("{}", clean_last_newline(output));
            //         }
            //     }
            //     Err(error) => {
            //         if error != "" { 
            //             let _ = fs::write(redirect_stderr.unwrap(), clean_last_newline(error)); 
            //         }
            //     }
            // }
            match result.output {
                Some(output) => {
                    println!("{}", clean_last_newline(output)); 
                }, 
                None => { }
            }
            match result.error {
                Some(error) => {
                    let _ = fs::write(redirect_stderr.unwrap(), clean_last_newline(error));
                }, 
                None => { }
            }
        } else {
            // match result {
            //     Ok(output) => {
            //         if output != "" {
            //             println!("{}", clean_last_newline(output));
            //         }
            //     }
            //     Err(error) => {
            //         if error != "" {
            //             println!("{}", clean_last_newline(error));
            //         }
            //     }
            // }
            match result.output {
                Some(output) => {
                    println!("{}", clean_last_newline(output)); 
                }, 
                None => { }
            }
            match result.error {
                Some(error) => {
                    println!("{}", clean_last_newline(error));
                }, 
                None => { }
            }
        }
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}