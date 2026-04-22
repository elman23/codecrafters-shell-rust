use std::collections::HashMap;

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