#[allow(unused_imports)]
use std::io::{self, Write};

fn repl_loop() {
    loop {
        // Display prompt.
        print!("$ ");
        io::stdout().flush().unwrap();
        
        // Wait for user input.
        let mut command: String = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = String::from(command.trim());

        // Exit builtin.
        if command == String::from("exit") {
            break;
        }

        println!("{}: command not found", command);
    }
}

fn main() {
    // Shell's infinite REPL loop.
    repl_loop();
}
