use crate::markup_format::MarkupFormat::{Asciidoc, Markdown};
use crate::{parse_enum, EnumError};
use clap::ArgEnum;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::slice::Iter;

lazy_static! {
    pub static ref MARKUP_FORMAT_EXTENSIONS: HashMap<&'static str, MarkupFormat> = {
        let mut map = HashMap::new();
        for markup_format in MarkupFormat::iterator() {
            map.insert(markup_format.extension(), markup_format.to_owned());
        }
        map
    };
}

#[derive(ArgEnum, Clone, Copy, Debug)]
#[non_exhaustive]
pub enum MarkupFormat {
    Asciidoc,
    Markdown,
    // TODO: Other(str)?
}

impl MarkupFormat {
    pub(crate) fn iterator() -> Iter<'static, MarkupFormat> {
        return [Asciidoc, Markdown].iter();
    }

    pub(crate) fn extension(&self) -> &'static str {
        return match self {
            Asciidoc => "adoc",
            Markdown => "md",
        };
    }

    pub(crate) fn leading_header_character(&self) -> char {
        return match self {
            Asciidoc => '=',
            Markdown => '#',
        };
    }
}

impl Default for MarkupFormat {
    fn default() -> Self {
        Markdown
    }
}

impl Display for MarkupFormat {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.extension())
    }
}

impl Serialize for MarkupFormat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = self.extension();
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
