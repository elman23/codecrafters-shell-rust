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

const SHELL_BUILTINS: &[&str] = &[EXIT_CMD, ECHO_CMD, TYPE_CMD];

// fn is_executable(path: &str) -> bool {
//     if let Ok(metadata) = fs::metadata(path) {
//         let permissions = metadata.permissions();
//         // On Unix-like systems, check if the owner's executable bit is set
//         permissions.mode() & 0o001 != 0
//     } else {
//         false
//     }
// }

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

    // match env::var("PATH") {
    //     Ok(path) => {
    //         let dirs = path.split(":"); // TODO: Use path separation.
    //         let mut found = false;
    //         for dir in dirs {
    //             let entries: Vec<PathBuf> = fs::read_dir(dir)
    //                 .unwrap()
    //                 .map(|res| res.map(|e| e.path()))
    //                 .collect::<Result<Vec<_>, io::Error>>()
    //                 .unwrap();

    //             let mut entries: Vec<PathBuf> = entries;
    //             entries.sort();

    //             for entry in entries {
    //                 let path_as_string = entry.to_string_lossy();
    //                 let filename = entry
    //                     .file_name()
    //                     .and_then(|s| s.to_str())
    //                     .unwrap_or("");
    //                 if filename.split(".").next() == Some(command) && is_executable(&path_as_string) {
    //                     println!("{} is {}", command, path_as_string);
    //                     found = true;
    //                     break;
    //                 }
    //             }
    //             if found {
    //                 break;
    //             }
    //         }
    //         if !found {
    //             println!("{}: not found", command);
    //         }                
    //     },
    //     Err(e) => {
    //         println!("Couldn't read PATH: {}", e);
    //     },
    // }
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
            println!("Entry: {:?}", entry);
            let filename = entry.file_stem().unwrap();
            if filename == OsStr::new(filename) && is_executable(&entry.as_path()).expect("Failed to check execution permissions!") {
                found = true;
                let mut command_split = command.split_whitespace();
                let command_name = command_split.next().unwrap_or("");
                let command_args: Vec<_> = command_split.collect();
                println!("Command: {:?}", command_name);
                println!("Args: {:?}", command_args);
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

    // match env::var("PATH") {
    //     Ok(path) => {
    //         let dirs = path.split(":");
    //         let mut found = false;
    //         for dir in dirs {
    //             let entries: Vec<PathBuf> = fs::read_dir(dir)
    //                 .unwrap()
    //                 .map(|res| res.map(|e| e.path()))
    //                 .collect::<Result<Vec<_>, io::Error>>()
    //                 .unwrap();

    //             let mut entries: Vec<PathBuf> = entries;
    //             entries.sort();

    //             for entry in entries {
    //                 let path_as_string = entry.to_string_lossy();
    //                 let filename = entry
    //                     .file_name()
    //                     .and_then(|s| s.to_str())
    //                     .unwrap_or("");
    //                 let executable = command.split_whitespace().next().expect("");
    //                 if filename.split(".").next() == Some(executable) && is_executable(&path_as_string) {
    //                     found = true;
    //                     let mut command_split = command.split_whitespace();
    //                     let command_name = command_split.next().unwrap_or("");
    //                     let command_args: Vec<_> = command_split.collect();
    //                     Command::new(command_name)
    //                         .args(command_args)
    //                         .spawn()
    //                         .expect("Command failed to start")
    //                         .wait_with_output()
    //                         .expect("Failed to wait on command");
    //                     break;
    //                 }
    //             }
    //             if found {
    //                 break;
    //             }
    //         }
    //         if !found {
    //             println!("{}: not found", command);
    //         }                
    //     },
    //     Err(e) => {
    //         println!("Couldn't read PATH: {}", e);
    //     },
    // }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}
