pub use crate::ollama::types::{GenerateRequest, GenerateResponse};
use anyhow::anyhow;
use futures_util::stream::BoxStream;
use futures_util::{StreamExt, TryStreamExt};
use reqwest::header::ACCEPT;
use reqwest::RequestBuilder;
use serde::Deserialize;
use std::io;
use std::sync::LazyLock;
use tokio::runtime::{Handle, Runtime};
use tokio_util::codec::{FramedRead, LinesCodec};
use tokio_util::io::StreamReader;
use url::Url;

pub const DEFAULT_BASE_URL: &'static str = "http://127.0.0.1:11434";

static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to create tokio runtime")
});

pub struct Client {
    base_url: Url,
    client: reqwest::Client,
    handle: Handle,
}

impl Default for Client {
    fn default() -> Self {
        Client {
            base_url: DEFAULT_BASE_URL.parse().unwrap(),
            client: reqwest::Client::new(),
            handle: RUNTIME.handle().clone(),
        }
    }
}

impl Client {
    fn get(&self, path: &'static str) -> RequestBuilder {
        self.client.get(self.base_url.join(path).unwrap().as_str())
    }

    fn post(&self, path: &'static str) -> RequestBuilder {
        self.client.post(self.base_url.join(path).unwrap().as_str())
    }
}

impl Client {
    pub async fn version(&self) -> anyhow::Result<String> {
        #[derive(Deserialize)]
        struct Version {
            version: String,
        }

        let request = self.get("api/version");

        let version = self
            .handle
            .spawn(async { request.send().await })
            .await??
            .json::<Version>()
            .await?;

        Ok(version.version)
    }
}

static DEFAULT_CLIENT: LazyLock<Client> = LazyLock::new(|| Client::default());

pub async fn version() -> anyhow::Result<String> {
    DEFAULT_CLIENT.version().await
}

impl Client {
    pub async fn generate(
        &self,
        generate_request: GenerateRequest,
    ) -> anyhow::Result<BoxStream<'static, anyhow::Result<GenerateResponse>>> {
        let request = self.post("/api/generate");

        let resp = self
            .handle
            .spawn(async move {
                request
                    .header(ACCEPT, "application/x-ndjson")
                    .json(&generate_request)
                    .send()
                    .await
            })
            .await??;

        let status_code = resp.status();
        if !status_code.is_success() {
            let text = resp.text().await?;
            anyhow::bail!("{}: {}", status_code, text);
        }

        let stream = resp
            .bytes_stream()
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err));

        let reader = StreamReader::new(stream);
        let lines = FramedRead::new(reader, LinesCodec::new());

        Ok(lines
            .filter_map(async |line| match line {
                Ok(line) if line.trim().is_empty() => None,
                Ok(line) => Some(
                    serde_json::from_str::<GenerateResponse>(&line).map_err(|err| anyhow!(err)),
                ),
                Err(err) => Some(Err(anyhow!(err))),
            })
            .boxed())
    }
}

pub async fn generate(
    generate_request: GenerateRequest,
) -> anyhow::Result<BoxStream<'static, anyhow::Result<GenerateResponse>>> {
    DEFAULT_CLIENT.generate(generate_request).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use crate::prompt::Prompt;

    #[tokio::test]
    #[ignore]
    async fn test_version() {
        println!("{:?}", version().await);
    }

    #[tokio::test]
    #[ignore]
    async fn test_generate() {
        let prompt = Prompt::new(
            Language::new("zh-Hans", "Chinese"),
            Language::new("en", "english"),
            "你好，世界！",
        );
        let req = GenerateRequest::builder()
            .model("translategemma:4b")
            .prompt(prompt.to_string())
            .build();

        let mut result = generate(req).await.unwrap();

        while let Some(item) = result.next().await {
            println!("{}", item.unwrap().response);
        }
    }
}
