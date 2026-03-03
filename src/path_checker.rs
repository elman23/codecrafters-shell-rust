use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

pub fn check_path(command: &str) -> Option<String> {
    let path_var = env::var("PATH").unwrap_or_default();

    for dir in env::split_paths(&path_var) {
        if dir.is_dir() {
            for entry in fs::read_dir(&dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();

                if path.is_file() {
                    let metadata = fs::metadata(&path).unwrap();
                    let permissions = metadata.permissions();

                    // Check executable bits
                    if permissions.mode() & 0o111 != 0 {
                        if let Some(name) = path.file_name() {
                            let complete_name = String::from(name.to_string_lossy());
                            if complete_name.starts_with(command) {
                                return Some(format!("{} ", complete_name));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}