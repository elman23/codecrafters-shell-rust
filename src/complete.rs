use std::collections::HashMap;

pub struct Complete {
    pub scripts: HashMap<String, String>,
}

impl Complete {
    pub fn new() -> Self {
        Complete {
            scripts: HashMap::new(),
        }
    }
}