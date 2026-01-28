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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex_1().flex().flex_col().child(
            Input::new(&self.state)
                .w_full()
                .h_full()
                .focus_bordered(false)
                .when(self.state.focus_handle(cx).is_focused(window), |this| {
                    this.shadow_sm().border_1().border_color(gray_300())
                }),
        )
    }
}
