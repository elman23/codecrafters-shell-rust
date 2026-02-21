#[allow(unused_imports)]
use std::io::{self, Write, Error};

use std::fs;
use std::env;
use std::path::PathBuf;
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::ffi::OsStr;

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";
const PWD_CMD: &str = "pwd";
const CD_CMD: &str = "cd";

// TODO: Improve. This requires that each new built-in command shall be added manually.
const SHELL_BUILTINS: &[&str] = &[EXIT_CMD, ECHO_CMD, TYPE_CMD, PWD_CMD, CD_CMD];

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

pub fn print_pwd() -> Result<String, String> {
    // println!("{}", env::current_dir().unwrap().to_str().unwrap());
    match env::current_dir() {
        Ok(output) => {
            return Ok(output.into_os_string().into_string().unwrap())
        }
        Err(error) => {
            return Err(error.to_string())
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

pub fn handle_echo_command(command: &str) -> Result<String, String> {
    let arguments = &command[(ECHO_CMD.len() + 1)..];
    let mut arguments = String::from(arguments);
    arguments = arguments.replace("\"\"", "");
    arguments = arguments.replace("''", "");
    let arguments = parse_echo_args(&arguments);
    // println!("{}", arguments);
    Ok(arguments)
}

fn check_type(command: &str) -> Result<String, String> {
    let path_var = env::var_os("PATH").expect("PATH variable not set!");
    let paths: Vec<PathBuf> = env::split_paths(&path_var).collect();

    for path in paths {
        let files: Vec<PathBuf> = get_directory_content(&path);
        for file in files {
            let filename = file.file_stem();
            let executable = is_executable(&file.as_path()).expect("Failed to check execution permissions!");
            if filename == Some(OsStr::new(command)) && executable {
                // println!("{} is {}", command, file.to_str().unwrap());
                // found = true;
                // break;
                return Ok(format!("{} is {}", command, file.to_str().unwrap()));
            }
        }
        // if found {
            // break;
        // }
    }

    // if !found {
        // println!("{}: not found", command);
    // }
    Err(format!("{}: not found", command))
}

pub fn handle_type_command(command: &str) -> Result<String, String> {
    let arguments = &command[(TYPE_CMD.len() + 1)..];
    if SHELL_BUILTINS.contains(&arguments) {
        Ok(format!("{} is a shell builtin", arguments))
    } else {
        check_type(arguments)
    }
}

fn change_dir(dir: &str) -> Result<(), Error>{
    std::env::set_current_dir(dir)
}

pub fn handle_cd_command(command: &str) -> Result<(), Error> {
    let arguments = &command[(CD_CMD.len() + 1)..];
    let dir = arguments.split_whitespace().next().unwrap();

    // The "~" special case.
    if dir == "~" {
        let home_dir = env::var_os("HOME").expect("HOME variable not set!");
        let home_dir = home_dir.to_str().unwrap();
        return change_dir(home_dir);
    }

    if dir_exists(dir) {
        return change_dir(dir);
    } else {
        // println!("cd: {}: No such file or directory", dir)
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("cd: {}: No such file or directory", dir),
        ));
    }
}