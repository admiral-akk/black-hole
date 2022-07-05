enum SourceType {
    VERTEX,
    FRAGMENT,
}

pub struct SourceContext {
    raw_source: String,
}

impl SourceContext {
    pub fn new(raw_source: &str) -> SourceContext {
        let mut source_type = SourceType::VERTEX;
        if raw_source.contains("gl_FragCoord") || raw_source.contains("outColor") {
            source_type = SourceType::FRAGMENT;
        }
        let raw_source = raw_source.to_string();
        SourceContext { raw_source }
    }

    pub fn generate_source(&self) -> String {
        self.raw_source.clone()
    }
}
