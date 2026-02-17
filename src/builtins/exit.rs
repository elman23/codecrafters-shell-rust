use crate::parser::Command;

pub fn run(cmd: &Command) {
    let code: i32 = cmd.args.first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    std::process::exit(code);
}