use std::collections::HashMap;

pub struct SourceContext {
    raw_source: String,
    parameters: HashMap<String, String>,
}

impl SourceContext {
    pub fn new(raw_source: &str) -> SourceContext {
        let parameters = HashMap::new();
        SourceContext {
            raw_source: raw_source.to_string(),
            parameters,
        }
    }

    pub fn add_parameter(&mut self, key: &str, val: &str) {
        self.parameters.insert(key.to_string(), val.to_string());
    }

    pub fn generate_source(&self) -> String {
        self.raw_source.clone()
    }
}
