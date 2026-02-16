#[allow(unused_imports)]
use std::io::{self, Write};

use std::fs;
use std::env;
use std::path::PathBuf;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::ffi::OsStr;

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";
const PROMPT: &str = "$ ";
const PWD_CMD: &str = "pwd";
const CD_CMD: &str = "cd";

// TODO: Improve. This requires that each new built-in command shall be added manually.
const SHELL_BUILTINS: &[&str] = &[EXIT_CMD, ECHO_CMD, TYPE_CMD, PWD_CMD, CD_CMD];

fn print_pwd() {
    println!("{}", env::current_dir().unwrap().to_str().unwrap());
}

fn dir_exists(dir: &str) -> bool {
    let dir_path = Path::new(dir);
    return dir_path.exists()
}

fn change_dir(dir: &str) {
    let _ = std::env::set_current_dir(dir);
}

fn is_executable(path: &std::path::Path) -> std::io::Result<bool> {
    let metadata = fs::metadata(path)?;
    let mode = metadata.permissions().mode();
    Ok(mode & 0o111 != 0)
}

fn read_command() -> String {
    let mut command: String = String::new();
    io::stdin().read_line(&mut command).unwrap();
    command = String::from(command.trim());
    command
}

fn handle_echo_command(command: &str) {
    let arguments = &command[(ECHO_CMD.len() + 1)..];
    let mut arguments = String::from(arguments);
    arguments = arguments.replace("\"\"", "");
    arguments = arguments.replace("''", "");
    let arguments = parse_echo_args(&arguments);
    println!("{}", arguments);
}

fn parse_echo_args(input: &str) -> String {
    let mut in_double_quotes = false;
    let mut in_single_quotes = false;
    let mut escaped = false;
    let mut result = String::new();

    for c in input.chars() {
        if escaped {
            result.push(c);
            escaped = false;
            continue;
        }
        if c == '\"' {
            if !in_single_quotes {
                in_double_quotes = !in_double_quotes;
            } else {
                result.push(c);
            }
        } else if c == '\'' {
            if !in_double_quotes {
                in_single_quotes = !in_single_quotes;
            } else {
                result.push(c);
            }
        } else if c =='\\' && !in_single_quotes && !in_double_quotes {
            escaped = true;
        } else if c != ' ' || in_double_quotes || in_single_quotes {
            result.push(c);
        } else if c == ' ' && !result.ends_with(" ") {
            result.push(c);
        }
    }

    result
}

fn handle_type_command(command: &str) {
    let arguments = &command[(TYPE_CMD.len() + 1)..];
    if SHELL_BUILTINS.contains(&arguments) {
        println!("{} is a shell builtin", arguments);
    } else {
        check_type(arguments);
    }
}

fn handle_cd_command(command: &str) {
    let arguments = &command[(CD_CMD.len() + 1)..];
    let dir = arguments.split_whitespace().next().unwrap();

    // The "~" special case.
    if dir == "~" {
        let home_dir = env::var_os("HOME").expect("HOME variable not set!");
        let home_dir = home_dir.to_str().unwrap();
        change_dir(home_dir);
        return;
    }

    if dir_exists(dir) {
        change_dir(dir);
    } else {
        println!("cd: {}: No such file or directory", dir)
    }
}

fn check_type(command: &str) {
    let path_var = env::var_os("PATH").expect("PATH variable not set!");
    let paths: Vec<PathBuf> = env::split_paths(&path_var).collect();

    let mut found = false;

    for path in paths {
        let files: Vec<PathBuf> = get_directory_content(&path);
        for file in files {
            let filename = file.file_stem();
            let executable = is_executable(&file.as_path()).expect("Failed to check execution permissions!");
            if filename == Some(OsStr::new(command)) && executable {
                println!("{} is {}", command, file.to_str().unwrap());
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }

    if !found {
        println!("{}: not found", command);
    }  
}

fn get_directory_content(path: &PathBuf) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(path)
        .unwrap()
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    files.sort();
    files
}

fn split_char(ch: char, input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_quotes = false;
    let mut current = String::new();

    for c in input.chars() {
        if c == ch {
            if in_quotes {
                result.push(std::mem::take(&mut current));
            }
            in_quotes = !in_quotes;
        } else if c == '\\' {
            continue;  
        } else if in_quotes {
            current.push(c);
        }
    }

    result
}

fn clean_char(ch: char, input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut in_quotes = false;
    let mut current = String::new();

    for c in input.chars() {
        if c == ch {
            if in_quotes {
                result.push(std::mem::take(&mut current));
            }
            in_quotes = !in_quotes;
        } else if c != ' ' || in_quotes {
            current.push(c);
        }
    }
    result.push(std::mem::take(&mut current));

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

    if handle_slashes {
        for arg in &mut args {
            if arg.contains("\\\\") {
                *arg = arg.replace("\\\\", "\\");
            } else {
                *arg = arg.replace("\\", "");
            }
        }
    }

    args
}

fn exec_command(command: &str) {

    let path_var = env::var_os("PATH").expect("PATH variable not set!");
    let paths: Vec<PathBuf> = env::split_paths(&path_var).collect();

    let command_path = PathBuf::from(command.split_whitespace().next().unwrap());
    let command_name = command_path.file_stem().unwrap();

    let mut found = false;

    for path in paths {
        let files: Vec<PathBuf> = get_directory_content(&path);

        for file in files {
            let filename = file.file_stem().unwrap();
            let executable = is_executable(&file.as_path()).expect("Failed to check execution permissions!");
            if filename == OsStr::new(command_name) && executable {
                found = true;

                let (name, args) = command.split_once(' ').unwrap();

                let args = get_command_args(args);
                
                Command::new(name)
                    .args(&args)
                    .spawn()
                    .expect("Command failed to start")
                    .wait_with_output()
                    .expect("Failed to wait on command");

                break;
            }
        }
        if found {
            break;
        }
    }

    if !found {
        println!("{}: command not found", command);
    }  
}

fn repl_loop() {
    loop {
        // Display prompt.
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();
        
        // Wait for user input.
        let command = read_command();

        // Exit command.
        if command == String::from(EXIT_CMD) {
            break;
        }

        // Echo command.
        if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
            handle_echo_command(&command);
            continue;
        }

        // Type command.
        if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
            handle_type_command(&command);
            continue;
        }

        // Pwd command.
        if command == String::from(PWD_CMD) {
            print_pwd();
            continue;
        }

        // Cd command.
        if command.starts_with(&*format!("{} ", &CD_CMD)) {
            handle_cd_command(&command);
            continue;
        }

        // Execute command.
        exec_command(&command);
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}
