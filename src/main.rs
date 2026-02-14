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
    if !arguments.contains("'") {
        arguments = arguments.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    let arguments = &arguments.replace("'", "");
    println!("{}", arguments);
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

                let mut command_split = command.split_whitespace();
                let name = command_split.next().unwrap_or("");

                let mut args: Vec<String> = if name == "cat" {
                    command
                        .split_once(' ')
                        .map(|(_, after)| vec![after.to_string()])
                        .unwrap_or_default()
                } else {
                    command_split.map(|s| s.to_string()).collect()
                };

                // Remove single quotes
                for arg in &mut args {
                    if arg.contains('\'') {
                        *arg = arg.replace('\'', "");
                    }
                }

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

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}
