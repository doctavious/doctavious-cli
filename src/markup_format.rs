use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use crate::{EnumError, parse_enum};
use clap::ArgEnum;

lazy_static! {
    pub static ref MARKUP_FORMAT_EXTENSIONS: HashMap<&'static str, MarkupFormat> = {
        let mut map = HashMap::new();
        map.insert("md", MarkupFormat::Markdown);
        map.insert("adoc", MarkupFormat::Asciidoc);
        map
    };
}

// TODO: is there a better name for this?
// TODO: can these enums hold other attributes? extension value (adoc / md), leading char (= / #), etc
#[derive(ArgEnum, Clone, Copy, Debug)]
#[non_exhaustive]
pub enum MarkupFormat {
    Markdown,
    Asciidoc,
    // TODO: Other(str)?
}

impl MarkupFormat {

    pub(crate) fn variants() -> [&'static str; 2] {
        ["adoc", "md"]
    }

}

impl Default for MarkupFormat {
    fn default() -> Self {
        MarkupFormat::Markdown
    }
}

impl Display for MarkupFormat {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            MarkupFormat::Markdown => write!(f, "md"),
            MarkupFormat::Asciidoc => write!(f, "adoc"),
        }
    }
}

impl Serialize for MarkupFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = match *self {
            MarkupFormat::Markdown => "md",
            MarkupFormat::Asciidoc => "adoc",
        };

        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for MarkupFormat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let extension = match parse_markup_format_extension(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error when parsing {}, fallback to default settings. Error: {:?}\n", s, e);
                MarkupFormat::default()
            }
        };
        Ok(extension)
    }
}

pub(crate) fn parse_markup_format_extension(
    src: &str,
) -> Result<MarkupFormat, EnumError> {
    parse_enum(&MARKUP_FORMAT_EXTENSIONS, src)
}

pub(crate) fn get_leading_character(extension: MarkupFormat) -> char {
    return match extension {
        MarkupFormat::Markdown => '#',
        MarkupFormat::Asciidoc => '=',
    };
}
