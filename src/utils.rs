use std::collections::{HashMap, HashSet};
use std::fs::OpenOptions;
use std::io::{self, Write};

use sysinfo::{Pid, ProcessStatus, ProcessesToUpdate, System};

// ── Redirect parsing ──────────────────────────────────────────────────────────

pub struct RedirectInfo {
    pub redirect_stdout_file: Option<String>,
    pub redirect_stderr_file: Option<String>,
    /// Byte offset in the original string where the redirect token begins.
    /// The caller can use `&command[..file_index_start]` to strip the redirect.
    pub file_index_start: Option<usize>,
    pub append_stdout: bool,
    pub append_stderr: bool,
}

#[derive(Clone, Copy)]
enum RedirectMode {
    Stdout,
    Stderr,
}

/// Parse the first unquoted redirect operator (`>`, `>>`, `2>`, `2>>`, `1>`,
/// `1>>`) found in `input` and return metadata about it.
///
/// A digit (`1` or `2`) is only treated as a file-descriptor prefix when it
/// sits at a word boundary (preceded by a space or at the start of the string)
/// **and** is immediately followed by `>`.  Digits inside filenames or
/// arguments are left untouched.
pub fn get_redirect(input: &str) -> RedirectInfo {
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    // State once a redirect operator has been found
    let mut mode: Option<RedirectMode> = None;
    let mut filename = String::new();
    let mut append = false;
    let mut redirect_index: usize = 0;

    let mut byte_offset: usize = 0;
    let chars: Vec<char> = input.chars().collect();

    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        // ── Collecting the filename after the operator ────────────────────────
        if mode.is_some() {
            // A second `>` immediately after the operator means append (`>>`)
            if c == '>' && filename.is_empty() && !append {
                append = true;
            } else {
                filename.push(c);
            }
            byte_offset += c.len_utf8();
            i += 1;
            continue;
        }

        // ── Quote tracking ────────────────────────────────────────────────────
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            byte_offset += c.len_utf8();
            i += 1;
            continue;
        }
        if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            byte_offset += c.len_utf8();
            i += 1;
            continue;
        }

        if in_single_quote || in_double_quote {
            byte_offset += c.len_utf8();
            i += 1;
            continue;
        }

        // ── Redirect operator detection ───────────────────────────────────────

        // Check for `1>` or `2>` only at a word boundary so that digits inside
        // arguments (e.g. "file2.txt") are never mistaken for fd prefixes.
        let at_word_boundary = i == 0 || chars[i - 1] == ' ';
        if (c == '1' || c == '2') && at_word_boundary {
            if let Some('>') = chars.get(i + 1).copied() {
                redirect_index = byte_offset; // points at the digit
                mode = Some(if c == '2' {
                    RedirectMode::Stderr
                } else {
                    RedirectMode::Stdout
                });
                byte_offset += c.len_utf8() + '>'.len_utf8();
                i += 2; // consume digit + '>'
                continue;
            }
            // Not followed by `>` — treat as a normal character
        }

        if c == '>' {
            redirect_index = byte_offset;
            mode = Some(RedirectMode::Stdout);
            byte_offset += c.len_utf8();
            i += 1;
            continue;
        }

        byte_offset += c.len_utf8();
        i += 1;
    }

    let trimmed_filename = filename.trim().to_string();

    match mode {
        Some(RedirectMode::Stdout) => RedirectInfo {
            redirect_stdout_file: Some(trimmed_filename),
            redirect_stderr_file: None,
            file_index_start: Some(redirect_index),
            append_stdout: append,
            append_stderr: false,
        },
        Some(RedirectMode::Stderr) => RedirectInfo {
            redirect_stdout_file: None,
            redirect_stderr_file: Some(trimmed_filename),
            file_index_start: Some(redirect_index),
            append_stdout: false,
            append_stderr: append,
        },
        None => RedirectInfo {
            redirect_stdout_file: None,
            redirect_stderr_file: None,
            file_index_start: None,
            append_stdout: false,
            append_stderr: false,
        },
    }
}

// ── File I/O ──────────────────────────────────────────────────────────────────

/// Write `content` to `path`, creating the file if it doesn't exist and
/// **truncating** any existing content (same semantics as `>` in a shell).
pub fn write_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true) // was `false` — left stale bytes after shorter writes
        .open(path)?;
    file.write_all(content.as_bytes())
}

/// Overwrite `path` with `content`, truncating first.  Identical to
/// `write_file`; kept as a separate entry point for call-site clarity.
pub fn overwrite_file(path: &str, content: &str) -> io::Result<()> {
    // Previously spelled `owerwrite_file` — fixed typo to match callers.
    write_file(path, content)
}

/// Read the entire contents of `path` as a `String`, returning an empty string
/// if the file does not exist or cannot be read.
pub fn read_file_content(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

// ── Process helpers ───────────────────────────────────────────────────────────

/// Returns `true` if the process with the given `pid` is alive and not a zombie.
///
/// A fresh `System` is created on each call because this function is invoked
/// infrequently (job reaping).  If call frequency increases, consider threading
/// a shared `System` instance through the `Jobs` struct instead.
pub fn is_process_running(pid: u32) -> bool {
    let mut system = System::new();
    let sysinfo_pid = Pid::from(pid as usize);
    system.refresh_processes(ProcessesToUpdate::Some(&[sysinfo_pid]), false);

    system
        .process(sysinfo_pid)
        .map(|p| p.status() != ProcessStatus::Zombie)
        .unwrap_or(false)
}

pub fn validate(name: &str, declaration: &str) -> bool {
    let chars: Vec<char> = name.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i == 0 {
            if !(c.is_alphabetic() || c == &'_') {
                println!("declare: `{}': not a valid identifier", declaration);
                return false;
            }
        }
        if !(c.is_alphanumeric() || c == &'_') {
            println!("declare: `{}': not a valid identifier", declaration);
            return false;
        }
    }
    true
}

pub fn expand_args<'a, S>(args: Vec<S>, variables: &HashMap<String, String>) -> Vec<String>
where
    S: AsRef<str> + 'a,
{
    args.into_iter()
        .map(|arg| expand_string_with_vars(arg.as_ref(), variables))
        .collect()
}

fn expand_string(
    s: &str,
    variables: &HashMap<String, String>,
    visited: &mut HashSet<String>,
) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' {
            let mut key = String::new();
            let braced = if chars.peek() == Some(&'{') {
                chars.next(); // consume '{'
                true
            } else {
                false
            };

            // Collect the key
            while let Some(&next) = chars.peek() {
                if braced && next == '}' {
                    chars.next(); // consume '}'
                    break;
                } else if next.is_ascii_alphanumeric() || next == '_' {
                    key.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            if !key.is_empty() {
                if let Some(value) = variables.get(&key) {
                    // Check for cycles
                    if visited.contains(&key) {
                        // Cycle detected, return the variable as-is to avoid infinite recursion
                        result.push_str(&format!("${}", key));
                    } else {
                        visited.insert(key.clone());
                        let expanded = expand_string(value, variables, visited);
                        visited.remove(&key);
                        result.push_str(&expanded);
                    }
                } else {
                    // Variable not found, leave as-is
                    if braced {
                        result.push_str(&format!("${{{}}}", key));
                    } else {
                        result.push_str(&format!("${}", key));
                    }
                }
            } else {
                // No key found, just push the '$'
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }

    result
}

// Wrapper function for external use
pub fn expand_string_with_vars(s: &str, variables: &HashMap<String, String>) -> String {
    let mut visited = HashSet::new();
    expand_string(s, variables, &mut visited)
}
