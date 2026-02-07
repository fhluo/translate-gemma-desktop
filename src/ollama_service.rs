use crate::ollama;
use gpui::{Context, EventEmitter};
use semver::Version;
use std::time::Duration;

pub struct OllamaService {
    pub version: Option<Version>,
    pub models: Vec<String>,
}

pub enum OllamaServiceEvent {
    VersionChanged,
    ModelsChanged,
}

impl EventEmitter<OllamaServiceEvent> for OllamaService {}

impl OllamaService {
    pub fn new() -> Self {
        OllamaService {
            version: None,
            models: Vec::new(),
        }
    }

    pub fn start_polling(&mut self, cx: &mut Context<Self>) {
        self.poll_version(cx);
        self.poll_models(cx);
    }

    fn poll_version(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async |this, cx| {
            loop {
                let version = ollama::version().await.ok();

                this.update(cx, |this, cx| {
                    this.version = version;
                    cx.emit(OllamaServiceEvent::VersionChanged);
                    cx.notify();
                })
                .ok();

                cx.background_executor().timer(Duration::from_mins(5)).await;
            }
        })
        .detach();
    }

    fn poll_models(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async |this, cx| {
            loop {
                let models = ollama::list().await.ok();

                if let Some(models) = models {
                    let models = models
                        .into_iter()
                        .filter_map(|model| {
                            if model.name.starts_with("translategemma") {
                                Some(model.name)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    this.update(cx, |this, cx| {
                        this.models = models;
                        cx.emit(OllamaServiceEvent::ModelsChanged);
                        cx.notify();
                    })
                    .ok();
                }

                cx.background_executor().timer(Duration::from_mins(5)).await;
            }
        })
        .detach();
    }
}
