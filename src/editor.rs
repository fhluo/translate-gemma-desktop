use gpui::{
    div, prelude::*, App, Entity, FocusHandle, Focusable, IntoElement, Window,
};
use gpui_component::gray_300;
use gpui_component::input::{Input, InputState};

#[derive(IntoElement)]
pub struct Editor {
    state: Entity<InputState>,
}

impl Focusable for Editor {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}

impl Editor {
    pub fn new(state: &Entity<InputState>) -> Editor {
        Editor {
            state: state.clone(),
        }
    }
}

impl RenderOnce for Editor {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().flex_1().flex().flex_col().child(
            Input::new(&self.state)
                .size_full()
                .focus_bordered(false)
                .when(self.focus_handle(cx).is_focused(window), |this| {
                    this.shadow_sm().border_1().border_color(gray_300())
                }),
        )
    }
}
