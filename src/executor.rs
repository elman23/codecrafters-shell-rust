use std::arch::x86_64::CpuidResult;
use std::fs::{self, File, OpenOptions};
#[allow(unused_imports)]
use std::io::{self, Write, Error};
use std::os::unix::process::ExitStatusExt;
use std::process::{Child, ChildStdout, Command, ExitCode, ExitStatus, Output, Stdio};
use std::vec;
use rustyline::Editor;
use rustyline::history::FileHistory;
use crate::{my_helper::MyHelper, output::MyOutput};
use crate::builtins;
use crate::utils::{self, get_redirect};

fn clean_last_newline(s: &String) -> String {
    // s.trim().to_string()
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

pub fn exec_command(command: &str, input: Option<Stdio>) -> (Stdio, Stdio) {

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

    let mut binding = Command::new(command_name);
    let my_command: &mut Command = binding
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
        

    if let Some(i) = input {
        my_command.stdin(i);
    }
    let mut my_command = my_command.spawn()
        .unwrap();
    let stdout = my_command.stdout.take().unwrap();
    let stderr = my_command.stderr.take().unwrap();

    (stdout.into(), stderr.into())
    
}

pub fn execute(mut command: String) -> ExitStatus {
    let mut result: Output;

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

    if builtins::is_builtin(&command.split(' ').next().unwrap()) {
        result = builtins::execute_builtin(&command);
    } else {
        let cmd = command.clone(); // TODO: Fix.
        match execute_piped(command) {
            Ok(r) => {
                result = r;
                result.status = ExitStatusExt::from_raw(0);
            },
            Err(e) => {
                // eprint!("{}", e);
                // TODO: Flush?
                // print_cleaned(&format!("{}: command not found", cmd));
                // print!("{}: command not found", cmd);
                // let _ = std::io::stdout().flush();
                // let _ = std::io::stderr().flush();
                // print_cleaned(&format!("{}: command not found", cmd));
                let _ = writeln!(std::io::stderr(), "{}: command not found", cmd);
                return ExitStatusExt::from_raw(0);
            }
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
            let _ = writeln!(std::io::stderr(), "{}", &String::from_utf8(result.stderr).unwrap());
        }
    }
    result.status
}

pub fn execute_piped(input: String) -> io::Result<std::process::Output> {

    let cmds: Vec<&str> = input
                        .split('|')
                        .map(|c| c.trim())
                        .collect(); 

    let mut children: Vec<Child> = Vec::new();
    let mut previous: Option<ChildStdout> = None;

    for (i, c) in cmds.iter().enumerate() {
        // let split: Vec<&str> = c.split(" ").collect();

        // Command
        // let cmd_name = split[0];
        let command_path = get_command_path(c);
        let cmd_name = command_path.split("/").last().unwrap_or("Failed to parse command name");
        let cmd_name = cleanup_name(cmd_name);

        
        let mut cmd = Command::new(&cmd_name);
        
        // Arguments - TODO: FIX
        let args = get_command_args(&c[command_path.len()..]);
        cmd.args(args);
        // let mut j = 1;
        // while j < split.len() {
        //     cmd.arg(split[j]);
        //     j += 1;
        // }

        if let Some(stdin) = previous.take() {
            cmd.stdin(stdin);
        }

        if i < cmds.len() - 1 {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }

        let mut child = cmd.spawn()?;

        if i < cmds.len() - 1 {
            previous = child.stdout.take();
        }

        children.push(child);
    }

    let last = children.pop().unwrap();
    let output = last.wait_with_output()?;

    for mut child in children {
        child.wait().unwrap();
    }

    Ok(output)
}