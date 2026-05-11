use crate::{complete::Complete, helper::MyHelper, jobs::Jobs};
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod builtins;
mod complete;
mod constants;
mod executor;
mod helper;
mod jobs;
mod path_checker;
mod utils;

fn repl_loop() {
    let config = rustyline::Config::builder()
        .completion_type(rustyline::CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config).unwrap();

    // Jobs list
    let mut jobs = Jobs::new();

    // Variables list
    let mut variables: HashMap<String, String> = HashMap::new();

    // Complete script list — wrapped in Arc<Mutex> so MyHelper always sees updates
    let complete = Arc::new(Mutex::new(Complete::new()));

    // Helper (tab completion) shares the same Complete instance
    let variables_arg = Arc::new(Mutex::new(variables.clone()));
    let helper = MyHelper::new(Arc::clone(&complete), Arc::clone(&variables_arg));
    rl.set_helper(Some(helper));

    // History
    let mut history: Vec<String> = Vec::new();
    load_history_from_file(&mut history);

    loop {
        jobs::reap_jobs(&mut jobs, true);

        let input = match rl.readline(constants::PROMPT) {
            Ok(line) => line,
            // Ctrl+D: end of input — exit cleanly
            Err(ReadlineError::Eof) => break,
            // Ctrl+C: interrupt — exit cleanly
            Err(ReadlineError::Interrupted) => break,
            Err(e) => {
                eprintln!("Readline error: {e}");
                break;
            }
        };

        rl.add_history_entry(input.as_str()).unwrap();
        history.push(format!("\t{}  {}", history.len() + 1, input.clone()));

        let ec: std::io::Result<u8> = executor::execute(
            input,
            &mut history,
            &mut jobs,
            &complete,      // pass Arc reference so executor can register new scripts
            &mut variables, // TODO: Improve using Arc and Mutex
        );

        match ec {
            Ok(0) => {}
            _ => break,
        }
    }

    save_history_to_file(&history);
}

fn load_history_from_file(history: &mut Vec<String>) {
    match std::env::var("HISTFILE") {
        Ok(f) => {
            let file_content = utils::read_file_content(&f);
            let mut lines: Vec<String> = file_content
                .split('\n')
                .filter(|s| !s.is_empty())
                .enumerate()
                .map(|(i, s)| format!("\t{}  {}", i, s))
                .collect();
            history.append(&mut lines);
        }
        Err(_) => {}
    }
}

// Takes &[String] instead of &Vec<String> — more idiomatic, accepts any contiguous slice
fn save_history_to_file(history: &[String]) {
    match std::env::var("HISTFILE") {
        Ok(f) => {
            let content: Vec<&str> = history
                .iter()
                .map(|s| s.trim().split_once(' ').unwrap_or(("", "")).1.trim())
                .collect();
            let mut content = content.join("\n");
            content.push('\n');
            let _ = utils::write_file(&f, &content);
        }
        Err(_) => {}
    }
}

fn main() {
    repl_loop();
}
