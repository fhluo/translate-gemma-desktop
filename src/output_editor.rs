use gpui::{div, prelude::*, Context, Entity, EntityInputHandler, Focusable, Window};
use gpui_component::gray_300;
use gpui_component::input::{Input, InputState};

pub struct OutputEditor {
    state: Entity<InputState>,
}

impl OutputEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> OutputEditor {
        let state = cx.new(|cx| InputState::new(window, cx).multi_line(true));

        OutputEditor { state }
    }

    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |this, cx| {
            this.set_value("", window, cx);
            this.set_placeholder("", window, cx);
        });
    }

    pub fn wait_for_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |this, cx| {
            this.set_value("", window, cx);
            this.set_placeholder(t!("translate.wait-input"), window, cx);
        });
    }

    pub fn translate_in_progress(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |this, cx| {
            this.set_value("", window, cx);
            this.set_placeholder(t!("translate.in-progress"), window, cx);
        });
    }

    pub fn append(&mut self, text: impl AsRef<str>, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |this, cx| {
            let end = this.text().len_utf16();
            this.replace_text_in_range(Some(end..end), text.as_ref(), window, cx);
        });
    }
}

impl Render for OutputEditor {
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
