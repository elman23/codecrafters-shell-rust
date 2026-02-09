#[allow(unused_imports)]
use std::io::{self, Write};

use std::fs;
use std::env;
use std::path::PathBuf;
use std::os::unix::fs::PermissionsExt;

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";

const SHELL_BUILTINS: &[&str] = &[EXIT_CMD, ECHO_CMD, TYPE_CMD];

fn is_executable(path: &str) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        let permissions = metadata.permissions();
        // On Unix-like systems, check if the owner's executable bit is set
        permissions.mode() & 0o001 != 0
    } else {
        false
    }
}

fn repl_loop() {
    loop {
        // Display prompt.
        print!("$ ");
        io::stdout().flush().unwrap();
        
        // Wait for user input.
        let mut command: String = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = String::from(command.trim());

        // Exit command.
        if command == String::from(EXIT_CMD) {
            break;
        }

        // Echo command.
        if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
            let arguments = &command[(ECHO_CMD.len() + 1)..];
            println!("{}", arguments);
            continue;
        }

        if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
            let arguments = &command[(TYPE_CMD.len() + 1)..];
            if SHELL_BUILTINS.contains(&arguments) {
                println!("{} is a shell builtin", arguments);
            } else {
                check_type(arguments);
            }
            continue;
        }

        println!("{}: command not found", command);
    }
}

fn check_type(command: &str) {
    match env::var("PATH") {
        Ok(path) => {
            //println!("PATH: {}", path);
            let dirs = path.split(":");
            let mut found = false;
            for dir in dirs {
                let entries: Vec<PathBuf> = fs::read_dir(dir)
                    .unwrap()
                    .map(|res| res.map(|e| e.path()))
                    .collect::<Result<Vec<_>, io::Error>>()
                    .unwrap();

                let mut entries: Vec<PathBuf> = entries;
                entries.sort();

                for entry in entries {
                    let path_as_string = entry.to_string_lossy();
                    let filename = entry
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("");
                    if filename.split(".").next() == Some(command) && is_executable(&path_as_string) {
                        println!("{} is {}", command, path_as_string);
                        found = true;
                    }
                }
            }
            if !found {
                println!("{}: not found", command);
            }                
        },
        Err(e) => {
            println!("Couldn't read PATH: {}", e);
        },
    }
}

fn main() -> io::Result<()> {
    // Shell's infinite REPL loop.
    repl_loop();

    Ok(())
}
