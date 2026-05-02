use std::sync::{Arc, Mutex};
use rustyline::Context;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use crate::constants::SHELL_BUILTINS;
use crate::executor;
use crate::path_checker::list_executables;
use crate::complete::Complete;
use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

#[derive(Helper, Hinter, Validator, Highlighter)]
pub struct MyHelper {
    executables: Vec<String>,
    file_completion: FilenameCompleter,
    // Arc<Mutex<>> so registered scripts are always visible — no stale clone
    complete: Arc<Mutex<Complete>>,
}

impl MyHelper {
    pub fn new(complete: Arc<Mutex<Complete>>) -> Self {
        let builtins = SHELL_BUILTINS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let externals = list_executables();
        let executables = [builtins, externals].concat();
        Self {
            executables,
            file_completion: FilenameCompleter::new(),
            complete, // no redundant .clone() — value is already owned
        }
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let line_to_cursor = &line[..pos];

        if line_to_cursor.contains(' ') {
            // ── Argument completion ──────────────────────────────────────────
            let start = line_to_cursor
                .rfind(' ')
                .map(|i| i + 1)
                .unwrap_or(0);
            let token = &line_to_cursor[start..];

            // The key for script lookup is the COMMAND (first word), not the
            // partial token being typed. E.g. for "git ch<TAB>" the command is
            // "git", not "ch".
            let words: Vec<&str> = line_to_cursor.split_whitespace().collect();
            let command = words.first().copied().unwrap_or("");

            // `token` is the partial word at the cursor (may be empty if the
            // line ends with a space). Use the collected word list to find the
            // word that immediately precedes it:
            //   - cursor mid-word  ("git checkout br<TAB>") → prev = words[-2]
            //   - cursor after gap ("git checkout <TAB>")   → prev = words[-1]
            let word_being_completed = token;
            let previous_word = if line_to_cursor.ends_with(' ') {
                words.last().copied().unwrap_or("")
            } else {
                words.len()
                    .checked_sub(2)
                    .map(|i| words[i])
                    .unwrap_or("")
            };
                        
            // Clone the script string out of the lock so we don't hold the
            // mutex across the executor call.
            let script = {
                let complete = self.complete.lock().unwrap();
                complete.scripts.get(command).cloned()
            };

            if let Some(script) = script {
                if !script.is_empty() {
                    let mut args: Vec<&str> = Vec::new();
                    args.push(command);            // $1 — command name
                    args.push(word_being_completed); // $2 — word being completed
                    args.push(previous_word);     // $3 — previous word
                    match executor::execute_script(&script, args) {
                        Ok(output) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);

                            // Filter script output to entries that match the
                            // token already typed, then build Pair list.
                            let pairs: Vec<Pair> = stdout
                                .lines()
                                .filter(|l| l.starts_with(token))
                                .map(|l| Pair {
                                    display: l.to_string(),
                                    replacement: format!("{} ", l),
                                })
                                .collect();

                            return Ok((start, pairs));
                        }
                        Err(e) => eprintln!("Completion script error: {e}"),
                    }
                }
            }

            // Fallback: filename completion
            let (start, mut matches) =
                self.file_completion.complete_path_unsorted(line, pos)?;

            matches.sort_by(|a, b| a.display.cmp(&b.display));

            matches.iter_mut().for_each(|pair| {
                let last_segment = line_to_cursor.rsplit(' ').next().unwrap_or("");
                let raw_path = std::path::Path::new(last_segment);

                let path = if raw_path.is_dir() {
                    raw_path.join(&pair.display)
                } else {
                    raw_path
                        .parent()
                        .unwrap_or_else(|| std::path::Path::new(""))
                        .join(&pair.display)
                };

                if path.is_dir() {
                    pair.display.push('/');
                }

                if !pair.replacement.ends_with('/') {
                    pair.replacement.push(' ');
                }
            });

            Ok((start, matches))
        } else {
            // ── Command completion (first word, no space yet) ────────────────
            // Return nothing for an empty line; listing every executable is
            // overwhelming and not standard shell behaviour.
            if line_to_cursor.is_empty() {
                return Ok((0, vec![]));
            }

            let mut matches: Vec<Pair> = self
                .executables
                .iter()
                .filter(|c| c.starts_with(line_to_cursor))
                .map(|c| Pair {
                    display: c.clone(),
                    replacement: format!("{} ", c),
                })
                .collect();

            matches.sort_by(|a, b| a.display.cmp(&b.display));

            Ok((0, matches))
        }
    }
}