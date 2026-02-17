use std::env;
use std::path::Path;

use crate::parser::Command;

pub fn run(cmd: &Command) {
    let dir = match cmd.args.first() {
        Some(d) => d.as_str(),
        None => {
            // `cd` with no args goes home
            change_to_home();
            return;
        }
    };

    if dir == "~" {
        change_to_home();
        return;
    }

    let path = Path::new(dir);
    if path.exists() {
        if let Err(e) = env::set_current_dir(path) {
            eprintln!("cd: {}: {}", dir, e);
        }
    } else {
        eprintln!("cd: {}: No such file or directory", dir);
    }
}

fn change_to_home() {
    match env::var_os("HOME") {
        Some(home) => {
            if let Err(e) = env::set_current_dir(&home) {
                eprintln!("cd: {}", e);
            }
        }
        None => eprintln!("cd: HOME not set"),
    }
}