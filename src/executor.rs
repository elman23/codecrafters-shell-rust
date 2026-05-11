use crate::builtins;
use crate::complete::Complete;
use crate::constants;
use crate::jobs::Jobs;
use crate::utils;
use std::collections::HashMap;
use std::env::var;
use std::fmt::Arguments;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Child, ChildStdout, Command, ExitStatus, Output, Stdio};
use std::sync::{Arc, Mutex};

// ── String helpers ────────────────────────────────────────────────────────────

fn clean_last_newline(s: &str) -> &str {
    s.strip_suffix('\n').unwrap_or(s)
}

fn print_cleaned(s: &str) {
    if !s.is_empty() {
        let _ = writeln!(std::io::stdout(), "{}", clean_last_newline(s));
    }
}

// ── Argument / quoting helpers ────────────────────────────────────────────────

/// Split `input` on `ch` (a quote character), collecting both quoted segments
/// and the unquoted text that surrounds them into a single flat token list.
fn split_char(ch: char, input: &str) -> Vec<String> {
    let double_quotes = ch == '"';

    let mut result = Vec::new();
    let mut in_quotes = false;
    let mut escaped = false;
    let mut current = String::new();

    for c in input.chars() {
        if escaped {
            current.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' && (double_quotes || !in_quotes) {
            escaped = true;
            continue;
        }
        if c == ch {
            // Toggling in/out of a quoted region — flush the accumulated token
            // only when leaving a quoted section so that adjacent unquoted text
            // merges correctly with what came before.
            in_quotes = !in_quotes;
            if !in_quotes && !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
        } else if !in_quotes && c == ' ' {
            // Word boundary outside quotes
            if !current.is_empty() {
                result.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

fn get_command_args(args: &str, variables: &HashMap<String, String>) -> Vec<String> {
    let mut handle_slashes = false;
    let mut args: Vec<String> = if args.contains('"') {
        split_char('"', args)
    } else if args.contains('\'') {
        split_char('\'', args)
    } else {
        handle_slashes = true;
        args.split_whitespace().map(|s| s.to_string()).collect()
    };

    for arg in &mut args {
        if handle_slashes {
            if arg.contains("\\\\") {
                *arg = arg.replace("\\\\", "\\");
            } else {
                *arg = arg.replace('\\', "");
            }
        }
        *arg = arg.trim().to_string();
    }

    // args
    utils::expand_args(args, variables)
}

fn cleanup_name(name: &str) -> String {
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;
    let mut cleaned = String::new();

    for c in name.chars() {
        if escaped {
            cleaned.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' && in_double_quote {
            escaped = true;
            continue;
        }
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
        } else if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
        } else {
            cleaned.push(c);
        }
    }

    cleaned
}

fn get_command_path(s: &str) -> String {
    let mut command_path = String::new();
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    for c in s.chars() {
        if c == ' ' && !in_double_quote && !in_single_quote {
            break;
        }
        if c == '"' {
            in_double_quote = !in_double_quote;
        }
        if c == '\'' {
            in_single_quote = !in_single_quote;
        }
        command_path.push(c);
    }

    command_path
}

// ── Redirect helpers ──────────────────────────────────────────────────────────

fn write_to_file(path: &str, content: &[u8], append: bool) -> io::Result<()> {
    let content_str = String::from_utf8_lossy(content);
    let cleaned = clean_last_newline(&content_str);
    if cleaned.is_empty() {
        return Ok(());
    }
    let mut file = if append {
        OpenOptions::new().append(true).create(true).open(path)?
    } else {
        File::create(path)?
    };
    writeln!(file, "{}", cleaned)
}

// ── Public entry points ───────────────────────────────────────────────────────

pub fn execute(
    mut command: String,
    history: &mut Vec<String>,
    jobs: &mut Jobs,
    complete: &Arc<Mutex<Complete>>,
    variables: &mut HashMap<String, String>,
) -> std::io::Result<u8> {
    // ── Background job ────────────────────────────────────────────────────────
    let trimmed = command.trim();
    if trimmed.ends_with(" &") {
        let bare = trimmed[..trimmed.len() - 2].trim().to_string();
        let job_number = jobs.jobs_list.keys().max().copied().unwrap_or(0) + 1;
        jobs.jobs_list.insert(job_number, bare.clone());
        let pid = run_command_background(&bare, variables);
        jobs.process_list.insert(job_number, pid);
        println!("[{}] {}", job_number, pid);
        return Ok(0);
    }

    // ── Redirect detection ────────────────────────────────────────────────────
    let redirect_info = utils::get_redirect(&command);
    let redirect_stdout = redirect_info.redirect_stdout_file;
    let redirect_stderr = redirect_info.redirect_stderr_file;
    let append_stdout = redirect_info.append_stdout;
    let append_stderr = redirect_info.append_stderr;

    if redirect_stdout.is_some() || redirect_stderr.is_some() {
        let index = redirect_info.file_index_start.unwrap() - 1;
        command = command[..index].trim().to_string();
    }

    // ── Execute ───────────────────────────────────────────────────────────────
    let result = match execute_piped(&command, history, jobs, complete, variables) {
        Ok(r) => r,
        Err(_) => {
            let _ = writeln!(std::io::stderr(), "{}: command not found", command);
            return Ok(0);
        }
    };

    if command.starts_with(constants::EXIT_CMD) {
        return Ok(1);
    }

    // ── Output routing ────────────────────────────────────────────────────────
    if let Some(ref path) = redirect_stdout {
        // Always create the file — real shells create the redirect target even
        // when the command produces no stdout (e.g. `ls nonexistent >> file`).
        // For append mode we must not truncate an existing file, so we open with
        // `.append(true)`; for overwrite mode we truncate via `File::create`.
        if !Path::new(path).exists() {
            if append_stdout {
                OpenOptions::new().create(true).append(true).open(path)?;
            } else {
                File::create(path)?;
            }
        }
        if !result.stdout.is_empty() {
            write_to_file(path, &result.stdout, append_stdout)?;
        }
        if !result.stderr.is_empty() {
            let _ = writeln!(
                std::io::stderr(),
                "{}",
                String::from_utf8_lossy(&result.stderr).trim()
            );
        }
    } else if let Some(ref path) = redirect_stderr {
        if !result.stdout.is_empty() {
            print_cleaned(&String::from_utf8_lossy(&result.stdout));
        }
        // Same guarantee: always create the file even when stderr is empty.
        if !Path::new(path).exists() {
            if append_stderr {
                OpenOptions::new().create(true).append(true).open(path)?;
            } else {
                File::create(path)?;
            }
        }
        if !result.stderr.is_empty() {
            write_to_file(path, &result.stderr, append_stderr)?;
        }
    } else {
        if !result.stdout.is_empty() {
            print_cleaned(&String::from_utf8_lossy(&result.stdout));
        }
        if !result.stderr.is_empty() {
            let _ = writeln!(
                std::io::stderr(),
                "{}",
                String::from_utf8_lossy(&result.stderr).trim()
            );
        }
    }

    Ok(0)
}

fn run_command_background(command: &str, variables: &HashMap<String, String>) -> u32 {
    // Collect ALL tokens, not just the first argument
    let mut parts = command.split_whitespace();
    let cmd = parts.next().unwrap_or("");
    // Remaining tokens are arguments — collect them all
    let args: Vec<&str> = parts.collect();

    // TODO: Expand arguments.
    let args = utils::expand_args(args, variables);

    Command::new(cmd)
        .args(&args)
        .spawn()
        .expect("Failed to start background process")
        .id()
}

pub fn execute_piped(
    input: &str,
    history: &mut Vec<String>,
    jobs: &mut Jobs,
    complete: &Arc<Mutex<Complete>>,
    variables: &mut HashMap<String, String>,
) -> io::Result<Output> {
    let cmds: Vec<&str> = input.split('|').map(|c| c.trim()).collect();

    let mut children: Vec<Child> = Vec::new();
    let mut previous_stdout: Option<ChildStdout> = None;
    let mut previous_ec: ExitStatus = ExitStatusExt::from_raw(0);
    let mut previous_out: Option<Vec<u8>> = None;
    let mut previous_err: Option<Vec<u8>> = None;
    let mut is_last_builtin = false;

    for (i, c) in cmds.iter().enumerate() {
        let first_word = c.split_whitespace().next().unwrap_or("");
        let is_last = i == cmds.len() - 1;

        if builtins::is_builtin(first_word) {
            let result: Output = builtins::execute_builtin(c, history, jobs, complete, variables);
            previous_ec = result.status;
            previous_out = if result.stdout.is_empty() {
                None
            } else {
                Some(result.stdout)
            };
            previous_err = if result.stderr.is_empty() {
                None
            } else {
                Some(result.stderr)
            };
            previous_stdout = None;
            if is_last {
                is_last_builtin = true;
            }
            continue;
        }

        // ── Build Command ─────────────────────────────────────────────────────
        let command_path = get_command_path(c);
        // Use only the last path component so that e.g. "/usr/bin/grep" works
        let cmd_name = Path::new(&command_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let cmd_name = cleanup_name(cmd_name);

        let mut cmd = Command::new(&cmd_name);
        cmd.args(get_command_args(&c[command_path.len()..], variables));

        // ── Stdin wiring ──────────────────────────────────────────────────────
        if let Some(stdin) = previous_stdout.take() {
            cmd.stdin(stdin);
        } else if previous_out.is_some() || previous_err.is_some() {
            cmd.stdin(Stdio::piped());
        }

        // ── Stdout/stderr capture ─────────────────────────────────────────────
        if !is_last || cmds.len() == 1 {
            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());
        }

        let mut child = cmd.spawn()?;

        // Feed previous builtin output into this process's stdin
        if let Some(s) = previous_out.take() {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&s)?;
            }
        } else if let Some(s) = previous_err.take() {
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(&s)?;
            }
        }

        if !is_last {
            previous_stdout = child.stdout.take();
        }

        children.push(child);
    }

    // ── Wait for children ─────────────────────────────────────────────────────
    let output: Option<Output> = if !children.is_empty() && !is_last_builtin {
        let last = children.pop().unwrap();
        let out = last.wait_with_output()?;
        for mut child in children {
            child.wait().unwrap();
        }
        Some(out)
    } else {
        for mut child in children {
            child.wait().unwrap();
        }
        None
    };

    if previous_out.is_none() && previous_err.is_none() {
        Ok(output.unwrap_or(Output {
            status: previous_ec,
            stdout: vec![],
            stderr: vec![],
        }))
    } else {
        Ok(Output {
            status: previous_ec,
            stdout: previous_out.unwrap_or_default(),
            stderr: previous_err.unwrap_or_default(),
        })
    }
}

/// Run a script and return its combined output.
pub fn execute_script(
    script: &str,
    args: Vec<&str>,
    variables: &HashMap<String, String>,
) -> io::Result<Output> {
    // TODO: Expand arguments
    let args = utils::expand_args(args, variables);
    Command::new(script).args(args).output()
}
