use std::collections::HashMap;

use crate::utils;

pub struct Jobs {
    pub jobs_list: HashMap<u32, String>,
    pub process_list: HashMap<u32, u32>,
}

impl Jobs {
    pub fn new() -> Self {
        Jobs {
            jobs_list: HashMap::new(),
            process_list: HashMap::new(),
        }
    }
}

pub fn reap_jobs(jobs: &mut Jobs, print: bool) {
    let mut keys: Vec<_> = jobs.jobs_list.keys().copied().collect();
    keys.sort();
    let total = keys.len();

    let mut done_jobs = Vec::new();

    for (i, k) in keys.iter().enumerate() {
        let v = jobs.jobs_list.get(k).unwrap();
        let pid = *jobs.process_list.get(k).unwrap();

        let is_running = utils::is_process_running(pid);
        let job_state = if is_running { "Running" } else { "Done" };

        // Determine marker: + (last), - (second last), or space
        let marker = if i == total - 1 {
            "+"
        } else if i == total.saturating_sub(2) {
            "-"
        } else {
            " "
        };

        // Clean command if done
        let display_cmd = v.replace(" &", "").to_string();

        if !is_running {
            if print {
                println!("[{}]{}  {:<8} {}", k, marker, job_state, display_cmd);
            }
            done_jobs.push(*k);
        }
    }

    for k in done_jobs {
        jobs.jobs_list.remove(&k);
        jobs.process_list.remove(&k);
    }
}