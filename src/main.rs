#[allow(unused_imports)]
use std::fs;
use std::{fs::{File, OpenOptions}, io::Write, process::{Child, ChildStdout, Command}};
use std::process::Stdio;
use rustyline::Editor;
use crate::{my_helper::MyHelper, output::MyOutput};

mod executor;
mod builtins;
mod output;
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
        
        // let mut previous_output: MyOutput = MyOutput { status: 0, output: None, error: None };
        // let mut count = 1;
        // let l = commands.len();
        // for command in commands {
        //     let input: Option<Stdio>;
        //     if count > 1 {
        //         input = previous_output.output;
        //     } else {
        //         input = None;
        //     }
        //     let my_output = execute(command.to_string(), input, count < l);
        //     if my_output.status == 1 {
        //         break;
        //     }
        //     previous_output = my_output;
        //     count += 1;
        // }
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