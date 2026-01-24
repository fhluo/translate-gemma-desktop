mod assets;
mod language;
mod language_selector;
mod prompt;

use crate::assets::Assets;
use crate::language_selector::LanguageSelector;
use gpui::{
    div, prelude::*, px, size, Application, Bounds, Entity, Window, WindowBounds, WindowOptions,
};
use gpui_component::{Root, TitleBar};
use icu_locale::locale;

struct TranslateApp {
    source_language_selector: Entity<LanguageSelector>,
    target_language_selector: Entity<LanguageSelector>,
}

impl TranslateApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let source_language_selector =
            cx.new(|cx| LanguageSelector::new(locale!("zh-Hans"), window, cx));

        let target_language_selector =
            cx.new(|cx| LanguageSelector::new(locale!("zh-Hans"), window, cx));

        TranslateApp {
            source_language_selector,
            target_language_selector,
        }
    }
}

impl Render for TranslateApp {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h_full()
            .flex()
            .flex_col()
            .child(TitleBar::new())
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
