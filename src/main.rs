#[allow(unused_imports)]
use std::fs;
use std::{fs::{File, OpenOptions}, io::{self, Write}};

use rustyline::{Context, Editor, Helper};
use crate::my_helper::MyHelper;

mod executor;
mod builtins;
mod output;
mod utils;
mod my_helper;

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";
const PROMPT: &str = "$ ";
const PWD_CMD: &str = "pwd";
const CD_CMD: &str = "cd";

fn clean_last_newline(s: String) -> String {
    s.trim().to_string()
}

fn print_cleaned(s: String) {
    if s != "" {
        println!("{}", clean_last_newline(s));
    }
}

fn repl_loop() {
    let mut rl = Editor::new().unwrap();
    rl.set_helper(Some(MyHelper));    
    loop {
        // print!("{}", PROMPT);
        // io::stdout().flush().unwrap();
        let mut command = rl.readline(PROMPT).unwrap();
        
        // let mut command = executor::read_command();
        let result;

        // TODO: Check if redirect
        let redirect_info = utils::get_redirect(&command);
        let redirect_stdout = redirect_info.redirect_stdout_file;
        let redirect_stderr = redirect_info.redirect_stderr_file;
        let index = redirect_info.file_index_start;
        let append_stdout = redirect_info.append_stdout;
        let append_stderr = redirect_info.append_stderr;
        if redirect_stdout.is_some() || redirect_stderr.is_some() {
            let index = index.unwrap() - 1;
            command = command[..index].trim().to_string();
            // TODO: Fix, dirty hack.
            if command.ends_with("1") || command.ends_with("2") {
                command = command[..command.len() - 1].trim().to_string();
            }
        }

        if command.trim() == EXIT_CMD {
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
            let stdout_file = redirect_stdout.unwrap();
            if !fs::exists(&stdout_file).unwrap() {
                let _ = File::create(&stdout_file);
            }
            match result.output {
                Some(output) => {
                    if append_stdout {
                        let mut file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(&stdout_file)
                            .unwrap();
                        let cleaned_output = clean_last_newline(output);
                        if cleaned_output != "" {
                            writeln!(file, "{}", cleaned_output).unwrap();
                        }
                    } else {
                        let mut file = File::create(stdout_file).unwrap();
                        let cleaned_output = clean_last_newline(output);
                        if cleaned_output != "" {
                            writeln!(file, "{}", cleaned_output).unwrap(); 
                        }
                    }
                }, 
                None => { }
            }
            match result.error {
                Some(error) => {
                    print_cleaned(error);
                }, 
                None => { }
            }
        } else if redirect_stderr.is_some() {
            let stderr_file = redirect_stderr.unwrap();
            if !fs::exists(&stderr_file).unwrap() {
                let _ = File::create(&stderr_file);
            }
            match result.output {
                Some(output) => {
                    print_cleaned(output); 
                }, 
                None => { }
            }
            match result.error {
                Some(error) => {
                    if append_stderr {
                        let mut file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open(&stderr_file)
                            .unwrap();
                        let cleaned_error = clean_last_newline(error);
                        if cleaned_error != "" {
                            writeln!(file, "{}", cleaned_error).unwrap();
                        }  
                    } else {
                        let mut file = File::create(stderr_file).unwrap();
                        let cleaned_error = clean_last_newline(error);
                        if cleaned_error != "" {
                            writeln!(file, "{}", cleaned_error).unwrap();
                        }
                    }
                }, 
                None => { }
            }
        } else {
            match result.output {
                Some(output) => {
                    print_cleaned(output);
                }, 
                None => { }
            }
            match result.error {
                Some(error) => {
                    print_cleaned(error);
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