use gpui::{div, prelude::*, App, IntoElement, SharedString, Window};
use gpui_component::description_list::DescriptionList;
use gpui_component::label::Label;
use gpui_component::link::Link;
use gpui_component::{ActiveTheme, Icon, IconName, StyledExt, WindowExt};
use std::fmt::Display;
use std::path::PathBuf;

#[derive(IntoElement)]
struct Title(SharedString);

impl RenderOnce for Title {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .gap_3()
            .items_center()
            .child(
                Icon::new(IconName::CircleX)
                    .text_color(cx.theme().red)
                    .size_6(),
            )
            .child(Label::new(self.0).font_semibold().text_lg())
    }
}

pub fn show_dialog<F, E>(
    title: impl Into<SharedString>,
    content_builder: F,
    window: &mut Window,
    cx: &mut App,
) where
    F: Fn() -> E + 'static,
    E: IntoElement,
{
    let title = title.into();
    window.open_dialog(cx, move |dialog, _, _| {
        dialog
            .alert()
            .title(Title(title.clone()))
            .child(content_builder())
    });
}

#[allow(dead_code)]
pub fn show_error(message: impl Into<SharedString>, window: &mut Window, cx: &mut App) {
    let message = message.into();
    show_dialog(t!("error"), move || Label::new(message.clone()), window, cx);
}

pub fn show_io_error(
    title: impl Into<SharedString>,
    path: impl Into<PathBuf>,
    error: impl Display,
    window: &mut Window,
    cx: &mut App,
) {
    let title = title.into();
    let path = path.into();
    let error = error.to_string();

    show_dialog(
        title,
        move || {
            let path = path.clone();
            DescriptionList::vertical()
                .columns(1)
                .bordered(false)
                .item(
                    t!("path").to_string(),
                    Link::new("path")
                        .text_sm()
                        .child(path.display().to_string())
                        .on_click(move |_, _, cx| cx.reveal_path(&path))
                        .into_any_element(),
                    1,
                )
                .item(
                    t!("error").to_string(),
                    Label::new(error.clone()).text_sm().into_any_element(),
                    1,
                )
        },
        window,
        cx,
    );
}
