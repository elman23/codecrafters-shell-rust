use std::io;
use std::io::{Write};

use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{Validator, ValidationContext, ValidationResult};
use rustyline::{Context, Editor, Helper};

pub struct MyHelper;

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let mut completed_line = String::new();
        if line == "ech" {
            completed_line = String::from("echo ");
            print!("o ");
            io::stdout().flush().unwrap();
        }
        if line == "exi" {
            completed_line = String::from("exit ");
            print!("t ");
            io::stdout().flush().unwrap();
        }
        Ok((0, vec![Pair {display: completed_line.clone(), replacement: completed_line}]))
    }
}

impl Hinter for MyHelper {
    type Hint = String;
}

impl Highlighter for MyHelper {}

impl Validator for MyHelper {
    fn validate(
        &self,
        _ctx: &mut ValidationContext,
    ) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Helper for MyHelper {}

// fn main() -> rustyline::Result<()> {
//     let mut rl = Editor::new()?;
//     rl.set_helper(Some(MyHelper));
//     // rl.set_helper(Some(()));
//     // loop {    
//         // rl.readline("> ")?;
//     // }
//     rl.readline("> ")?;
//     Ok(())
// }