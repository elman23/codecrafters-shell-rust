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

    // History
    let mut history: Vec<String> =  Vec::new(); 
    load_history_from_file(&mut history);

    loop {
        let input = rl.readline(constants::PROMPT).unwrap();
        rl.add_history_entry(input.as_str()).unwrap();
        history.push(format!("\t{}  {}", history.len() + 1, input.clone())); // TODO: Is there a better way?
        let ec: std::io::Result<u8> = executor::execute(input, &mut history);
        match ec {  
            Ok(0) => { },
            _ => { break; }
        }
    }

    save_history_to_file(&mut history);
}

fn load_history_from_file(history: &mut Vec<String>) {
    let history_file = std::env::var("HISTFILE");
    match history_file {
        Ok(f) => {
            let file_content = utils::read_file_content(&f);
            let mut lines: Vec<String> = file_content.split('\n')
                                                 .filter(|s| !s.is_empty())
                                                 .enumerate()
                                                 .map(|(i, s)| format!("\t{}  {}", i, s))
                                                 .collect();
            history.append(&mut lines);
        },
        Err(_) => { }
    }
}

fn save_history_to_file(history: &Vec<String>) {
    let history_file = std::env::var("HISTFILE");
    match history_file {
        Ok(f) => {
            let content: Vec<&str> = history.iter().map(|s| s.trim().split_once(" ").unwrap_or(("", "")).1.trim()).collect();
            let mut content = content.join("\n");
            content.push('\n');
            let _ = utils::write_file(&f, &content);
        },
        Err(_) => { }
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}