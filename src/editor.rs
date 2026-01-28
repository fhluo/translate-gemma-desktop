use gpui::{
    div, prelude::*, transparent_white, white, App, ClipboardItem, ElementId, Entity,
    FocusHandle, Focusable, IntoElement, SharedString, Window,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};
use gpui_component::{gray_300, gray_500, ActiveTheme, IconName};
use std::sync::Arc;

pub trait InputStateEntityExt {
    fn is_empty(&self, cx: &App) -> bool;
    fn text(&self, cx: &App) -> SharedString;
}

impl InputStateEntityExt for Entity<InputState> {
    fn is_empty(&self, cx: &App) -> bool {
        self.read(cx).text().len() == 0
    }

    fn text(&self, cx: &App) -> SharedString {
        self.read(cx).value()
    }
}

#[derive(IntoElement)]
pub struct Editor {
    id: ElementId,
    state: Entity<InputState>,
}

impl Focusable for Editor {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}

impl Editor {
    pub fn new(id: impl Into<ElementId>, state: &Entity<InputState>) -> Editor {
        Editor {
            id: id.into(),
            state: state.clone(),
        }
    }
}

impl RenderOnce for Editor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .relative()
            .flex_1()
            .flex()
            .flex_col()
            .border_1()
            .border_color(cx.theme().border)
            .shadow_xs()
            .rounded_xl()
            .bg(white())
            .when(self.focus_handle(cx).is_focused(window), |this| {
                this.shadow_sm().border_1().border_color(gray_300())
            })
            .child(
                Input::new(&self.state)
                    .size_full()
                    .bordered(false)
                    .rounded_xl(),
            )
            .child(
                div()
                    .bg(transparent_white())
                    .absolute()
                    .bottom_0()
                    .flex()
                    .flex_row()
                    .w_full()
                    .px_3()
                    .py_1()
                    .rounded_b_xl()
                    .child(div().ml_auto().h_full().flex().flex_row().when(
                        !self.state.is_empty(cx),
                        |this| {
                            this.child(
                                Button::new(ElementId::NamedChild(
                                    Arc::new(self.id),
                                    "copy".into(),
                                ))
                                .icon(IconName::Copy)
                                .text_color(gray_500())
                                .ghost()
                                .tooltip(t!("copy"))
                                .on_click(move |_, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(
                                        self.state.text(cx).to_string(),
                                    ));
                                }),
                            )
                        },
                    )),
            )
    }
}
