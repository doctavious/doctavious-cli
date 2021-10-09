use crate::utils::parse_enum;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use clap::arg_enum;
use tera::{
    Context as TeraContext,
    Result as TeraResult,
    Tera,
    Value,
};
use crate::doctavious_error::{DoctaviousError, Result as DoctavousResult, EnumError};
use csv::ReaderBuilder;
use log::Record;
use indexmap::map::IndexMap;

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
#[non_exhaustive]
pub enum TemplateExtension {
    Markdown,
    Asciidoc,
    // TODO: Other(str)?
}

impl TemplateExtension {

    pub(crate) fn variants() -> [&'static str; 2] {
        ["adoc", "md"]
    }


// TODO: can also access fields like the following. Make sure to include in docs
// {%- for row in data %}
// | {{- row['status'] }} | {{ row['RFD'] }} |
// {%- endfor -%}

    pub(crate) fn toc_template(self) ->  &'static str {
        return match self {
            TemplateExtension::Markdown => {
r#"{# snippet::markdown_toc #}
{% if headers -%}
  | {{ headers | join(sep=" | ") }}
|
    {%- for i in range(end=headers|length) -%}
        --- |
    {%- endfor -%}
{%- endif -%}
{% for row in data %}
|
    {%- for key, value in row %}
        {{- value }} |
    {%- endfor -%}
{%- endfor -%}
{# end::markdown_toc #}"#
            }
            TemplateExtension::Asciidoc => {
r#"{# snippet::asciidoc_toc #}
|===
{% if headers -%}
  | {{ headers | join(sep=" | ") }}
{%- endif %}
{%- for row in data %}
{% for key, value in row %}
| {{ value }}
{%- endfor -%}
{%- endfor -%}
{# end::asciidoc_toc #}"#
            }
        }
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
                eprintln!("Error when parsing {}, fallback to default settings. Error: {:?}\n", s, e);
                TemplateExtension::default()
            }
        };
        Ok(extension)
    }
}

pub(crate) fn parse_template_extension(
    src: &str,
) -> Result<TemplateExtension, EnumError> {
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

/// Wrapper for [`Tera`].
#[derive(Debug)]
pub struct Templates {
    tera: Tera,
}

impl Templates {
    /// Constructs a new instance.
    pub fn new(template: String) -> DoctavousResult<Self> {
        let mut tera = Tera::default();
        tera.add_raw_template("template", &template)?;
        tera.register_filter("upper_first", Self::upper_first_filter);
        Ok(Self { tera })
    }

    /// Filter for making the first character of a string uppercase.
    fn upper_first_filter(
        value: &Value,
        _: &HashMap<String, Value>,
    ) -> TeraResult<Value> {
        let mut s =
            tera::try_get_value!("upper_first_filter", "value", String, value);
        let mut c = s.chars();
        s = match c.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        };
        Ok(tera::to_value(&s)?)
    }

    // TODO: render method
    /// Renders the template.
    pub fn render<S>(&self, release: S) -> DoctavousResult<String>
        where
            S: Serialize,
    {
        Ok(self
            .tera
            .render("template", &TeraContext::from_serialize(release)?)?)
    }
}

// toc from Vec<&str> list of content
// toc from Vec<PathBuf> list of files
// toc from list<string> (headers) and Vec<HashMap> (data)
pub(crate) fn render_toc(path: PathBuf, template: &str) -> String {
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path).unwrap();

    let mut context = TeraContext::new();

    let headers = rdr.headers().unwrap().clone();
    let headers_vec: Vec<String> = headers.deserialize(None).unwrap();
    context.insert("headers", &headers_vec);

    let mut output: Vec<IndexMap<String,String>> = Vec::new();
    for record in rdr.records() {
        // let record: IndexMap<String,Option<String>> = row.unwrap().deserialize(Some(&headers)).unwrap();
        // output.push(record);
        // let record: Record = row.unwrap().deserialize(Some(&headers)).unwrap();
        let mut map: IndexMap<String, String> = IndexMap::new();
        for row in record.iter() {
            for (pos, field) in row.into_iter().enumerate() {
                println!("{} / {}", pos, field);
                map.insert(headers_vec.get(pos).unwrap().to_string(), field.to_string());
            }
        }
        output.push(map);
    }
    context.insert("data", &output);
    println!("{:?}", &output);
    println!("{:?}", &context.clone().into_json());

    return Tera::one_off(template, &context, false).unwrap();
}


// TODO: tests
#[cfg(test)]
mod tests {
    use std::{fs, env};
    use std::path::Path;
    use crate::output::Output;
    use crate::templates::TemplateExtension;

    // TODO: invalid template should return valid error

    #[test]
    fn markdown_toc() {
        let toc = super::render_toc(
            Path::new("./tests/resources/sample_csv.csv").to_path_buf(),
            TemplateExtension::Markdown.toc_template()
        );
        println!("{}", toc);
    }

    #[test]
    fn asciidoc_toc() {
        let toc = super::render_toc(
            Path::new("./tests/resources/sample_csv.csv").to_path_buf(),
            TemplateExtension::Asciidoc.toc_template()
        );
        println!("{}", toc);
    }
}
