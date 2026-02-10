#[allow(unused_imports)]
use std::io::{self, Write};

use std::fs;
use std::env;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::ffi::OsStr;

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";
const PROMPT: &str = "$ ";
const PWD_CMD: &str = "pwd";

const SHELL_BUILTINS: &[&str] = &[EXIT_CMD, ECHO_CMD, TYPE_CMD, PWD_CMD];

fn print_pwd() {
    println!("{}", env::current_dir().unwrap().to_str());
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

        // Execute command.
        exec_command(&command);
    }
}

fn check_type(command: &str) {
    let path_var = env::var_os("PATH").expect("PATH variable not set!");
    let paths: Vec<PathBuf> = env::split_paths(&path_var).collect();

    let mut found = false;

    for path in paths {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();
        entries.sort();

        for entry in entries {
            let filename = entry.file_stem();
            if filename == Some(OsStr::new(command)) && is_executable(&entry.as_path()).expect("Failed to check execution permissions!") {
                println!("{} is {:?}", command, entry);
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

fn exec_command(command: &str) {

    let path_var = env::var_os("PATH").expect("PATH variable not set!");
    let paths: Vec<PathBuf> = env::split_paths(&path_var).collect();

    let mut found = false;

    for path in paths {
        let mut entries: Vec<PathBuf> = fs::read_dir(path)
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();
        entries.sort();

        for entry in entries {
            let filename = entry.file_stem().unwrap();
            if filename == OsStr::new(filename) && is_executable(&entry.as_path()).expect("Failed to check execution permissions!") {
                found = true;
                let mut command_split = command.split_whitespace();
                let command_name = command_split.next().unwrap_or("");
                let command_args: Vec<_> = command_split.collect();
                Command::new(command_name)
                    .args(command_args)
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
        println!("{}: not found", command);
    }  
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}
