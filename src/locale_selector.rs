use crate::assets::Icons;
use gpui::{
    Action, Context, Corner, FocusHandle, InteractiveElement, IntoElement, Render, Styled, Window,
};
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::DropdownMenu;
use gpui_component::{gray_500, Sizable};
use icu_experimental::displaynames::LocaleDisplayNamesFormatter;
use rust_i18n::{available_locales, t};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, Action)]
pub struct ChangeLocale(pub String);

#[derive(Debug, Clone)]
pub struct Locale {
    id: String,
    display_name: Option<String>,
}

impl Display for Locale {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name.as_ref().unwrap_or(&self.id))
    }
}

impl Locale {
    pub fn new(locale: impl Into<String>) -> Self {
        let locale = locale.into();

        Locale {
            display_name: {
                locale
                    .parse::<icu_locale::Locale>()
                    .ok()
                    .and_then(|locale| {
                        match LocaleDisplayNamesFormatter::try_new(
                            locale.clone().into(),
                            Default::default(),
                        ) {
                            Ok(display_name) => Some(display_name.of(&locale).into_owned()),
                            Err(err) => {
                                eprintln!(
                                    "failed to create locale display names formatter for {}: {}",
                                    locale, err
                                );
                                None
                            }
                        }
                    })
            },
            id: locale,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocaleSelector {
    locales: Vec<Locale>,
    focus_handle: FocusHandle,
}

impl LocaleSelector {
    pub fn new(focus_handle: FocusHandle) -> LocaleSelector {
        let locales = available_locales!()
            .into_iter()
            .map(|locale| Locale::new(locale))
            .collect::<Vec<_>>();

        LocaleSelector {
            locales,
            focus_handle,
        }
    }
}

impl Render for LocaleSelector {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Button::new("language-button")
            .small()
            .ghost()
            .icon(Icons::Languages)
            .text_color(gray_500())
            .tooltip(t!("language"))
            .dropdown_menu({
                let this = self.clone();

                move |mut menu, _, _| {
                    menu = menu.action_context(this.focus_handle.clone());
                    let selected = rust_i18n::locale().to_owned();

                    for locale in &this.locales {
                        menu = menu.menu_with_check(
                            locale.to_string(),
                            selected == locale.id,
                            Box::new(ChangeLocale(locale.id.clone())),
                        );
                    }

                    menu
                }
            })
            .anchor(Corner::TopRight)
    }
}
