use rustyline::{Context};
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use crate::builtins::SHELL_BUILTINS;
use crate::path_checker::list_executables;

use rustyline_derive::{Helper, Highlighter, Hinter, Validator};

#[derive(Helper, Hinter, Validator, Highlighter)]
pub struct MyHelper {
    executables: Vec<String>,
    file_completion: FilenameCompleter,
}

impl MyHelper {
    pub fn new() -> Self {
        let builtins = SHELL_BUILTINS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let externals = list_executables();
        let executables = [builtins, externals].concat();
        let file_completion = FilenameCompleter::new();
        Self {
            executables,
            file_completion,
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
        if line.is_empty() || line[..pos].contains(' ') {
            let (start,mut matches) = self.file_completion.complete_path_unsorted(line, pos)?;
            matches.sort_by(|a, b| a.display.cmp(&b.display));
            matches.iter_mut().for_each(|pair| {
                let last_segment = line[..pos].rsplit(' ').next().unwrap_or("");
                let raw_path = std::path::Path::new(last_segment);
                let path = if raw_path.is_dir() {
                    raw_path.join(&pair.display)
                } else {
                    raw_path.parent().unwrap_or_else(|| std::path::Path::new("")).join(&pair.display)
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
            let mut matches: Vec<Pair> = self.executables.iter().filter(|c| c.starts_with(&line[..pos])).map(|c| Pair {
                display: c.clone(),
                replacement: c.clone() + " "
            }).collect();
            matches.sort_by(|a, b| a.display.cmp(&b.display));
            Ok((0, matches))
        }
    }
}