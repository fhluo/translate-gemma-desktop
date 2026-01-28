#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rust_i18n;

mod about;
mod assets;
mod config;
mod editor;
mod input_editor;
mod language;
mod language_selector;
mod locale_selector;
mod ollama;
mod output_editor;
mod prompt;
mod status_bar;

use crate::about::open_about_dialog;
use crate::assets::{Assets, Icons};
use crate::config::{Config, ConfigEvent};
use crate::input_editor::InputEditor;
use crate::language_selector::LanguageSelector;
use crate::locale_selector::{ChangeLocale, LocaleSelector};
use crate::ollama::{generate, GenerateRequest};
use crate::output_editor::OutputEditor;
use crate::prompt::Prompt;
use crate::status_bar::StatusBar;
use futures_util::StreamExt;
use gpui::{
    actions, div, prelude::*, px, size, Action, App, Application, Bounds, ClickEvent, Entity,
    Focusable, Menu, MenuItem, Task, Window, WindowBounds, WindowOptions,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{InputEvent, InputState};
use gpui_component::menu::AppMenuBar;
use gpui_component::{gray_600, Root, TitleBar};
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::time::Duration;

i18n!("locales", fallback = "en");

actions!([About, Repository]);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Action)]
struct ChangeModel {
    name: String,
}

impl ChangeModel {
    fn new(name: impl Into<String>) -> Self {
        ChangeModel { name: name.into() }
    }
}

struct TranslateApp {
    config: Entity<Config>,

    locale_selector: Entity<LocaleSelector>,

    source_language_selector: Entity<LanguageSelector>,
    target_language_selector: Entity<LanguageSelector>,

    menu_bar: Entity<AppMenuBar>,

    input_editor: Entity<InputEditor>,
    output_editor: Entity<OutputEditor>,

    generate: Option<Task<anyhow::Result<()>>>,

    ollama_version: Option<Version>,
    models: Vec<String>,
}

