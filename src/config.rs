use gpui::{Context, EventEmitter};
use icu_locale::fallback::{LocaleFallbackConfig, LocaleFallbackPriority};
use icu_locale::{locale, DataLocale, Locale, LocaleFallbacker};
use rust_i18n::set_locale;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(skip)]
    app_name: String,

    locale: Option<String>,
    source_language: Option<String>,
    target_language: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_name: env!("CARGO_PKG_NAME").to_string(),
            locale: None,
            source_language: Some("zh-Hans".to_owned()),
            target_language: Some("en".to_owned()),
        }
    }
}

impl Config {
    pub fn load(app_name: impl Into<String>) -> Config {
        let app_name = app_name.into();

        match confy::load::<Config>(&app_name, None) {
            Ok(mut config) => {
                config.app_name = app_name;
                config
            }
            Err(err) => {
                eprintln!("{err}");
                Default::default()
            }
        }
    }

    pub fn store(&self) {
        if let Err(err) = confy::store(&self.app_name, None, self) {
            eprintln!("{err}")
        }
    }

    pub fn init(&mut self, cx: &mut Context<Self>) {
        self.init_locale();
        cx.emit(ConfigEvent::LocaleChange)
    }

    fn get_locale(&self) -> Locale {
        if let Some(locale) = &self.locale
            && let Ok(locale) = locale.parse::<Locale>()
        {
            locale
        } else {
            sys_locale::get_locale()
                .unwrap_or_else(|| "en".to_string())
                .parse::<Locale>()
                .ok()
                .unwrap_or_else(|| locale!("en"))
        }
    }

    fn init_locale(&mut self) {
        let mut fallback_config = LocaleFallbackConfig::default();
        fallback_config.priority = LocaleFallbackPriority::Language;

        let mut fallback_iter = LocaleFallbacker::new()
            .for_config(fallback_config)
            .fallback_for(self.get_locale().into());

        let locales = available_locales!()
            .into_iter()
            .filter_map(|locale| locale.parse::<Locale>().map(DataLocale::from).ok())
            .collect::<Vec<_>>();

        let locale = loop {
            let locale = fallback_iter.get();
            if locale.is_unknown() {
                break locale!("en");
            }

            if locales.contains(locale) {
                break locale.into_locale();
            }

            fallback_iter.step();
        }
        .to_string();

        set_locale(&locale);
        self.locale = Some(locale);
    }

    pub fn set_locale(&mut self, locale: impl Into<String>, cx: &mut Context<Self>) {
        let locale = locale.into();

        set_locale(&locale);
        self.locale = Some(locale);

        cx.emit(ConfigEvent::LocaleChange);
    }

    pub fn get_source_language(&self) -> Option<&String> {
        self.source_language.as_ref()
    }

    pub fn set_source_language(&mut self, language: impl Into<String>, cx: &mut Context<Self>) {
        self.source_language = Some(language.into());

        cx.emit(ConfigEvent::SourceLanguageChange);
    }

    pub fn get_target_language(&self) -> Option<&String> {
        self.target_language.as_ref()
    }

    pub fn set_target_language(&mut self, language: impl Into<String>, cx: &mut Context<Self>) {
        self.target_language = Some(language.into());

        cx.emit(ConfigEvent::TargetLanguageChange);
    }
}

pub enum ConfigEvent {
    LocaleChange,
    SourceLanguageChange,
    TargetLanguageChange,
}

impl EventEmitter<ConfigEvent> for Config {}
