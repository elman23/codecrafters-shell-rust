use std::{fs::OpenOptions, io::Write};

use sysinfo::{Pid, ProcessStatus, ProcessesToUpdate, System};

pub struct RedirectInfo {
    pub redirect_stdout_file: Option<String>,
    pub redirect_stderr_file: Option<String>,
    pub file_index_start: Option<usize>,
    pub append_stdout: bool,
    pub append_stderr: bool,
}

pub fn get_redirect(input: &str) -> RedirectInfo {
    let mut redirect_stdout = false;
    let mut redirect_stderr = false;
    let mut redirect_stdout_file = String::new();
    let mut redirect_stderr_file = String::new();
    let mut redirect_index: usize = 0;
    let mut previous: char = ' ';

    let mut append_stdout = false;
    let mut append_stderr = false;

    let mut in_single_quote = false;
    let mut in_double_quote = false;

    let mut counter = 0;
    for c in input.chars() {
        counter += 1;
        if redirect_stdout {
            if c == '>' {
                append_stdout = true;
                continue;
            }
            redirect_stdout_file.push(c);
            previous = c;
            continue;
        }
        if redirect_stderr {
            if c == '>' {
                append_stderr = true;
                continue;
            }
            redirect_stderr_file.push(c);
            previous = c;
            continue;
        }
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            previous = c;
            continue;
        }
        if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            previous = c;
            continue;
        }
        if c == '1' {
            previous = c;
            continue;
        }
        if c == '2' {
            previous = c;
            continue;
        }
        if c == '>' && !in_single_quote && !in_double_quote {
            if previous == '1' || previous == '2' {
                redirect_index = counter - 1;
            }
            if previous == '2' {
                redirect_stderr = true;
            } else {
                redirect_stdout = true;
            }
            if redirect_index == 0 {
                redirect_index = counter;
            }
        }
        previous = c;
    }

    if redirect_stdout {
        RedirectInfo { 
            redirect_stdout_file: Some(redirect_stdout_file.trim().to_string()), 
            redirect_stderr_file: None, 
            file_index_start: Some(redirect_index),
            append_stdout,
            append_stderr,
        }
    } else if redirect_stderr {
        RedirectInfo { 
            redirect_stdout_file: None, 
            redirect_stderr_file: Some(redirect_stderr_file.trim().to_string()), 
            file_index_start: Some(redirect_index) ,
            append_stdout,
            append_stderr,
        }
    } else {
        RedirectInfo {
            redirect_stdout_file: None, 
            redirect_stderr_file: None, 
            file_index_start: None ,
            append_stdout,
            append_stderr,
        }
    }
}

pub fn write_file(path: &str, content: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(path).expect(&format!("Failed to open file {}", path));

    file.write_all(content.as_bytes())
        .expect(&format!("Failed to write to file {}", path));
}

pub fn owerwrite_file(path: &str, content: &str) {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(path).expect(&format!("Failed to open file {}", path));

    file.write_all(content.as_bytes())
        .expect(&format!("Failed to write to file {}", path));
}

pub fn read_file_content(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or("".to_string())
}

pub fn is_process_running(pid: u32) -> bool {
    let mut system = System::new();
    let pid = Pid::from(pid as usize);
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);

    if let Some(process) = system.process(pid) {
        process.status() != ProcessStatus::Zombie
    } else {
        false
    }

}
