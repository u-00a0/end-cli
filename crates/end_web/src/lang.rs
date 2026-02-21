use crate::{Error, Result};

#[derive(Debug, Clone, Copy)]
pub enum Lang {
    Zh,
    En,
}

impl Lang {
    pub(crate) fn parse(tag: &str) -> Result<Self> {
        match tag.trim().to_ascii_lowercase().as_str() {
            "zh" => Ok(Self::Zh),
            "en" => Ok(Self::En),
            value => Err(Error::UnknownLang {
                value: value.to_string(),
            }),
        }
    }

    pub(crate) fn tag(self) -> &'static str {
        match self {
            Self::Zh => "zh",
            Self::En => "en",
        }
    }
}
