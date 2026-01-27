use crate::language::Language;
use std::fmt::{Display, Formatter};
use std::mem;

#[derive(Debug, Clone)]
pub struct Prompt {
    pub source_language: Language,
    pub target_language: Language,
    pub text: String,
}

impl Prompt {
    pub fn new(
        source_language: Language,
        target_language: Language,
        text: impl Into<String>,
    ) -> Self {
        Prompt {
            source_language,
            target_language,
            text: text.into(),
        }
    }

    #[allow(dead_code)]
    /// Swaps languages.
    pub fn swap(mut self) -> Prompt {
        mem::swap(&mut self.source_language, &mut self.target_language);

        self
    }

    #[allow(dead_code)]
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();

        self
    }
}

impl Display for Prompt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let Prompt {
            source_language: source,
            target_language: target,
            text,
        } = self;

        write!(
            f,
            "You are a professional {} ({}) to {} ({}) translator. Your goal is to accurately convey the meaning and nuances of the original {0} text while adhering to {2} grammar, vocabulary, and cultural sensitivities. Produce only the {2} translation, without any additional explanations or commentary. Please translate the following {0} text into {2}:\n\n\n{text}",
            source.name, source.code, target.name, target.code,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::language::Language;
    use crate::prompt::Prompt;

    #[test]
    fn test_prompt() {
        let en = Language::new("en", "English");
        let zh = Language::new("zh-Hans", "Chinese");

        let prompt = Prompt::new(en, zh, "Hello, world!");

        assert_eq!(
            prompt.to_string(),
            "You are a professional English (en) to Chinese (zh-Hans) translator. Your goal is to accurately convey the meaning and nuances of the original English text while adhering to Chinese grammar, vocabulary, and cultural sensitivities. Produce only the Chinese translation, without any additional explanations or commentary. Please translate the following English text into Chinese:\n\n\nHello, world!"
        );

        assert_eq!(
            prompt.swap().text("你好，世界！").to_string(),
            "You are a professional Chinese (zh-Hans) to English (en) translator. Your goal is to accurately convey the meaning and nuances of the original Chinese text while adhering to English grammar, vocabulary, and cultural sensitivities. Produce only the English translation, without any additional explanations or commentary. Please translate the following Chinese text into English:\n\n\n你好，世界！"
        );
    }
}
