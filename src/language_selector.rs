use crate::language::{Language, LANGUAGES};
use gpui::{prelude::*, App, AppContext, Entity, EventEmitter, SharedString, Window};
use gpui_component::select::{SearchableVec, Select, SelectEvent, SelectItem, SelectState};
use gpui_component::{IconName, IndexPath};
use icu_experimental::displaynames::LocaleDisplayNamesFormatter;
use icu_locale::{locale, Locale};

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

    pub fn all() -> Vec<LanguageItem> {
        let locale = rust_i18n::locale()
            .parse::<Locale>()
            .unwrap_or_else(|_| locale!("en"));

        let display_name = LocaleDisplayNamesFormatter::try_new(locale.into(), Default::default())
            .expect("failed to load compiled data");

        LANGUAGES
            .iter()
            .cloned()
            .map(|lang| {
                let locale = lang.code.parse::<Locale>().expect("failed to parse locale");

                LanguageItem::new(lang, display_name.of(&locale).into_owned())
            })
            .collect::<Vec<_>>()
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

#[derive(Debug, Clone)]
pub struct LanguageSelector {
    state: Entity<SelectState<SearchableVec<LanguageItem>>>,
}

impl LanguageSelector {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        LanguageSelector {
            state: Self::setup_state(window, cx),
        }
    }

    fn setup_state(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<SelectState<SearchableVec<LanguageItem>>> {
        let state = cx.new(|cx| {
            SelectState::new(SearchableVec::new(LanguageItem::all()), None, window, cx)
                .searchable(true)
        });

        cx.subscribe(
            &state,
            |_, _, event: &SelectEvent<SearchableVec<LanguageItem>>, cx| match event {
                SelectEvent::Confirm(lang) => {
                    cx.emit(LanguageSelectEvent(lang.map(|lang| lang.clone())))
                }
            },
        )
        .detach();

        state
    }

    pub fn reset_state(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.state = Self::setup_state(window, cx);
    }

    pub fn update_items(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |this, cx| {
            this.set_items(SearchableVec::new(LanguageItem::all()), window, cx)
        })
    }

    pub fn selected_language(&self, cx: &App) -> Option<Language> {
        self.state.read(cx).selected_value().cloned()
    }

    pub fn set_selected_language(
        &self,
        language_code: impl AsRef<str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let index = LANGUAGES
            .iter()
            .position(|item| item.code == language_code.as_ref())
            .map(IndexPath::new);

        self.state
            .update(cx, |state, cx| state.set_selected_index(index, window, cx));
    }
}

impl Render for LanguageSelector {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Select::new(&self.state).icon(IconName::Search)
    }
}

pub struct LanguageSelectEvent(Option<Language>);

impl LanguageSelectEvent {
    pub fn value(&self) -> Option<Language> {
        self.0
    }
}

impl EventEmitter<LanguageSelectEvent> for LanguageSelector {}
