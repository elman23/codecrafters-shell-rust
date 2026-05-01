use std::collections::HashMap;

#[derive(Clone, Debug)]
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