use std::collections::HashMap;

pub struct SourceContext {
    raw_source: String,
    parameters: HashMap<String, String>,
    pub code: String,
}

impl SourceContext {
    pub fn new(raw_source: &str) -> SourceContext {
        let parameters = HashMap::new();
        SourceContext {
            raw_source: raw_source.to_string(),
            parameters,
            code: "".to_string(),
        }
    }

    pub fn add_parameter(&mut self, key: &str, val: &str) {
        self.parameters.insert(key.to_string(), val.to_string());
    }

    pub fn add_code(&mut self, code: String) {
        self.code = code;
    }

    fn generate_header(&self) -> String {
        let mut define_string = "#version 300 es\n".to_string();
        for (key, value) in &self.parameters {
            define_string.push_str(&format!("#define {} {}\n", key, value));
        }
        define_string
    }

    pub fn generate_source(&self) -> String {
        let mut source = self.generate_header();
        source.push_str(&self.raw_source.replace("#version 300 es", "").clone());
        source = source.replace("// Disc code here!", &self.code);
        source
    }
}
