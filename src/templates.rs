use crate::utils::parse_enum;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use clap::arg_enum;

lazy_static! {
    pub static ref TEMPLATE_EXTENSIONS: HashMap<&'static str, TemplateExtension> = {
        let mut map = HashMap::new();
        map.insert("md", TemplateExtension::Markdown);
        map.insert("adoc", TemplateExtension::Asciidoc);
        map
    };
}

// TODO: is there a better name for this?
// TODO: can these enums hold other attributes? extension value (adoc / md), leading char (= / #), etc
#[derive(Clone, Copy, Debug)]
pub enum TemplateExtension {
    Markdown,
    Asciidoc,
}

impl TemplateExtension {

    pub(crate) fn variants() -> [&'static str; 2] {
        ["adoc", "md"]
    }

}

impl Default for TemplateExtension {
    fn default() -> Self {
        TemplateExtension::Markdown
    }
}

impl Display for TemplateExtension {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            TemplateExtension::Markdown => write!(f, "md"),
            TemplateExtension::Asciidoc => write!(f, "adoc"),
        }
    }
}

impl Serialize for TemplateExtension {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match *self {
            TemplateExtension::Markdown => "md",
            TemplateExtension::Asciidoc => "adoc",
        };

        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TemplateExtension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let extension = match parse_template_extension(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error when parsing {}, fallback to default settings. Error: {}\n", s, e);
                TemplateExtension::default()
            }
        };
        Ok(extension)
    }
}

pub(crate) fn parse_template_extension(
    src: &str,
) -> Result<TemplateExtension, String> {
    parse_enum(&TEMPLATE_EXTENSIONS, src)
}

pub(crate) fn get_template(
    dir: &str,
    extension: TemplateExtension,
    default_template_path: &str,
) -> PathBuf {
    let custom_template =
        Path::new(dir).join("template").with_extension(extension.to_string());

    let template = if custom_template.exists() {
        custom_template
    } else {
        Path::new(default_template_path)
            .with_extension(extension.to_string())
            .to_path_buf()
    };

    return template;
}
