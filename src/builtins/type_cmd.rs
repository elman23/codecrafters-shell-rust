use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::builtins;
use crate::parser::Command;

pub fn run(cmd: &Command) {
    for arg in &cmd.args {
        identify(arg);
    }
}

fn identify(name: &str) {
    if builtins::is_builtin(name) {
        println!("{} is a shell builtin", name);
        return;
    }

    match find_in_path(name) {
        Some(path) => println!("{} is {}", name, path.display()),
        None => println!("{}: not found", name),
    }
}

pub fn find_in_path(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;

    for dir in env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }

    None
}

fn is_executable(path: &std::path::Path) -> bool {
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}