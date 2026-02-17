use crate::parser::{parse_echo_args, Command};

pub fn run(cmd: &Command) {
    let output = parse_echo_args(&cmd.raw_args);
    println!("{}", output);
}