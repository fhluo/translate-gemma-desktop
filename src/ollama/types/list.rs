use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ListResponse {
    pub models: Vec<Model>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Model {
    pub name: String,
    pub modified_at: String,
    pub size: i64,
    pub digest: String,
    pub details: ModelDetails,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ModelDetails {
    pub format: String,
    pub family: String,
    pub families: Vec<String>,
    pub parameter_size: String,
    pub quantization_level: String,
}
