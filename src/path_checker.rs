use std::env;
use std::fs;
use std::collections::HashSet;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::MetadataExt;

pub fn check_path(command: &str) -> Option<Vec<String>> {
    let path_var = env::var("PATH").unwrap_or_default();
    let mut executables: Vec<String> = Vec::new();

    for dir in env::split_paths(&path_var) {
        // read_dir already implies is_dir; skip the extra syscall
        let Ok(read_dir) = fs::read_dir(&dir) else { continue };

        for entry in read_dir.flatten() {
            // DirEntry::file_name() avoids a full path allocation
            let file_name = entry.file_name();
            let name_lossy = file_name.to_string_lossy();

            // Filter by prefix *before* any further syscalls
            if !name_lossy.starts_with(command) {
                continue;
            }

            // entry.metadata() uses fstatat() — no path lookup overhead
            let Ok(metadata) = entry.metadata() else { continue };

            if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 {
                executables.push(name_lossy.into_owned());
            }
        }
    }

    if executables.is_empty() {
        None
    } else {
        executables.sort_unstable(); // faster than stable sort; order ties don't matter
        Some(executables)
    }
}

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