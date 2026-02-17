use std::io::{self, Write};
use std::process;

use crate::builtins::type_cmd::find_in_path;
use crate::parser::Command;

/// Execute an external command found on PATH (or by absolute/relative path).
pub fn run(cmd: &Command) {
    // Prefer explicit path; fall back to PATH lookup.
    let program = if cmd.name.contains('/') {
        Some(std::path::PathBuf::from(&cmd.name))
    } else {
        find_in_path(&cmd.name)
    };

    let program = match program {
        Some(p) => p,
        None => {
            eprintln!("{}: command not found", cmd.name);
            return;
        }
    };

    match process::Command::new(&program)
        .args(&cmd.args)
        .spawn()
    {
        Ok(child) => match child.wait_with_output() {
            Ok(output) => {
                print!("{}", String::from_utf8_lossy(&output.stdout));
                io::stdout().flush().unwrap();
            }
            Err(e) => eprintln!("{}: {}", cmd.name, e),
        },
        Err(e) => eprintln!("{}: {}", cmd.name, e),
    }
}