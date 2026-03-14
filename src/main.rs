#[allow(unused_imports)]
use std::fs;
use rustyline::Editor;
use crate::my_helper::MyHelper;

mod executor;
mod builtins;
mod utils;
mod my_helper;
mod path_checker;

const PROMPT: &str = "$ ";

fn repl_loop() {
    let config = rustyline::Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl = Editor::with_config(config).unwrap();
    let helper = MyHelper::new();
    rl.set_helper(Some(helper));    
    loop {
        let input = rl.readline(PROMPT).unwrap();
        let ec = executor::execute(input);
        if !ec.success() {
            break;
        }
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}