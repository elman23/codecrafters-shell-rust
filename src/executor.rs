use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ChildStdout, Command, ExitStatus, Output, Stdio};
use crate::builtins;
use crate::utils;
use crate::constants;

fn clean_last_newline(s: &String) -> String {
    s.strip_suffix('\n').unwrap_or(s).to_string()
}

fn print_cleaned(s: &String) {
    if s != "" {
        let _ = writeln!(std::io::stdout(), "{}", clean_last_newline(s));
    }
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

pub fn execute(mut command: String, history: &Vec<String>) -> std::io::Result<u8> {
    let result: Output;

    // Check if redirect
    let redirect_info = utils::get_redirect(&command);
    let redirect_stdout = redirect_info.redirect_stdout_file;
    let redirect_stderr = redirect_info.redirect_stderr_file;
    let index = redirect_info.file_index_start;
    let append_stdout = redirect_info.append_stdout;
    let append_stderr = redirect_info.append_stderr;
    if redirect_stdout.is_some() || redirect_stderr.is_some() {
        let index = index.unwrap() - 1;
        command = command[..index].trim().to_string();
    }

    match execute_piped(&command, history) {
        Ok(r) => {
            result = r;
            if command.starts_with(constants::EXIT_CMD) {
                return Ok(1);
            }
        },
        Err(_) => {
            let _ = writeln!(std::io::stderr(), "{}: command not found", command);
            return Ok(0);
        }
    }

    if redirect_stdout.is_some() {
        let stdout_file = redirect_stdout.unwrap();
        if !fs::exists(&stdout_file).unwrap() {
            let _ = File::create(&stdout_file);
        }
        if !result.stdout.is_empty() {
            if append_stdout {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&stdout_file)
                    .unwrap();
                let cleaned_output = clean_last_newline(&String::from_utf8(result.stdout).unwrap());
                if cleaned_output != "" {
                    let _ = writeln!(file, "{}", cleaned_output).unwrap();
                }
            } else {
                let mut file = File::create(stdout_file).unwrap();
                let cleaned_output = clean_last_newline(&String::from_utf8(result.stdout).unwrap());
                if cleaned_output != "" {
                    let _ = writeln!(file, "{}", cleaned_output).unwrap(); 
                }
            }
        }
        if !result.stderr.is_empty() {
            let _ = writeln!(std::io::stderr(), "{}", &String::from_utf8(result.stderr).unwrap().trim());
        }
    } else if redirect_stderr.is_some() {
        let stderr_file = redirect_stderr.unwrap();
        if !fs::exists(&stderr_file).unwrap() {
            let _ = File::create(&stderr_file);
        }
        if !result.stdout.is_empty() {
            print_cleaned(&String::from_utf8(result.stdout).unwrap()); 
        }
        if !result.stderr.is_empty() {
            if append_stderr {
                let mut file = OpenOptions::new()
                    .append(true)
                    .create(true)
                    .open(&stderr_file)
                    .unwrap();
                let cleaned_error = clean_last_newline(&String::from_utf8(result.stderr).unwrap());
                if cleaned_error != "" {
                    let _ = writeln!(file, "{}", cleaned_error).unwrap();
                }  
            } else {
                let mut file = File::create(stderr_file).unwrap();
                let cleaned_error = clean_last_newline(&String::from_utf8(result.stderr).unwrap());
                if cleaned_error != "" {
                    let _ = writeln!(file, "{}", cleaned_error).unwrap();
                }
            }
        }
    } else {
        if !result.stdout.is_empty() {
            print_cleaned(&String::from_utf8(result.stdout).unwrap());
        }
        if !result.stderr.is_empty() {
            let _ = writeln!(std::io::stderr(), "{}", &String::from_utf8(result.stderr).unwrap().trim());
        }
    }
    Ok(0)
}

pub fn execute_piped(input: &str, history: &Vec<String>) -> io::Result<std::process::Output> {

    let cmds: Vec<&str> = input
                        .split('|')
                        .map(|c| c.trim())
                        .collect(); 

    let mut children: Vec<Child> = Vec::new();
    let mut previous: Option<ChildStdout> = None;
    let mut previous_ec: ExitStatus = ExitStatusExt::from_raw(0);
    let mut previous_out: Option<Vec<u8>> = None;
    let mut previous_err: Option<Vec<u8>> = None;
    let mut is_last_builtin = false;

    for (i, c) in cmds.iter().enumerate() {

        if builtins::is_builtin(&c.split(' ').next().unwrap()) {
            let result: Output = builtins::execute_builtin(&c, history);
            previous_ec = result.status;
            previous_out = match result.stdout.len() {
                0 => None,
                _ => {
                    // Add new line to STDOUT if not present.
                    Some(result.stdout)
                },
            };
            previous_err = match result.stderr.len() {
                0 => None,
                _ => {
                    // Add new line to STDERR if not present.
                    Some(result.stderr)
                },
            };
            previous = None;
            if i == cmds.len() - 1 {
                is_last_builtin = true;
            }
            continue;
        }

        // Command
        let command_path = get_command_path(c);
        let cmd_name = command_path.split("/").last().unwrap_or("Failed to parse command name");
        let cmd_name = cleanup_name(cmd_name);
        
        let mut cmd = Command::new(&cmd_name);
        
        // Arguments
        let args = get_command_args(&c[command_path.len()..]);
        cmd.args(args);

        if let Some(stdin) = previous.take() {
            cmd.stdin(stdin);
        }

        if previous_out.is_some() || previous_err.is_some() {
            cmd.stdin(Stdio::piped());
        }

        if i < cmds.len() - 1 || cmds.len() == 1 {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }

        let mut child = cmd.spawn()?;

        if let Some(s) = previous_out.take() {
            child.stdin.take().unwrap().write_all(&s)?;
        } else if let Some(s) = previous_err.take() {
            child.stdin.take().unwrap().write_all(&s)?;
        }

        if i < cmds.len() - 1 {
            previous = child.stdout.take();
            previous_out = None;
            previous_err = None;
        }

        children.push(child);
    }

    let mut output: Option<Output> = None;

    if !children.is_empty() {
        if !is_last_builtin {
            let last = children.pop().unwrap();
            output = Some(last.wait_with_output()?);
        }
        for mut child in children {
            child.wait().unwrap();
        }
    }

    if previous_out.is_none() && previous_err.is_none() {
        Ok(output.unwrap_or(Output { 
            status: previous_ec, 
            stdout: vec![], 
            stderr: vec![] 
        }))
    } else {
        Ok(Output {
            status: previous_ec,
            stdout: previous_out.unwrap_or(vec![]),
            stderr: previous_err.unwrap_or(vec![]),
        })
    }
}