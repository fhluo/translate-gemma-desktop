#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rust_i18n;

mod assets;
mod config;
mod language;
mod language_selector;
mod locale_selector;
mod ollama;
mod prompt;

use crate::assets::{Assets, Icons};
use crate::config::{Config, ConfigEvent};
use crate::language_selector::LanguageSelector;
use crate::locale_selector::{ChangeLocale, LocaleSelector};
use crate::ollama::{generate, GenerateRequest};
use crate::prompt::Prompt;
use futures_util::StreamExt;
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, ClickEvent, Entity,
    FocusHandle, Task, Window, WindowBounds, WindowOptions,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::{gray_600, Root, TitleBar};
use std::time::Duration;

i18n!("locales", fallback = "en");

struct TranslateApp {
    config: Entity<Config>,

    locale_selector: Entity<LocaleSelector>,

    source_language_selector: Entity<LanguageSelector>,
    target_language_selector: Entity<LanguageSelector>,

    input_editor: Entity<InputState>,
    output_editor: Entity<InputState>,

    focus_handle: FocusHandle,

    generate: Option<Task<anyhow::Result<()>>>,
}

impl TranslateApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let locale_selector = cx.new(|_| LocaleSelector::new(focus_handle.clone()));

        let source_language_selector = cx.new(|cx| LanguageSelector::new(window, cx));

        cx.subscribe(&source_language_selector, |this, _, event, cx| {
            if let Some(language) = event.value() {
                this.config.update(cx, |this, cx| {
                    this.set_source_language(language.code, cx);
                })
            }
        })
        .detach();

        let target_language_selector = cx.new(|cx| LanguageSelector::new(window, cx));
        cx.subscribe(&target_language_selector, |this, _, event, cx| {
            if let Some(language) = event.value() {
                this.config.update(cx, |this, cx| {
                    this.set_target_language(language.code, cx);
                })
            }
        })
        .detach();

        let input_editor = cx.new(|cx| InputState::new(window, cx).multi_line(true));
        let output_editor = cx.new(|cx| InputState::new(window, cx).multi_line(true));

        cx.subscribe_in(&input_editor, window, Self::on_input_event)
            .detach();

        TranslateApp {
            config: Self::setup_config(window, cx),
            locale_selector,
            source_language_selector,
            target_language_selector,
            input_editor,
            output_editor,
            focus_handle,
            generate: None,
        }
    }

    fn setup_config(window: &mut Window, cx: &mut Context<Self>) -> Entity<Config> {
        let config = cx.new(|_| Config::load("translate-gemma-desktop"));

        cx.observe_new(|this: &mut Self, window, cx| {
            let source_language_selector = this.source_language_selector.clone();
            let target_language_selector = this.target_language_selector.clone();

            this.config.update(cx, |this, cx| {
                this.init(cx);

                if let Some(window) = window {
                    if let Some(language) = this.get_source_language() {
                        source_language_selector.update(cx, |this, cx| {
                            this.set_selected_language(language, window, cx)
                        });
                    }

                    if let Some(language) = this.get_target_language() {
                        target_language_selector.update(cx, |this, cx| {
                            this.set_selected_language(language, window, cx)
                        })
                    }
                }
            });
        })
        .detach();

        cx.on_release(|this, cx| {
            this.config.update(cx, |this, _| {
                this.store();
            });
        })
        .detach();

        cx.subscribe_in(&config, window, |this, _, event, window, cx| match event {
            ConfigEvent::LocaleChange => {
                this.source_language_selector.update(cx, |this, cx| {
                    this.update_items(window, cx);
                });
                this.target_language_selector.update(cx, |this, cx| {
                    this.update_items(window, cx);
                });
                window.refresh();
            }
            ConfigEvent::SourceLanguageChange(language) => {
                if let Some(language) = language {
                    this.source_language_selector.update(cx, |this, cx| {
                        this.set_selected_language(language, window, cx);
                    });
                    this.translate(window, cx);
                }
            }
            ConfigEvent::TargetLanguageChange(language) => {
                if let Some(language) = language {
                    this.target_language_selector.update(cx, |this, cx| {
                        this.set_selected_language(language, window, cx);
                    });
                    this.translate(window, cx);
                }
            }
            ConfigEvent::SwapLanguages {
                source_language,
                target_language,
            } => {
                if let Some(language) = source_language {
                    this.source_language_selector.update(cx, |this, cx| {
                        this.set_selected_language(language, window, cx);
                    });
                }

                if let Some(language) = target_language {
                    this.target_language_selector.update(cx, |this, cx| {
                        this.set_selected_language(language, window, cx);
                    });
                }

                this.translate(window, cx);
            }
        })
        .detach();

        config
    }

    fn on_input_event(
        &mut self,
        _: &Entity<InputState>,
        event: &InputEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(event, InputEvent::Change) {
            self.translate(window, cx);
        }
    }

    fn prompt(&mut self, cx: &App) -> Option<Prompt> {
        let source_language = self.source_language_selector.read(cx).selected_language(cx);
        let target_language = self.target_language_selector.read(cx).selected_language(cx);

        if let (Some(source_language), Some(target_language)) = (source_language, target_language) {
            Some(Prompt::new(
                source_language,
                target_language,
                self.input_editor.read(cx).value(),
            ))
        } else {
            None
        }
    }

    fn translate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(prompt) = self.prompt(cx) {
            let output_editor = self.output_editor.clone();
            self.generate = Some(cx.spawn_in(window, async move |_, window| {
                window
                    .background_executor()
                    .timer(Duration::from_millis(500))
                    .await;

                let req = GenerateRequest::builder()
                    .model("translategemma:4b")
                    .stream(true)
                    .prompt(prompt.to_string())
                    .build();

                let mut result = generate(req).await?;

                output_editor.update_in(window, |state, window, cx| {
                    state.set_value("", window, cx);
                })?;

                while let Some(item) = result.next().await {
                    let response = item?.response;

                    output_editor.update_in(window, |state, window, cx| {
                        state.insert(response, window, cx);
                    })?;
                }

                Ok::<_, anyhow::Error>(())
            }));
        }
    }

    fn on_action_change_locale(
        &mut self,
        ChangeLocale(locale): &ChangeLocale,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.config.update(cx, |this, cx| {
            this.set_locale(locale, cx);
        });
    }

    fn on_click_swap_languages(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.config.update(cx, |this, cx| {
            this.swap_languages(cx);
        })
    }
}

