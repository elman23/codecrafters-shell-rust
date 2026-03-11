#[allow(unused_imports)]
use std::fs;
use std::{fs::{File, OpenOptions}, io::{Write}};

use rustyline::Editor;
use crate::{my_helper::MyHelper, output::MyOutput};

mod executor;
mod builtins;
mod output;
mod utils;
mod my_helper;
mod path_checker;

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";
const PROMPT: &str = "$ ";
const PWD_CMD: &str = "pwd";
const CD_CMD: &str = "cd";

fn clean_last_newline(s: &String) -> String {
    s.trim().to_string()
}

fn print_cleaned(s: &String) {
    if s != "" {
        println!("{}", clean_last_newline(s));
    }
}

fn repl_loop() {
    let config = rustyline::Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl = Editor::with_config(config).unwrap();
    let helper = MyHelper::new();
    rl.set_helper(Some(helper));    
    loop {
        let input = rl.readline(PROMPT).unwrap();
        let commands: Vec<&str> = input
                        .split('|')
                        .map(|c| c.trim())
                        .collect(); 
        let mut previous_output: MyOutput = MyOutput { status: 0, output: None, error: None };
        let mut count = 1;
        for command in commands {
            let mut input: Option<String>;
            if count > 1 {
                input = previous_output.output;
            } else {
                input = None;
            }
            let my_output = execute(command.to_string(), input);
            if my_output.status == 1 {
                break;
            }
            previous_output = my_output;
            count += 1;
        }
    }
}

fn execute(mut command: String, input: Option<String>) -> MyOutput {
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
        return MyOutput { status: 1, output: None, error: None };
    } else if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
        result = builtins::handle_echo_command(&command);
    } else if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
        result = builtins::handle_type_command(&command);
    } else if command == String::from(PWD_CMD) {
        result = builtins::print_pwd();
    } else if command.starts_with(&*format!("{} ", &CD_CMD)) {
        result = builtins::handle_cd_command(&command);
    } else {
        result = executor::exec_command(&command, input);
    }

    if redirect_stdout.is_some() {
        let stdout_file = redirect_stdout.unwrap();
        if !fs::exists(&stdout_file).unwrap() {
            let _ = File::create(&stdout_file);
        }
        match result.output {
            Some(ref output) => {
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
            Some(ref error) => {
                print_cleaned(error);
            }, 
            None => { 
            }
        }
    } else if redirect_stderr.is_some() {
        let stderr_file = redirect_stderr.unwrap();
        if !fs::exists(&stderr_file).unwrap() {
            let _ = File::create(&stderr_file);
        }
        match result.output {
            Some(ref output) => {
                print_cleaned(output); 
            }, 
            None => { }
        }
        match result.error {
            Some(ref error) => {
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
            None => { 
            }
        }
    } else {
        match result.output {
            Some(ref output) => {
                print_cleaned(output);
            }, 
            None => { }
        }
        match result.error {
            Some(ref error) => {
                print_cleaned(error);
            }, 
            None => {
            }
        }
    }
    result
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}