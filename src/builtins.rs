use std::io::{self, Error};
use std::fs;
use std::env;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use std::ffi::OsStr;
use std::process::Output;
use std::sync::{Arc, Mutex};

use crate::constants;
use crate::jobs::Jobs;
use crate::jobs;
use crate::utils;
use crate::complete::Complete;

// ── Helpers ───────────────────────────────────────────────────────────────────

pub fn is_builtin(cmd: &str) -> bool {
    constants::SHELL_BUILTINS.contains(&cmd)
}

fn make_output(status: i32, stdout: Vec<u8>, stderr: Vec<u8>) -> Output {
    Output {
        status: ExitStatusExt::from_raw(status),
        stdout,
        stderr,
    }
}

fn ok_output(stdout: Vec<u8>) -> Output {
    make_output(0, stdout, vec![])
}

fn err_output(stderr: Vec<u8>) -> Output {
    make_output(1, vec![], stderr)
}

fn empty_ok() -> Output {
    make_output(0, vec![], vec![])
}

// ── Dispatch ──────────────────────────────────────────────────────────────────

pub fn execute_builtin(
    command: &str,
    history: &mut Vec<String>,
    jobs: &mut Jobs,
    complete: &Arc<Mutex<Complete>>,
) -> Output {
    // Strip leading/trailing whitespace once for all comparisons
    let trimmed = command.trim();

    if trimmed == constants::EXIT_CMD {
        make_output(1, vec![], vec![])
    } else if trimmed == constants::PWD_CMD {
        // Exact match — no allocation needed
        print_pwd()
    } else if trimmed == constants::ECHO_CMD {
        // Bare `echo` with no arguments
        ok_output(b"\n".to_vec())
    } else if trimmed.starts_with(&format!("{} ", constants::ECHO_CMD)) {
        handle_echo_command(trimmed)
    } else if trimmed.starts_with(&format!("{} ", constants::TYPE_CMD)) {
        handle_type_command(trimmed)
    } else if trimmed.starts_with(&format!("{} ", constants::CD_CMD)) {
        handle_cd_command(trimmed)
    } else if trimmed.starts_with(constants::HISTORY_CMD) {
        handle_history_command(trimmed, history)
    } else if trimmed.starts_with(constants::JOBS_CMD) {
        handle_jobs_command(jobs)
    } else if trimmed.starts_with(constants::COMPLETE_CMD) {
        handle_complete_command(trimmed, complete)
    } else if trimmed.starts_with(constants::DECLARE_CMD) {
        handle_declare_command(trimmed)
    } else {
        empty_ok()
    }
}

// ── history ───────────────────────────────────────────────────────────────────

fn handle_history_command(command: &str, history: &mut Vec<String>) -> Output {
    let mut parts = command.split_whitespace();
    parts.next(); // consume "history"

    let stdout = match parts.next() {
        None => history.join("\n").into_bytes(),

        Some(n) => match n.parse::<usize>() {
            Ok(count) => {
                let start = history.len().saturating_sub(count);
                history[start..].join("\n").into_bytes()
            }

            Err(_) => {
                // Subcommands that need a path argument
                let path = match parts.next() {
                    Some(p) => p,
                    None => {
                        return make_output(
                            1,
                            vec![],
                            format!("history {}: missing file operand\n", n).into_bytes(),
                        );
                    }
                };

                match n {
                    "-r" => {
                        // Read history from file
                        match fs::read_to_string(path) {
                            Ok(content) => {
                                let offset = history.len();
                                let mut lines: Vec<String> = content
                                    .trim()
                                    .split('\n')
                                    .enumerate()
                                    .map(|(i, s)| format!("\t{}  {}", offset + i + 1, s))
                                    .collect();
                                history.append(&mut lines);
                            }
                            Err(e) => {
                                return make_output(
                                    1,
                                    vec![],
                                    format!("history -r: {}\n", e).into_bytes(),
                                );
                            }
                        }
                    }

                    "-w" => {
                        // Write (overwrite) history to file
                        let content = history_to_file_content(history);
                        let _ = utils::write_file(path, &content);
                    }

                    "-a" => {
                        // Append new history to file
                        let content = history_to_file_content(history);
                        let existing = utils::read_file_content(path);

                        // Find where the previous `history -a` invocation ended so
                        // we don't double-append older entries.
                        let pattern = format!("history -a {}", path);
                        let base = match existing.find(&pattern) {
                            Some(idx) => &existing[..idx + pattern.len() + 1],
                            None      => &existing,
                        };

                        let combined = format!("{}{}", base, content);
                        let _ = utils::overwrite_file(path, &combined);
                        *history = vec![];
                    }

                    other => {
                        return make_output(
                            1,
                            vec![],
                            format!("history: {}: invalid option\n", other).into_bytes(),
                        );
                    }
                }

                vec![]
            }
        },
    };

    ok_output(stdout)
}

