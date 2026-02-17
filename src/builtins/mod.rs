mod cd;
mod echo;
mod exit;
mod pwd;
mod type_cmd;

use crate::parser::Command;

/// All registered shell builtins.
pub const BUILTINS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

pub fn is_builtin(name: &str) -> bool {
    BUILTINS.contains(&name)
}

/// Execute a builtin. Returns `true` if the shell should exit.
pub fn execute(cmd: &Command) -> bool {
    match cmd.name.as_str() {
        "exit" => {
            exit::run(cmd);
            true
        }
        "echo" => {
            echo::run(cmd);
            false
        }
        "type" => {
            type_cmd::run(cmd);
            false
        }
        "pwd" => {
            pwd::run();
            false
        }
        "cd" => {
            cd::run(cmd);
            false
        }
        _ => false,
    }
}