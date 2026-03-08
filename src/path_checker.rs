use std::collections::HashSet;
use std::os::unix::fs::MetadataExt;

fn is_executable(path: &str) -> bool {
    match std::fs::metadata(path) {
        Ok(metadata) => metadata.mode() & 0o111 != 0,
        Err(_) => false,
    }
}

pub fn list_executables() -> Vec<String> {
    let path_env_string = std::env::var("PATH").unwrap_or_default();
    let paths: Vec<&str> = path_env_string.split(":").collect();
    let mut executables = HashSet::new();
    for path in paths {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().into_string().unwrap_or_default();
                if is_executable(&format!("{}/{}", path, file_name)) {
                    executables.insert(file_name);
                }
            }
        }
    }
    executables.into_iter().collect()
}