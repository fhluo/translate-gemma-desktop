use gpui::{div, prelude::*, App, IntoElement, Window};
use gpui_component::label::Label;
use gpui_component::{gray_400, gray_600, green_500, ActiveTheme};
use semver::Version;

#[derive(IntoElement)]
pub struct StatusBar {
    ollama_version: Option<Version>,
}

impl StatusBar {
    pub(crate) fn new(ollama_version: Option<Version>) -> Self {
        Self { ollama_version }
    }
}

impl RenderOnce for StatusBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .h_8()
            .w_full()
            .flex()
            .flex_row()
            .items_center()
            .px_3()
            .bg(cx.theme().title_bar)
            .border_t_1()
            .border_color(cx.theme().title_bar_border)
            .child(
                div()
                    .ml_auto()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_1()
                            .items_center()
                            .when_none(&self.ollama_version, |this| {
                                this.child(Label::new("•").text_lg().text_color(gray_400()))
                                    .child(Label::new("Ollama").text_xs().text_color(gray_600()))
                            })
                            .when_some(self.ollama_version, |this, version| {
                                this.child(Label::new("•").text_lg().text_color(green_500()))
                                    .child(
                                        Label::new(format!("Ollama {version}"))
                                            .text_xs()
                                            .text_color(gray_600()),
                                    )
                            }),
                    ),
            )
    }
}
