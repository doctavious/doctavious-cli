use crate::commands::title_string;
use crate::constants::{DEFAULT_TIL_DIR, DEFAULT_TIL_TEMPLATE_PATH};
use crate::doctavious_error::Result as DoctaviousResult;
use crate::markup_format::{MarkupFormat, MARKUP_FORMAT_EXTENSIONS};
use crate::settings::{load_settings, persist_settings, TilSettings, SETTINGS};
use crate::{edit, init_dir};
use chrono::{DateTime, Utc};
use clap::Parser;
use std::collections::{BTreeMap};
use std::fs::File;
use std::io::{BufReader};
use std::path::Path;
use std::{fs};
use std::str::FromStr;
use walkdir::{DirEntry, WalkDir};
use crate::commands::design_decisions::{get_template_content};
use crate::templates::{TemplateContext, Templates};
use serde::{Serialize};
use crate::files::{friendly_filename, sanitize};

#[derive(Parser, Debug)]
#[command(about = "Gathers Today I Learned (TIL) management commands")]
pub(crate) struct Til {
    #[command(subcommand)]
    pub til_command: TilCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum TilCommand {
    Init(InitTil),
    New(NewTil),
    List(ListTils),
    Readme(BuildTilReadMe),
}

#[derive(Parser, Debug)]
#[command(about = "Init TIL")]
pub(crate) struct InitTil {
    #[arg(long, short, help = "Directory of TILs")]
    pub directory: Option<String>,

    // TODO: path to readme template or template string. two fields? one and we determine if its a path?
    // what do others do? Terraform has `var` and `var-file`

    #[arg(
        // value_enum,
        long,
        short,
        default_value = MarkupFormat::default().extension(),
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
        help = "Extension that should be used"
    )]
    pub extension: MarkupFormat,
}

#[derive(Parser, Debug)]
#[command(about = "New TIL")]
pub(crate) struct NewTil {
    #[arg(
        short,
        long,
        help = "TIL category. Represents the directory to place TIL entry under"
    )]
    pub category: String,

    #[arg(long, short, help = "title of the TIL entry")]
    pub title: String,

    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    #[arg(
        short = 'T',
        long,
        help = "Additional tags associated with the TIL entry"
    )]
    pub tags: Option<Vec<String>>,

    #[arg(
        long,
        short,
        help = "File name that should be used. If extension is included will take precedence over \
                extension argument and configuration file."
    )]
    pub file_name: Option<String>,

    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        value_parser,
        help = "Extension that should be used. This overrides value from configuration file."
    )]
    pub extension: Option<MarkupFormat>,

    // TODO: should this also be a setting in TilSettings?
    #[arg(
        short,
        long,
        help = "Whether to build_mod a README after a new TIL is added"
    )]
    pub readme: bool,
}

#[derive(Parser, Debug)]
#[command(about = "List TILs")]
pub(crate) struct ListTils {}

#[derive(Parser, Debug)]
#[command(about = "Build TIL ReadMe")]
pub(crate) struct BuildTilReadMe {
    #[arg(long, short, help = "Directory where TILs are stored")]
    pub directory: Option<String>,

    // TOOD: optional path to template.


    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        value_parser,
        help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,
}

#[derive(Clone, Debug, Serialize,)]
struct TilEntry {
    topic: String,
    title: String,
    file_name: String,
    date: DateTime<Utc>,
}

pub(crate) fn init_til(
    directory: Option<String>,
    extension: MarkupFormat,
) -> DoctaviousResult<()> {
    let mut settings = match load_settings() {
        Ok(settings) => settings,
        Err(_) => Default::default(),
    };

    let dir = match directory {
        None => DEFAULT_TIL_DIR,
        Some(ref d) => d,
    };

    let til_settings = TilSettings {
        dir: Some(dir.to_string()),
        template_extension: Some(extension),
    };
    settings.til_settings = Some(til_settings);

    persist_settings(settings)?;
    init_dir(dir)?;

    return Ok(());
}

pub(crate) fn new_til(
    title: String,
    category: String,
    tags: Option<Vec<String>>,
    file_name: Option<String>,
    markup_format: MarkupFormat,
    readme: bool,
    dir: &str,
) -> DoctaviousResult<()> {
    // https://stackoverflow.com/questions/7406102/create-sane-safe-filename-from-any-unsafe-string
    // https://docs.rs/sanitize-filename/latest/sanitize_filename/
    // TODO: convert to a better file name
    // spaces to hyphens
    // special characters?
    let file_name = if let Some(file_name) = file_name {
        file_name.trim().to_string()
    } else {
        friendly_filename(&title)
    };

    let path = Path::new(dir)
        .join(category)
        .join(file_name)
        .with_extension(markup_format.extension());

    if path.exists() {
        // TODO: this should return the error
        eprintln!("File {} already exists", path.to_string_lossy());
    } else {
        let leading_char = markup_format.leading_header_character();

        let mut starting_content = format!("{} {}\n", leading_char, title);
        if tags.is_some() {
            starting_content.push_str("\ntags: ");
            starting_content.push_str(tags.unwrap().join(" ").as_str());
        }

        let edited = edit::edit(&starting_content)?;

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, edited)?;

        if readme {
            build_til_readme(&dir, markup_format.extension())?;
        }
    }

    return Ok(());
}

// TODO: this should just build_mod the content and return and not write
pub(crate) fn build_til_readme(dir: &str, readme_extension: &str) -> DoctaviousResult<String> {
    let mut all_tils: BTreeMap<String, Vec<TilEntry>> = BTreeMap::new();
    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_entry(|e| !is_hidden(e))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        // skip files that are under til dir
        if Path::new(dir) == entry.path().parent().unwrap() {
            continue;
        }

        // TODO: handle unwraps better
        let topic = entry
            .path()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        if !all_tils.contains_key(&topic) {
            // TODO: is there a way to avoid this clone?
            all_tils.insert(topic.clone(), Vec::new());
        }

        let file_name =
            entry.path().file_name().unwrap().to_str().unwrap().to_string();
        let markup_format = MarkupFormat::from_str(
            entry.path().extension().unwrap().to_str().unwrap()
        ).unwrap();
        let file = match File::open(&entry.path()) {
            Ok(file) => file,
            Err(_) => panic!("Unable to read title from {:?}", entry.path()),
        };

        let buffer = BufReader::new(file);
        // TODO: should this use extension to get title? Would allow for users to mix/match file types
        let title = title_string(buffer, markup_format);

        all_tils.get_mut(&topic).unwrap().push(TilEntry {
            topic,
            title,
            file_name,
            date: DateTime::from(entry.metadata()?.created()?),
        });
    }

    let mut til_count = 0;
    for topic_tils in all_tils.values() {
        til_count += topic_tils.len();
    }

    let template = get_template_content(&dir, readme_extension, DEFAULT_TIL_TEMPLATE_PATH);
    let mut context = TemplateContext::new();
    context.insert("categories_count", &all_tils.keys().len());
    context.insert("til_count", &til_count);
    context.insert("tils", &all_tils);

    let rendered = Templates::one_off(template.as_str(), &context, false)?;
    return Ok(rendered);
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, tempfile, NamedTempFile};
    use crate::build_til_readme;
    use crate::markup_format::MarkupFormat::Markdown;

    #[test]
    fn markdown_til() {
        let dir = tempdir().unwrap();

        let r = build_til_readme("./docs/til/", Markdown.extension());
        match r {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }

    }

    #[test]
    fn asciidoc_til() {}
}
