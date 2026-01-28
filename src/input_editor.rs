use gpui::{div, prelude::*, App, Context, Entity, FocusHandle, Focusable, SharedString, Window};
use gpui_component::gray_300;
use gpui_component::input::{Input, InputState};

pub struct InputEditor {
    pub state: Entity<InputState>,
}

impl Focusable for InputEditor {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}

impl InputEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> InputEditor {
        let state = cx.new(|cx| InputState::new(window, cx).multi_line(true));

        cx.on_focus_lost(window, |this, window, cx| {
            this.state.update(cx, |this, cx| {
                this.focus(window, cx);
            });
        })
        .detach();

        InputEditor { state }
    }

    pub fn is_empty(&self, cx: &App) -> bool {
        self.state.read(cx).text().len() == 0
    }

    pub fn text(&self, cx: &App) -> SharedString {
        self.state.read(cx).value()
    }
}

impl Render for InputEditor {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Editor::new(&self.state)
    }
}
