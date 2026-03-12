#[allow(unused_imports)]
use std::fs;
use std::io;
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

const EXIT_CMD: &str = "exit";
const ECHO_CMD: &str = "echo";
const TYPE_CMD: &str = "type";
const PROMPT: &str = "$ ";
const PWD_CMD: &str = "pwd";
const CD_CMD: &str = "cd";

fn clean_last_newline(s: &String) -> String {
    // s.trim().to_string()
    s.strip_suffix('\n').unwrap_or(s).to_string()
}

fn print_cleaned(s: &String) {
    if s != "" {
        println!("{}", clean_last_newline(s));
    }
}

fn repl_loop() {
    let config = rustyline::Config::builder().completion_type(rustyline::CompletionType::List).build();
    let mut rl = Editor::with_config(config).unwrap();
    let helper = MyHelper::new();
    rl.set_helper(Some(helper));    
    loop {
        let input = rl.readline(PROMPT).unwrap();
        let commands: Vec<&str> = input
                        .split('|')
                        .map(|c| c.trim())
                        .collect(); 
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
        let output = execute_piped(commands);
        match output {
            Ok(o) => {
                println!("{}", String::from_utf8_lossy(&o.stdout));
            },
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}

fn execute(mut command: String, input: Option<Stdio>, piped: bool) -> MyOutput {
    let mut result = MyOutput { status: 0, output: None, error: None };

    // TODO: Check if redirect
    let redirect_info = utils::get_redirect(&command);
    let redirect_stdout = redirect_info.redirect_stdout_file;
    let redirect_stderr = redirect_info.redirect_stderr_file;
    let index = redirect_info.file_index_start;
    let append_stdout = redirect_info.append_stdout;
    let append_stderr = redirect_info.append_stderr;
    if redirect_stdout.is_some() || redirect_stderr.is_some() {
        let index = index.unwrap() - 1;
        command = command[..index].trim().to_string();
        // TODO: Fix, dirty hack.
        if command.ends_with("1") || command.ends_with("2") {
            command = command[..command.len() - 1].trim().to_string();
        }
    }

    let mut stdout: Stdio;
    let mut stderr: Stdio;
    if command.trim() == EXIT_CMD {
        return MyOutput { status: 1, output: None, error: None };
    } else if command.starts_with(&*format!("{} ", &ECHO_CMD)) {
        result = builtins::handle_echo_command(&command);
    } else if command.starts_with(&*format!("{} ", &TYPE_CMD)) {
        result = builtins::handle_type_command(&command);
    } else if command == String::from(PWD_CMD) {
        result = builtins::print_pwd();
    } else if command.starts_with(&*format!("{} ", &CD_CMD)) {
        result = builtins::handle_cd_command(&command);
    } else {
        (stdout, stderr) = executor::exec_command(&command, input);
    }

    if redirect_stdout.is_some() {
        let stdout_file = redirect_stdout.unwrap();
        if !fs::exists(&stdout_file).unwrap() {
            let _ = File::create(&stdout_file);
        }
        match result.output {
            Some(ref output) => {
                if append_stdout {
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&stdout_file)
                        .unwrap();
                    let cleaned_output = clean_last_newline(output);
                    if cleaned_output != "" {
                        writeln!(file, "{}", cleaned_output).unwrap();
                    }
                } else {
                    let mut file = File::create(stdout_file).unwrap();
                    let cleaned_output = clean_last_newline(output);
                    if cleaned_output != "" {
                        writeln!(file, "{}", cleaned_output).unwrap(); 
                    }
                }
            }, 
            None => { }
        }
        match result.error {
            Some(ref error) => {
                print_cleaned(error);
            }, 
            None => { 
            }
        }
    } else if redirect_stderr.is_some() {
        let stderr_file = redirect_stderr.unwrap();
        if !fs::exists(&stderr_file).unwrap() {
            let _ = File::create(&stderr_file);
        }
        match result.output {
            Some(ref output) => {
                print_cleaned(output); 
            }, 
            None => { }
        }
        match result.error {
            Some(ref error) => {
                if append_stderr {
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(&stderr_file)
                        .unwrap();
                    let cleaned_error = clean_last_newline(error);
                    if cleaned_error != "" {
                        writeln!(file, "{}", cleaned_error).unwrap();
                    }  
                } else {
                    let mut file = File::create(stderr_file).unwrap();
                    let cleaned_error = clean_last_newline(error);
                    if cleaned_error != "" {
                        writeln!(file, "{}", cleaned_error).unwrap();
                    }
                }
            }, 
            None => { 
            }
        }
    } else {
        if !piped {
            match result.output {
                Some(ref output) => {
                    print_cleaned(output);
                }, 
                None => { }
            }
            match result.error {
                Some(ref error) => {
                    print_cleaned(error);
                }, 
                None => {
                }
            }
        }
    }
    result
}

fn execute_piped(cmds: Vec<&str>) -> io::Result<std::process::Output> {

    let mut children: Vec<Child> = Vec::new();
    let mut previous: Option<ChildStdout> = None;

    for (i, c) in cmds.iter().enumerate() {
        let split: Vec<&str> = c.split(" ").collect();

        let mut cmd = Command::new(split[0]);
        let mut j = 1;
        while j < split.len() {
            cmd.arg(split[j]);
            j += 1;
        }

        if let Some(stdin) = previous.take() {
            cmd.stdin(stdin);
        }

        if i < cmds.len() - 1 {
            cmd.stdout(Stdio::piped());
        }

        let mut child = cmd.spawn()?;

        if i < cmds.len() - 1 {
            previous = child.stdout.take();
        }

        children.push(child);
    }

    let last = children.pop().unwrap();
    let output = last.wait_with_output()?;

    for mut child in children {
        child.wait().unwrap();
    }

    Ok(output)
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}