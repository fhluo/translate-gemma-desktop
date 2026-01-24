#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[macro_use]
extern crate rust_i18n;

mod assets;
mod language;
mod language_selector;
mod locale_selector;
mod prompt;

use crate::assets::Assets;
use crate::language_selector::LanguageSelector;
use crate::locale_selector::{ChangeLocale, LocaleSelector};
use gpui::{
    div, prelude::*, px, size, Application, Bounds, Entity, FocusHandle, Focusable,
    Window, WindowBounds, WindowOptions,
};
use gpui_component::{Root, TitleBar};
use icu_locale::locale;
use rust_i18n::set_locale;

i18n!("locales", fallback = "en");

struct TranslateApp {
    locale_selector: Entity<LocaleSelector>,

    source_language_selector: Entity<LanguageSelector>,
    target_language_selector: Entity<LanguageSelector>,

    focus_handle: FocusHandle,
}

impl TranslateApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let locale_selector = cx.new(|_| LocaleSelector::new(focus_handle.clone()));

        let source_language_selector =
            cx.new(|cx| LanguageSelector::new(locale!("zh-Hans"), window, cx));

        let target_language_selector =
            cx.new(|cx| LanguageSelector::new(locale!("zh-Hans"), window, cx));

        TranslateApp {
            locale_selector,
            source_language_selector,
            target_language_selector,
            focus_handle,
        }
    }

    fn on_action_change_locale(
        &mut self,
        ChangeLocale(locale): &ChangeLocale,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.locale_selector.update(cx, |_, _| {
            set_locale(locale);
            window.refresh();
        });
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
                    div()
                        .track_focus(&self.focus_handle)
                        .on_action(cx.listener(Self::on_action_change_locale))
                        .flex()
                        .flex_row()
                        .flex_1()
                        .child(
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
                    .h_full()
                    .flex()
                    .flex_row()
                    .child(self.source_language_selector.clone())
                    .child(self.target_language_selector.clone()),
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
