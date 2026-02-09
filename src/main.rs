#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // Prompt
    print!("$ ");
    io::stdout().flush().unwrap();
    
    // Wait for user input
    let mut command: String = String::new();
    io::stdin().read_line(&mut command).unwrap();
    println!("{}: command not found", command.trim());
}