/// Strip formatting prefix from history entries and join into file content.
fn history_to_file_content(history: &[String]) -> String {
    let mut content: String = history
        .iter()
        .map(|s| s.trim().split_once(' ').unwrap_or(("", "")).1.trim())
        .collect::<Vec<_>>()
        .join("\n");
    content.push('\n');
    content
}

// ── echo ──────────────────────────────────────────────────────────────────────

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
        match c {
            '"' if !in_single_quotes => in_double_quotes = !in_double_quotes,
            '\'' if !in_double_quotes => in_single_quotes = !in_single_quotes,
            '\\' if !in_single_quotes => escaped = true,
            ' ' if !in_double_quotes && !in_single_quotes => {
                // Collapse consecutive spaces
                if !result.ends_with(' ') {
                    result.push(' ');
                }
            }
            _ => result.push(c),
        }
    }

    result
}

pub fn handle_echo_command(command: &str) -> Output {
    let arguments = &command[(constants::ECHO_CMD.len() + 1)..];
    // Strip empty quote pairs before parsing
    let arguments = arguments.replace("\"\"", "").replace("''", "");
    let mut stdout = parse_echo_args(&arguments).into_bytes();
    stdout.push(b'\n');
    ok_output(stdout)
}

// ── complete ──────────────────────────────────────────────────────────────────

pub fn handle_complete_command(command: &str, complete: &Arc<Mutex<Complete>>) -> Output {
    let mut split = command.split_whitespace();
    split.next(); // consume "complete"

    let flag       = split.next().unwrap_or("");
    let argument   = split.next().unwrap_or("");
    let executable = split.next().unwrap_or("");

    let mut complete = complete.lock().unwrap();

    match flag {
        "-C" => {
            // Register a completion script for `executable`
            // e.g. `complete -C my_script git`
            complete.scripts.insert(executable.to_string(), argument.to_string());
            empty_ok()
        }
        "-p" => {
            // Print the registered completion for `argument`
            let line = match complete.scripts.get(argument) {
                Some(script) if !script.is_empty() => {
                    format!("complete -C '{}' {}\n", script, argument)
                }
                _ => {
                    format!("complete: {}: no completion specification\n", argument)
                }
            };
            // Return via stdout so the caller can pipe/redirect it
            ok_output(line.into_bytes())
        }
        "-r" => {
            complete.scripts.remove(argument);
            empty_ok()
        }
        _ => {
            let msg = format!("complete: {}: invalid option\n", flag);
            make_output(1, vec![], msg.into_bytes())
        }
    }
}

// ── declare ──────────────────────────────────────────────────────────────────

pub fn handle_declare_command(command: &str) -> Output {
    let mut split = command.split_whitespace();
    split.next(); // consume "declare"

    let flag       = split.next().unwrap_or("");
    let argument   = split.next().unwrap_or("");

    match flag {
        _ => {
            let msg = format!("declare: {}: not found\n", argument);
            make_output(1, vec![], msg.into_bytes())
        }
    }
}

// ── jobs ──────────────────────────────────────────────────────────────────────

