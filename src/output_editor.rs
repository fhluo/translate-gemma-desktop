use crate::editor::{Editor, InputStateEntityExt};
use gpui::{prelude::*, App, Context, Entity, EntityInputHandler, SharedString, Window};
use gpui_component::input::InputState;

pub struct OutputEditor {
    pub state: Entity<InputState>,
}

impl OutputEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> OutputEditor {
        let state = cx.new(|cx| InputState::new(window, cx).multi_line(true));

        OutputEditor { state }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self, cx: &App) -> bool {
        self.state.is_empty(cx)
    }

    pub fn text(&self, cx: &App) -> SharedString {
        self.state.text(cx)
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
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Editor::new("output-editor", &self.state)
    }
}
