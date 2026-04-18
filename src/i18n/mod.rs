// WebStack Deployer for Docker
// Copyright (c) 2026 Walter Nunez / Icaros Net S.A
// All Rights Reserved.
//
// Internationalization (i18n) module
// Supports: English, Spanish, French, Hindi, Chinese (Simplified)

use std::collections::BTreeMap;
use std::sync::{OnceLock, RwLock};

use serde::{Deserialize, Serialize};

type LocaleMap = BTreeMap<String, String>;

/// Supported languages.
///
/// The app defaults to English for new installs and configs that do not
/// define a language yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Language {
    #[default]
    En, // English
    Es, // Spanish (Espanol)
    Fr, // French (Francais)
    Hi, // Hindi
    Zh, // Chinese Simplified (Zhongwen)
}

impl Language {
    /// Native name for display in language selector.
    pub fn native_name(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Es => "Espanol",
            Self::Fr => "Francais",
            Self::Hi => "\u{0939}\u{093f}\u{0928}\u{094d}\u{0926}\u{0940}", // हिन्दी
            Self::Zh => "\u{4e2d}\u{6587}",                                 // 中文
        }
    }

    /// ISO 639-1 code.
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Fr => "fr",
            Self::Hi => "hi",
            Self::Zh => "zh",
        }
    }

    /// All available languages.
    pub fn all() -> &'static [Self] {
        &[Self::En, Self::Es, Self::Fr, Self::Hi, Self::Zh]
    }
}

const LOCALE_EN: &str = include_str!("locales/en.json");
const LOCALE_ES: &str = include_str!("locales/es.json");
const LOCALE_FR: &str = include_str!("locales/fr.json");
const LOCALE_HI: &str = include_str!("locales/hi.json");
const LOCALE_ZH: &str = include_str!("locales/zh.json");

#[derive(Debug, Clone)]
struct I18nState {
    language: Language,
    current: LocaleMap,
    fallback_en: LocaleMap,
}

impl I18nState {
    fn new(language: Language) -> Self {
        let fallback_en = parse_locale(LOCALE_EN, "en");
        let current = load_locale(language);

        Self {
            language,
            current,
            fallback_en,
        }
    }
}

static CURRENT: OnceLock<RwLock<I18nState>> = OnceLock::new();

fn locale_src(lang: Language) -> &'static str {
    match lang {
        Language::En => LOCALE_EN,
        Language::Es => LOCALE_ES,
        Language::Fr => LOCALE_FR,
        Language::Hi => LOCALE_HI,
        Language::Zh => LOCALE_ZH,
    }
}

fn parse_locale(json: &str, locale: &str) -> LocaleMap {
    match serde_json::from_str(json) {
        Ok(map) => map,
        Err(e) => {
            tracing::error!("Failed to parse locale {locale}: {e}");
            LocaleMap::new()
        }
    }
}

fn load_locale(lang: Language) -> LocaleMap {
    parse_locale(locale_src(lang), lang.code())
}

fn state_lock() -> &'static RwLock<I18nState> {
    CURRENT.get_or_init(|| RwLock::new(I18nState::new(Language::default())))
}

/// Initialize translations with the specified language.
pub fn init(lang: Language) {
    if let Ok(mut guard) = state_lock().write() {
        *guard = I18nState::new(lang);
    }
}

/// Change the current language at runtime.
pub fn set_language(lang: Language) {
    init(lang);
}

/// Returns the current active language.
pub fn current_language() -> Language {
    state_lock()
        .read()
        .map(|guard| guard.language)
        .unwrap_or_default()
}

/// Translate a key using the current language, then English fallback, then the key itself.
pub fn tr(key: &str) -> String {
    if let Ok(guard) = state_lock().read() {
        if let Some(value) = translate_from_maps(&guard.current, &guard.fallback_en, key) {
            return value.clone();
        }
    }

    key.to_string()
}

fn translate_from_maps<'a>(
    current: &'a LocaleMap,
    fallback_en: &'a LocaleMap,
    key: &str,
) -> Option<&'a String> {
    current.get(key).or_else(|| fallback_en.get(key))
}

/// Translate a key and replace `{placeholders}` with provided values.
pub fn trf(key: &str, vars: &[(&str, &str)]) -> String {
    let mut value = tr(key);
    for (name, replacement) in vars {
        let placeholder = format!("{{{name}}}");
        value = value.replace(&placeholder, replacement);
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn all_locales_parse() {
        for lang in Language::all() {
            let locale = load_locale(*lang);
            assert!(
                !locale.is_empty(),
                "locale {} should not be empty",
                lang.code()
            );
        }
    }

    #[test]
    fn english_locale_contains_all_non_english_keys() {
        let english_keys: BTreeSet<_> = load_locale(Language::En).into_keys().collect();

        for lang in Language::all() {
            if *lang == Language::En {
                continue;
            }

            let keys: BTreeSet<_> = load_locale(*lang).into_keys().collect();
            let extra_keys: Vec<_> = keys.difference(&english_keys).collect();
            assert!(
                extra_keys.is_empty(),
                "locale {} has keys missing in English: {extra_keys:?}",
                lang.code()
            );
        }
    }

    #[test]
    fn missing_current_language_key_falls_back_to_english() {
        let current = LocaleMap::new();
        let fallback_en =
            LocaleMap::from([("main_containers".to_string(), "Containers".to_string())]);

        assert_eq!(
            translate_from_maps(&current, &fallback_en, "main_containers").map(String::as_str),
            Some("Containers")
        );
        assert_eq!(translate_from_maps(&current, &fallback_en, "missing"), None);
    }
}
