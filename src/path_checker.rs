use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;

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