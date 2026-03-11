#[allow(unused_imports)]
use std::io::{self, Write, Error};
use std::io::{Read, Stderr};
use std::process::{Command, Stdio};

use crate::output::MyOutput;
use crate::utils::get_redirect;

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

pub fn exec_command(command: &str, input: Option<String>) -> MyOutput {

    let command_path = get_command_path(command);
    let command_name = command_path.split("/").last().unwrap_or("Failed to parse command name");
    let command_name = cleanup_name(command_name);

    let args;

    // TODO: Handle redirect in main function.
    let redirect_info = get_redirect(command);
    let redirect_stdout = redirect_info.redirect_stdout_file;
    let redirect_stderr = redirect_info.redirect_stderr_file;
    let index = redirect_info.file_index_start;
    if redirect_stdout.is_some() || redirect_stderr.is_some() {
        let index = index.unwrap() - 1;
        args = &command[command_path.len()..index];
    } else {
        args = &command[command_path.len()..];
    }

    let args = args.trim_start();
    let args = get_command_args(args);

    let mut my_command = Command::new(&command_name)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    match input {
        Some(i) => {
            my_command.stdin.take().unwrap().write_all(i.as_bytes()).unwrap();
        },
        None => { }
    }

    // match my_command.wait_with_output() {
    //     Ok(result) => {
    //         return MyOutput {
    //             status: if result.status.success() {
    //                 0
    //             } else {
    //                 1
    //             },
    //             output: Some(String::from_utf8_lossy(&result.stdout).to_string()),
    //             error: Some(String::from_utf8_lossy(&result.stderr).to_string())
    //         }
    //     }
    //     Err(error) => {
    //         return MyOutput {
    //             status: 1,
    //             output: None,
    //             error: Some(error.to_string())
    //         }
    //     }
    // }

    let mut stdout = my_command.stdout.take().unwrap();
    let mut stderr = my_command.stderr.take().unwrap();
    
    let mut out_buf = Vec::new();
    let mut err_buf = Vec::new();

    stdout.read_to_end(&mut out_buf).unwrap();
    stderr.read_to_end(&mut err_buf).unwrap();

    // let status = my_command.wait().unwrap();
    if let Some(_) = my_command.stderr.take() {
        MyOutput {
            status: 1,
            output: if out_buf.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&out_buf).into_owned())
            },
            error: if err_buf.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&err_buf).into_owned())
            },
        }
    } else {
        MyOutput {
            status: 0,
            output: if out_buf.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&out_buf).into_owned())
            },
            error: if err_buf.is_empty() {
                None
            } else {
                Some(String::from_utf8_lossy(&err_buf).into_owned())
            },
        }
    }
    
}