#[allow(unused_imports)]
use std::fs;
use rustyline::Editor;
use crate::my_helper::MyHelper;

mod executor;
mod builtins;
mod utils;
mod my_helper;
mod path_checker;
mod constants;

fn repl_loop() {
    let config = rustyline::Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl = Editor::with_config(config).unwrap();
    let helper = MyHelper::new();
    rl.set_helper(Some(helper));    
    loop {
        let input = rl.readline(constants::PROMPT).unwrap();
        let ec: std::io::Result<u8> = executor::execute(input);
        match ec {  
            Ok(0) => { },
            _ => { break; }
        }
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}