impl TranslateApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_editor = cx.new(|cx| InputEditor::new(window, cx));
        let output_editor = cx.new(|cx| OutputEditor::new(window, cx));

        let input_state = input_editor.read(cx).state.clone();
        cx.subscribe_in(&input_state, window, Self::on_input_event)
            .detach();

        let locale_selector =
            cx.new(|cx| LocaleSelector::new(input_editor.focus_handle(cx).clone()));

        TranslateApp {
            config: Self::setup_config(window, cx),
            locale_selector,
            source_language_selector: Self::setup_source_language_selector(window, cx),
            target_language_selector: Self::setup_target_language_selector(window, cx),
            menu_bar: AppMenuBar::new(cx),
            input_editor,
            output_editor,
            generate: None,
            ollama_version: None,
            models: Vec::new(),
        }
    }

    fn check_ollama_version(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async |this, cx| {
            loop {
                let version = ollama::version().await.ok();

                this.update(cx, |this, cx| {
                    this.ollama_version = version;
                    cx.notify();
                })
                .ok();

                cx.background_executor().timer(Duration::from_mins(5)).await;
            }
        })
        .detach();
    }

    fn list_models(&mut self, cx: &mut Context<Self>) {
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
                        if let Some(model) = this.models.first()
                            && this.config.read(cx).model().is_none()
                        {
                            this.config.update(cx, |this, cx| this.set_model(model, cx))
                        }
                        cx.notify();
                    })
                    .ok();
                }

                cx.background_executor().timer(Duration::from_mins(5)).await;
            }
        })
        .detach();
    }

    fn model_menu(&self, cx: &App) -> Menu {
        Menu {
            name: t!("model").into(),
            items: self
                .models
                .iter()
                .map(|model| {
                    MenuItem::action(model, ChangeModel::new(model))
                        .checked(self.config.read(cx).model() == Some(model))
                })
                .collect::<Vec<_>>(),
        }
    }

    fn help_menu() -> Menu {
        Menu {
            name: t!("help").into(),
            items: vec![
                MenuItem::action(t!("repository"), Repository),
                MenuItem::Separator,
                MenuItem::action(t!("about"), About),
            ],
        }
    }

    fn update_menu_bar(&mut self, cx: &mut Context<Self>) {
        cx.set_menus(vec![self.model_menu(cx), Self::help_menu()]);

        self.menu_bar.update(cx, |menu_bar, cx| {
            menu_bar.reload(cx);
        });
    }

    fn setup_source_language_selector(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<LanguageSelector> {
        let source_language_selector = cx.new(|cx| LanguageSelector::new(window, cx));

        cx.subscribe(&source_language_selector, |this, _, event, cx| {
            if let Some(language) = event.value() {
                this.config.update(cx, |this, cx| {
                    this.set_source_language(language.code, cx);
                })
            }
        })
        .detach();

        source_language_selector
    }

    fn setup_target_language_selector(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<LanguageSelector> {
        let target_language_selector = cx.new(|cx| LanguageSelector::new(window, cx));

        cx.subscribe(&target_language_selector, |this, _, event, cx| {
            if let Some(language) = event.value() {
                this.config.update(cx, |this, cx| {
                    this.set_target_language(language.code, cx);
                })
            }
        })
        .detach();

        target_language_selector
    }

    fn setup_config(window: &mut Window, cx: &mut Context<Self>) -> Entity<Config> {
        let config = cx.new(|_| Config::load("translate-gemma-desktop"));

        cx.observe_new(|this: &mut Self, window, cx| {
            this.check_ollama_version(cx);
            this.list_models(cx);

            let source_language_selector = this.source_language_selector.clone();
            let target_language_selector = this.target_language_selector.clone();

            this.config.update(cx, |this, cx| {
                this.init(cx);

                // Set the initial language for language selectors.
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

        cx.observe_self(|this, cx| {
            this.update_menu_bar(cx);
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
                cx.notify();
                window.refresh();
            }
            ConfigEvent::SourceLanguageChange(language) if language.is_some() => {
                this.translate(window, cx)
            }
            ConfigEvent::TargetLanguageChange(language) if language.is_some() => {
                this.translate(window, cx);
            }
            ConfigEvent::SwapLanguages {
                source_language,
                target_language,
            } => {
                if let (Some(source_language), Some(target_language)) =
                    (source_language, target_language)
                {
                    this.source_language_selector.update(cx, |this, cx| {
                        this.reset_state(window, cx);
                        this.set_selected_language(source_language, window, cx);
                    });

                    this.target_language_selector.update(cx, |this, cx| {
                        this.reset_state(window, cx);
                        this.set_selected_language(target_language, window, cx);
                    });

                    this.translate(window, cx);
                }
            }
            ConfigEvent::ModelChange => {
                this.translate(window, cx);
                cx.notify();
            }
            _ => {}
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

        if let (Some(source_language), Some(target_language)) = (source_language, target_language)
            && !self.input_editor.read(cx).is_empty(cx)
        {
            Some(Prompt::new(
                source_language,
                target_language,
                self.input_editor.read(cx).text(cx),
            ))
        } else {
            None
        }
    }

    fn translate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(model) = self.config.read(cx).model().cloned()
            && let Some(prompt) = self.prompt(cx)
        {
            let output_editor = self.output_editor.clone();

            let has_active_task = self.generate.as_ref().is_some_and(|task| !task.is_ready());
            self.generate = None;

            self.generate = Some(cx.spawn_in(window, async move |_, window| {
                if has_active_task {
                    output_editor.update_in(window, |this, window, cx| {
                        this.wait_for_input(window, cx);
                    })?;

                    window
                        .background_executor()
                        .timer(Duration::from_millis(500))
                        .await;

                    output_editor.update_in(window, |this, window, cx| {
                        this.translate_in_progress(window, cx);
                    })?;
                }

                let req = GenerateRequest::builder()
                    .model(model)
                    .stream(true)
                    .prompt(prompt.to_string())
                    .build();

                let mut result = generate(req).await?;

                if let Some(item) = result.next().await {
                    let response = item?.response;

                    output_editor.update_in(window, |this, window, cx| {
                        this.reset(window, cx);
                        this.append(response, window, cx);
                    })?;
                }

                while let Some(item) = result.next().await {
                    let response = item?.response;

                    output_editor.update_in(window, |this, window, cx| {
                        this.append(response, window, cx);
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

    fn on_action_change_model(
        &mut self,
        change_model: &ChangeModel,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.config.update(cx, |this, cx| {
            this.set_model(&change_model.name, cx);
        });
    }

    fn on_action_repository(&mut self, _: &Repository, _: &mut Window, cx: &mut Context<Self>) {
        cx.open_url("https://github.com/fhluo/translate-gemma-desktop")
    }

    fn on_action_about(&mut self, _: &About, window: &mut Window, cx: &mut Context<Self>) {
        open_about_dialog(window, cx);
    }

    fn on_click_swap_languages(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.config.update(cx, |this, cx| {
            this.swap_languages(cx);
        })
    }
}

impl Render for TranslateApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let dialog_layer = Root::render_dialog_layer(window, cx);

        div()
            .on_action(cx.listener(Self::on_action_repository))
            .on_action(cx.listener(Self::on_action_about))
            .on_action(cx.listener(Self::on_action_change_model))
            .on_action(cx.listener(Self::on_action_change_locale))
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(
                TitleBar::new()
                    .items_center()
                    .child(self.menu_bar.clone())
                    .child(
                        div().flex().flex_row().flex_1().child(
                            div()
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
                    .grid()
                    .grid_cols(2)
                    .p_3()
                    .gap_3()
                    .child(self.input_editor.clone())
                    .child(self.output_editor.clone()),
            )
            .child(StatusBar::new(self.ollama_version.clone()))
            .children(dialog_layer)
    }
}

fn main() -> anyhow::Result<()> {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        let bounds = Bounds::centered(None, size(px(1000.), px(625.)), cx);

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