impl Render for TranslateApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                TitleBar::new().items_center().child(
                    div().flex().flex_row().flex_1().child(
                        div()
                            .track_focus(&self.focus_handle)
                            .on_action(cx.listener(Self::on_action_change_locale))
                            .flex()
                            .flex_row()
                            .h_full()
                            .ml_auto()
                            .mr_3()
                            .child(self.locale_selector.clone()),
                    ),
                ),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .py_2()
                    .px_3()
                    .gap_1()
                    .child(self.source_language_selector.clone())
                    .child(
                        Button::new("swap-button")
                            .ghost()
                            .icon(Icons::ArrowRightLeft)
                            .text_color(gray_600())
                            .tooltip(t!("swap-languages"))
                            .on_click(cx.listener(Self::on_click_swap_languages)),
                    )
                    .child(self.target_language_selector.clone()),
            )
            .child(
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .flex_row()
                    .px_3()
                    .py_1()
                    .gap_3()
                    .child(Input::new(&self.input_editor).flex_1())
                    .child(Input::new(&self.output_editor).flex_1()),
            )
    }
}

fn main() -> anyhow::Result<()> {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(800.), px(600.)), cx);

        cx.spawn(async move |cx| {
            cx.open_window(
                WindowOptions {
                    titlebar: Some(TitleBar::title_bar_options()),
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |window, cx| {
                    let view = cx.new(|cx| TranslateApp::new(window, cx));

                    cx.new(|cx| Root::new(view, window, cx))
                },
            )?;

            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });

    Ok(())
}
