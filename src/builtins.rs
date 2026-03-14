#[allow(unused_imports)]
use std::io::{self, Write, Error};

use std::fs;
use std::env;
use std::os::unix::process::ExitStatusExt;
use std::path::PathBuf;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::ffi::OsStr;
use std::process::Output;

use crate::constants;

pub fn is_builtin(cmd: &str) -> bool {
    constants::SHELL_BUILTINS.contains(&cmd)
}

pub fn execute_builtin(command: &str, history: &mut Vec<String>) -> Output {
    if command.trim() == constants::EXIT_CMD {
        Output { 
            status: ExitStatusExt::from_raw(1), 
            stdout: vec![], 
            stderr: vec![] 
        }
    } else if command.starts_with(&*format!("{} ", &constants::ECHO_CMD)) {
        handle_echo_command(&command)
    } else if command.starts_with(&*format!("{} ", &constants::TYPE_CMD)) {
        handle_type_command(&command)
    } else if command == String::from(constants::PWD_CMD) {
        print_pwd()
    } else if command.starts_with(&*format!("{} ", &constants::CD_CMD)) {
        handle_cd_command(&command)
    } else if command.starts_with(constants::HISTORY_CMD) {
        handle_history_command(command, history)
    } else {
        Output { 
            status: ExitStatusExt::from_raw(0), 
            stdout: vec![], 
            stderr: vec![]
        }
    }
}

fn handle_history_command(command: &str, history: &mut Vec<String>) -> Output {
    let stdout: Vec<u8>;
    let n = command.split_whitespace().nth(1);
    match n {
        Some(i) => {
            match i.parse::<usize>() {
                Ok(c) => {
                    stdout = history[(history.len() - c)..].join("\n").into_bytes();
                },
                Err(_) => {
                    if i == "-r" {
                        // Read history from file.
                        let path = command.split_whitespace().nth(2).unwrap();
                        let content = fs::read_to_string(path).unwrap().trim().to_string();
                        let mut splitted_content: Vec<String> = content.split("\n")
                                                                    //    .map(|s| s.to_string())
                                                                       .enumerate()
                                                                       .map(|(j, s)| format!("\t{}  {}", j + 1, s))
                                                                       .collect();
                        history.append(&mut splitted_content);
                    }
                    // stdout = history.join("\n").into_bytes();
                    stdout = vec![];
                }
            }
        },
        None => {
            stdout = history.join("\n").into_bytes();
        }
    }
    Output { 
        status: ExitStatusExt::from_raw(0), 
        stdout: stdout,
        stderr: vec![] 
    }
}

fn dir_exists(dir: &str) -> bool {
    let dir_path = Path::new(dir);
    return dir_path.exists()
}

fn is_executable(path: &std::path::Path) -> std::io::Result<bool> {
    let metadata = fs::metadata(path)?;
    let mode = metadata.permissions().mode();
    Ok(mode & 0o111 != 0)
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

pub fn print_pwd() -> Output {
    match env::current_dir() {
        Ok(output) => {
            return Output { 
                status: ExitStatusExt::from_raw(0), 
                stdout: Vec::from(output.into_os_string().into_string().unwrap().as_bytes()), 
                stderr: vec![] 
            };
        }
        Err(error) => {
            return Output {
                status: ExitStatusExt::from_raw(1), 
                stdout: vec![], 
                stderr: Vec::from(error.to_string().as_bytes()) 
            };
        }
    }
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
        } else if c =='\\' && !in_single_quotes {
            escaped = true;
        } else if c != ' ' || in_double_quotes || in_single_quotes {
            result.push(c);
        } else if c == ' ' && !result.ends_with(" ") {
            result.push(c);
        }
    }

    result
}

pub fn handle_echo_command(command: &str) -> Output {
    let arguments = &command[(constants::ECHO_CMD.len() + 1)..];
    let mut arguments = String::from(arguments);
    arguments = arguments.replace("\"\"", "");
    arguments = arguments.replace("''", "");
    let arguments = parse_echo_args(&arguments);
    let mut stdout = Vec::from(arguments.as_bytes());
    stdout.push(b'\n');
    Output { 
        status: ExitStatusExt::from_raw(0), 
        stdout: stdout, 
        stderr: vec![]
    }
}

fn check_type(command: &str) -> Output {
    let path_var = env::var_os("PATH").expect("PATH variable not set!");
    let paths: Vec<PathBuf> = env::split_paths(&path_var).collect();

    for path in paths {
        let files: Vec<PathBuf> = get_directory_content(&path);
        for file in files {
            let filename = file.file_stem();
            let executable = is_executable(&file.as_path()).expect("Failed to check execution permissions!");
            if filename == Some(OsStr::new(command)) && executable {
                return Output { 
                    status: ExitStatusExt::from_raw(0), 
                    stdout: Vec::from(format!("{} is {}", command, file.to_str().unwrap()).as_bytes()),
                    stderr: vec![] 
                };
            }
        }
    }

    Output{ 
        status: ExitStatusExt::from_raw(0), 
        stdout: vec![], 
        stderr: Vec::from(format!("{}: not found", command).as_bytes()) 
    }
}

pub fn handle_type_command(command: &str) -> Output {
    let arguments = &command[(constants::TYPE_CMD.len() + 1)..];
    if constants::SHELL_BUILTINS.contains(&arguments) {
        Output { 
            status: ExitStatusExt::from_raw(0), 
            stdout: Vec::from(format!("{} is a shell builtin", arguments).as_bytes()), 
            stderr: vec![] 
        }
    } else {
        check_type(arguments)
    }
}

fn change_dir(dir: &str) -> Result<(), Error>{
    std::env::set_current_dir(dir)
}

pub fn handle_cd_command(command: &str) -> Output {
    let arguments = &command[(constants::CD_CMD.len() + 1)..];
    let dir = arguments.split_whitespace().next().unwrap();

    if dir == constants::HOME_DIR {
        let home_dir = env::var_os("HOME").expect("HOME variable not set!");
        let home_dir = home_dir.to_str().unwrap();
        match change_dir(home_dir) {
            Ok(_) => { 
                return Output { 
                    status: ExitStatusExt::from_raw(0), 
                    stdout: vec![], 
                    stderr: vec![] 
                }; 
            }
            Err(e) => { 
                return Output { 
                    status: ExitStatusExt::from_raw(0), 
                    stdout: vec![], 
                    stderr: Vec::from(e.to_string().as_bytes()) 
                }; 
            }
        }
    }

    if dir_exists(dir) {
        match change_dir(dir) {
            Ok(_) => { 
                return Output {
                    status: ExitStatusExt::from_raw(0), 
                    stdout: vec![], 
                    stderr: vec![] 
                }; 
            }
            Err(e) => { 
                return Output { 
                    status: ExitStatusExt::from_raw(1), 
                    stdout: vec![], 
                    stderr: Vec::from(e.to_string().as_bytes()) 
                };
            }
        }
    } else {
        return Output { 
            status: ExitStatusExt::from_raw(0), 
            stdout: vec![], 
            stderr: Vec::from(format!("cd: {}: No such file or directory", dir).as_bytes()) 
        };
    }
}