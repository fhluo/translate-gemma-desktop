use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use std::collections::HashMap;

#[skip_serializing_none]
#[derive(Debug, Default, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub suffix: Option<String>,
    pub images: Option<Vec<Vec<u8>>>,
    pub format: Option<Format>,
    pub system: Option<String>,
    /// Streaming is enabled by default.
    pub stream: Option<bool>,
    /// Reasoning control: boolean toggle or effort level.
    #[serde(rename = "think")]
    pub reasoning: Option<Reasoning>,
    pub raw: Option<bool>,

    /// How long to keep the model loaded after the request (default: "5m").
    pub keep_alive: Option<String>,
    pub options: Option<HashMap<String, Value>>,
}

#[derive(Debug, Serialize)]
pub enum Format {
    #[serde(rename = "json")]
    JSON,
    #[serde(untagged)]
    JSONSchema(Value),
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Reasoning {
    Reasoning(bool),
    ReasoningEffort(ReasoningEffort),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    High,
    Medium,
    Low,
}

impl GenerateRequest {
    pub fn builder() -> GenerateRequestBuilder {
        GenerateRequestBuilder(Default::default())
    }
}

pub struct GenerateRequestBuilder(GenerateRequest);

#[allow(dead_code)]
impl GenerateRequestBuilder {
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.0.model = model.into();
        self
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.0.prompt = prompt.into();
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.0.suffix = Some(suffix.into());
        self
    }

    pub fn image(mut self, image: impl Into<Vec<u8>>) -> Self {
        self.0
            .images
            .get_or_insert_with(Vec::new)
            .push(image.into());

        self
    }

    pub fn json(mut self) -> Self {
        self.0.format = Some(Format::JSON);
        self
    }

    pub fn json_schema(mut self, schema: Value) -> Self {
        self.0.format = Some(Format::JSONSchema(schema));
        self
    }

    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.0.system = Some(system.into());
        self
    }

    pub fn stream(mut self, stream: bool) -> Self {
        self.0.stream = Some(stream);
        self
    }

    pub fn stream_on(self) -> Self {
        self.stream(true)
    }

    pub fn stream_off(self) -> Self {
        self.stream(false)
    }

    pub fn reasoning(mut self, reasoning: bool) -> Self {
        self.0.reasoning = Some(Reasoning::Reasoning(reasoning));
        self
    }

    pub fn reasoning_effort(mut self, reasoning_effort: ReasoningEffort) -> Self {
        self.0.reasoning = Some(Reasoning::ReasoningEffort(reasoning_effort));
        self
    }

    pub fn reasoning_on(self) -> Self {
        self.reasoning(true)
    }

    pub fn reasoning_off(self) -> Self {
        self.reasoning(false)
    }

    pub fn reasoning_low(self) -> Self {
        self.reasoning_effort(ReasoningEffort::Low)
    }

    pub fn reasoning_medium(self) -> Self {
        self.reasoning_effort(ReasoningEffort::Medium)
    }

    pub fn reasoning_high(self) -> Self {
        self.reasoning_effort(ReasoningEffort::High)
    }

    pub fn raw(mut self, raw: bool) -> Self {
        self.0.raw = Some(raw);
        self
    }

    pub fn build(self) -> GenerateRequest {
        self.0
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub thinking: Option<String>,
    pub done: bool,
    pub done_reason: Option<String>,
    #[serde(flatten)]
    pub metrics: Metrics,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Metrics {
    pub total_duration: Option<i64>,
    pub load_duration: Option<i64>,
    pub prompt_eval_count: Option<isize>,
    pub prompt_eval_duration: Option<i64>,
    pub eval_count: Option<isize>,
    pub eval_duration: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_format() {
        println!("{:?}", serde_json::to_string_pretty(&Format::JSON).unwrap());
        println!(
            "{:?}",
            serde_json::to_string(&Format::JSONSchema(json!({
                "type": "json",
            })))
            .unwrap()
        );
    }

    #[test]
    fn test_reasoning_effort() {
        assert_eq!(
            serde_json::to_string(&ReasoningEffort::Medium).unwrap(),
            r#""medium""#
        );
    }

    #[test]
    fn test_think() {
        assert_eq!(
            serde_json::to_string(&Reasoning::ReasoningEffort(ReasoningEffort::Medium)).unwrap(),
            r#""medium""#
        );

        assert_eq!(
            serde_json::to_string(&Reasoning::Reasoning(true)).unwrap(),
            "true",
        );
    }

    #[test]
    #[ignore]
    fn test_generate_request() {
        println!("{:#?}", GenerateRequest::builder().build());
    }
}
