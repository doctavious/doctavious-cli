use crate::utils::parse_enum;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};

mod changelog;
mod commit;
mod release;
use crate::doctavious_error::EnumError;
use clap::{ArgEnum, PossibleValue};
use lazy_static::lazy_static;
use serde::de::Error;
use std::collections::HashMap;

lazy_static! {
    pub static ref STRIP_PARTS: HashMap<&'static str, StripParts> = {
        let mut map = HashMap::new();
        map.insert("header", StripParts::Header);
        map.insert("footer", StripParts::Footer);
        map.insert("all", StripParts::All);
        map
    };
}

/// Changelog configuration.
#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ChangelogConfig {
    /// Changelog header.
    pub header: Option<String>,
    /// Changelog body, template.
    pub body: String,
    /// Changelog footer.
    pub footer: Option<String>,
    /// Trim the template.
    pub trim: Option<bool>,
}

/// Git configuration
#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct GitConfig {
    /// Whether to enable conventional commits.
    pub conventional_commits: bool,
    /// Git commit parsers.
    pub commit_parsers: Option<Vec<CommitParser>>,
    /// Whether to filter out commits.
    pub filter_commits: Option<bool>,
    /// Blob pattern for git tags.
    pub tag_pattern: Option<String>,
    #[serde(with = "serde_regex", default)]
    /// Regex to skip matched tags.
    pub skip_tags: Option<Regex>,
}

/// Parser for grouping commits.
#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitParser {
    /// Regex for matching the commit message.
    #[serde(with = "serde_regex", default)]
    pub message: Option<Regex>,

    // TODO: add description
    /// Regex for matching the commit body.
    #[serde(with = "serde_regex", default)]
    pub body: Option<Regex>,
    /// Category of the commit.
    pub category: Option<String>,
    /// Whether to skip this commit category.
    pub skip: Option<bool>,
}

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum StripParts {
    Header,
    Footer,
    All,
}

impl StripParts {
    // TODO: certainly dont need both and probably dont need either.
    // verify help docs generated
    pub(crate) fn variants() -> [&'static str; 3] {
        ["header", "footer", "all"]
    }

    pub fn possible_values() -> impl Iterator<Item = PossibleValue<'static>> {
        StripParts::value_variants()
            .iter()
            .filter_map(ArgEnum::to_possible_value)
    }
}

impl Display for StripParts {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            StripParts::Header => write!(f, "header"),
            StripParts::Footer => write!(f, "footer"),
            StripParts::All => write!(f, "all"),
        }
    }
}

impl Serialize for StripParts {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match *self {
            StripParts::Header => "header",
            StripParts::Footer => "footer",
            StripParts::All => "all",
        };

        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for StripParts {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        return match parse_strip_parts(&s) {
            Ok(v) => Ok(v),
            Err(e) => {
                eprintln!("Error when parsing {}, fallback to default settings. Error: {}\n", s, e);
                // TODO: was having an issue with lifetimes / borrow with using variants.
                // find a better way to do this.
                Err(D::Error::unknown_field(
                    e.message.as_str(),
                    &["header", "footer", "all"],
                ))
            }
        };
    }
}

pub(crate) fn parse_strip_parts(src: &str) -> Result<StripParts, EnumError> {
    parse_enum(&STRIP_PARTS, src)
}