pub fn handle_jobs_command(jobs: &mut Jobs) -> Output {
    let mut keys: Vec<_> = jobs.jobs_list.keys().copied().collect();
    keys.sort();
    let total = keys.len();
    let mut lines = String::new();

    for (i, k) in keys.iter().enumerate() {
        let v   = jobs.jobs_list.get(k).unwrap();
        let pid = *jobs.process_list.get(k).unwrap();

        let is_running = utils::is_process_running(pid);
        let job_state  = if is_running { "Running" } else { "Done" };

        let marker = if i == total - 1 {
            '+'
        } else if i + 1 == total.saturating_sub(1) {
            '-'
        } else {
            ' '
        };

        let display_cmd = if is_running {
            v.clone()
        } else {
            v.replace(" &", "")
        };

        lines.push_str(&format!("[{}]{}  {:<8} {}\n", k, marker, job_state, display_cmd));
    }

    jobs::reap_jobs(jobs, false);

    // Return via stdout so the caller can pipe/redirect it
    ok_output(lines.into_bytes())
}

// ── pwd ───────────────────────────────────────────────────────────────────────

pub fn print_pwd() -> Output {
    match env::current_dir() {
        Ok(dir) => ok_output(dir.into_os_string().into_string().unwrap().into_bytes()),
        Err(e)  => err_output(e.to_string().into_bytes()),
    }
}

// ── type ──────────────────────────────────────────────────────────────────────

fn is_executable(path: &Path) -> io::Result<bool> {
    let mode = fs::metadata(path)?.permissions().mode();
    Ok(mode & 0o111 != 0)
}

fn get_directory_content(path: &PathBuf) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(path)
        .unwrap()
        .filter_map(|res| res.ok().map(|e| e.path()))
        .collect();
    files.sort();
    files
}

fn check_type(command: &str) -> Output {
    let path_var = env::var_os("PATH").expect("PATH variable not set!");

    for path in env::split_paths(&path_var) {
        for file in get_directory_content(&path) {
            // Use file_name() (not file_stem()) to match the exact binary name,
            // including any extension (e.g. "python3", "node").
            let filename   = file.file_name();
            let executable = is_executable(&file).unwrap_or(false);
            if filename == Some(OsStr::new(command)) && executable {
                let msg = format!("{} is {}", command, file.display());
                return ok_output(msg.into_bytes());
            }
        }
    }

    let msg = format!("{}: not found", command);
    make_output(0, vec![], msg.into_bytes())
}

pub fn handle_type_command(command: &str) -> Output {
    let arguments = command[(constants::TYPE_CMD.len() + 1)..].trim();
    if constants::SHELL_BUILTINS.contains(&arguments) {
        ok_output(format!("{} is a shell builtin", arguments).into_bytes())
    } else {
        check_type(arguments)
    }
}

// ── cd ────────────────────────────────────────────────────────────────────────

fn dir_exists(dir: &str) -> bool {
    Path::new(dir).exists()
}

fn change_dir(dir: &str) -> Result<(), Error> {
    env::set_current_dir(dir)
}

pub fn handle_cd_command(command: &str) -> Output {
    let arguments = command[(constants::CD_CMD.len() + 1)..].trim();
    let dir = match arguments.split_whitespace().next() {
        Some(d) => d,
        None => {
            return make_output(1, vec![], b"cd: missing argument\n".to_vec());
        }
    };

    let target: String = if dir == constants::HOME_DIR {
        match env::var("HOME") {
            Ok(h) => h,
            Err(_) => {
                return make_output(1, vec![], b"cd: HOME not set\n".to_vec());
            }
        }
    } else if dir_exists(dir) {
        dir.to_string()
    } else {
        return make_output(
            1,
            vec![],
            format!("cd: {}: No such file or directory\n", dir).into_bytes(),
        );
    };

    match change_dir(&target) {
        Ok(_)  => empty_ok(),
        Err(e) => make_output(1, vec![], e.to_string().into_bytes()),
    }
}