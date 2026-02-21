#[allow(unused_imports)]
use std::io::{self, Write, Error};

use std::process::{Command, Stdio};

pub fn read_command() -> String {
    let mut command: String = String::new();
    io::stdin().read_line(&mut command).unwrap();
    command = String::from(command.trim());
    command
}

fn split_char(ch: char, input: &str) -> Vec<String> {
    let double_quotes = ch == '"';
    
    let mut result = Vec::new();
    let mut in_quotes = false;
    let mut escaped = false;
    let mut current = String::new();

    for c in input.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }
        if c == ch {
            if in_quotes {
                result.push(std::mem::take(&mut current));
            }
            in_quotes = !in_quotes;
        } else if c == '\\' && (double_quotes || !in_quotes) {
            escaped = true;
        } else {
            current.push(c);
        }
    }

    result
}

fn get_command_args(args: &str) -> Vec<String> {
    let mut handle_slashes = false;
    let mut args = if args.contains('\"') {
        split_char('\"', args)
    } else if args.contains('\'') {
        split_char('\'', args)
    } else {
        handle_slashes = true;
        args.split_whitespace().map(|s| s.to_string()).collect()
    };

    for arg in &mut args {
        if handle_slashes {
            if arg.contains("\\\\") {
                *arg = arg.replace("\\\\", "\\");
            } else {
                *arg = arg.replace("\\", "");
            }
        }
        *arg = arg.trim().to_string();
    }

    args
}

fn cleanup_name(name: &str) -> String {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;
    let mut cleaned = String::new();

    for c in name.chars() {
        if escaped {
            cleaned.push(c);
            escaped = false;
            continue;
        }

        if c == '\\' && in_double_quote {
            escaped = true;
            continue;
        }

        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            continue;
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            continue;
        } else {
            cleaned.push(c);
        }
    }

    cleaned
}

fn get_command_path(s: &str) -> String {
    let mut command_path = String::new();
    
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    
    for c in s.chars() {
        if c == ' ' && !in_double_quote && !in_single_quote {
            break;
        }
        if c == '"' {
            in_double_quote = !in_double_quote;        
        }
        if c == '\'' {
            in_single_quote = !in_single_quote
        }
        command_path.push(c);
    }
    
    command_path
}

pub fn get_stdout_redirect(input: &str) -> (Option<String>, Option<String>, Option<usize>) {
    let mut redirect_stdout = false;
    let mut redirect_stderr = false;
    let mut redirect_stdout_file = String::new();
    let mut redirect_stderr_file = String::new();
    let mut redirect_index: usize = 0;
    let mut previous: char = ' ';

    let mut in_single_quote = false;
    let mut in_double_quote = false;

    let mut counter = 0;
    for c in input.chars() {
        counter += 1;
        if redirect_stdout {
            redirect_stdout_file.push(c);
            previous = c;
            continue;
        }
        if redirect_stderr {
            redirect_stderr_file.push(c);
            previous = c;
            continue;
        }
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            previous = c;
            continue;
        }
        if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            previous = c;
            continue;
        }
        if c == '1' {
            previous = c;
            continue;
        }
        if c == '2' {
            previous = c;
            continue;
        }
        if c == '>' && !in_single_quote && !in_double_quote {
            if previous == '2' {
                redirect_stderr = true;
            } else {
                redirect_stdout = true;
            }
            if redirect_index == 0 {
                redirect_index = counter;
            }
        }
        previous = c;
    }

    if redirect_stdout {
        (Some(redirect_stdout_file.trim().to_string()), None, Some(redirect_index))
    } else if redirect_stderr {
        (None, Some(redirect_stderr_file.trim().to_string()), Some(redirect_index))
    } else {
        (None, None, None)
    }
}

pub fn exec_command(command: &str) -> Result<String, String> {

    let command_path = get_command_path(command);
    let command_name = command_path.split("/").last().unwrap_or("Failed to parse command name");
    let command_name = cleanup_name(command_name);

    let args;

    // TODO: Handle redirect in main function.
    let (redirect_stdout, redirect_stderr, index) = get_stdout_redirect(command);
    if redirect_stdout.is_some() || redirect_stderr.is_some() {
        let index = index.unwrap() - 1;
        args = &command[command_path.len()..index];
    } else {
        args = &command[command_path.len()..];
    }

    let args = args.trim_start();
    let args = get_command_args(args);

    match Command::new(&command_name)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            match child.wait_with_output() {
                Ok(output) => {
                    // return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                    if output.status.success() {
                        let raw_output = String::from_utf8(output.stdout).unwrap();
                        Ok(raw_output)
                    } else {
                        let raw_error = String::from_utf8(output.stderr).unwrap();
                        return Err(raw_error);
                    }
                }
                Err(error) => {
                    return Err(error.to_string());
                }
            }
        }
        Err(_) => {
            return Err(format!("{}: command not found", command_name));
        }
    }
}