use crate::language::{Language, LANGUAGES};
use gpui::{prelude::*, AppContext, Entity, SharedString, Window};
use gpui_component::select::{SearchableVec, Select, SelectItem, SelectState};
use gpui_component::IconName;
use icu_experimental::displaynames::LocaleDisplayNamesFormatter;
use icu_locale::Locale;

#[derive(Debug, Clone)]
pub struct LanguageItem {
    language: Language,
    display_name: SharedString,
}

impl LanguageItem {
    pub fn new(language: Language, display_name: impl Into<SharedString>) -> Self {
        LanguageItem {
            language,
            display_name: display_name.into(),
        }
    }
}

impl SelectItem for LanguageItem {
    type Value = Language;

    fn title(&self) -> SharedString {
        self.display_name.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.language
    }

    fn matches(&self, query: &str) -> bool {
        let query = query.to_lowercase();

        self.display_name.to_lowercase().contains(&query)
            || self.language.name.to_lowercase().contains(&query)
            || self.language.code.to_lowercase().contains(&query)
    }
}

pub struct LanguageSelector {
    pub state: Entity<SelectState<SearchableVec<LanguageItem>>>,
}

impl LanguageSelector {
    pub fn new(locale: Locale, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let display_name = LocaleDisplayNamesFormatter::try_new(locale.into(), Default::default())
            .expect("failed to load compiled data");

        let items = LANGUAGES
            .iter()
            .cloned()
            .map(|lang| {
                let locale = lang.code.parse::<Locale>().expect("failed to parse locale");

                LanguageItem::new(lang, display_name.of(&locale).into_owned())
            })
            .collect::<Vec<_>>();

        let items = SearchableVec::new(items);

        let state = cx.new(|cx| SelectState::new(items, None, window, cx).searchable(true));

        LanguageSelector { state }
    }
}

impl Render for LanguageSelector {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Select::new(&self.state).icon(IconName::Search)
    }
}
