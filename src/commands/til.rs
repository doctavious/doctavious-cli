use crate::commands::title_string;
use crate::constants::{DEFAULT_TIL_DIR, DEFAULT_TIL_TEMPLATE_PATH};
use crate::doctavious_error::Result as DoctaviousResult;
use crate::markup_format::{parse_markup_format_extension, MarkupFormat, MARKUP_FORMAT_EXTENSIONS};
use crate::settings::{load_settings, persist_settings, TilSettings, SETTINGS};
use crate::{edit, init_dir};
use chrono::{DateTime, Utc};
use clap::Parser;
use std::collections::{BTreeMap};
use std::fs::File;
use std::io::{BufReader};
use std::path::Path;
use std::{fs};
use walkdir::WalkDir;
use crate::commands::design_decisions::{get_template_content};
use crate::templates::{TemplateContext, Templates};
use serde::{Serialize};

#[derive(Parser, Debug)]
#[clap(about = "Gathers Today I Learned (TIL) management commands")]
pub(crate) struct Til {
    #[clap(subcommand)]
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
#[clap(about = "Init TIL")]
pub(crate) struct InitTil {
    #[clap(long, short, help = "Directory to store TILs")]
    pub directory: Option<String>,

    #[clap(
        arg_enum,
        long,
        short,
        default_value_t, parse(try_from_str = parse_markup_format_extension),
        help = "Extension that should be used"
    )]
    pub extension: MarkupFormat,
}

#[derive(Parser, Debug)]
#[clap(about = "New TIL")]
pub(crate) struct NewTil {
    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    // TODO: change to category
    #[clap(
        short,
        long,
        help = "TIL category. Represents the directory to place TIL entry under"
    )]
    pub category: String,

    #[clap(long, short, help = "title of the TIL entry")]
    pub title: String,

    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    #[clap(
        short = 'T',
        long,
        help = "Additional tags associated with the TIL entry"
    )]
    pub tags: Option<Vec<String>>,

    #[clap(
        long,
        short,
        possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        parse(try_from_str = parse_markup_format_extension),
        help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,

    // TODO: should this also be a setting in TilSettings?
    #[clap(
        short,
        long,
        help = "Whether to build a README after a new TIL is added"
    )]
    pub readme: bool,
}

#[derive(Parser, Debug)]
#[clap(about = "List TILs")]
pub(crate) struct ListTils {}

#[derive(Parser, Debug)]
#[clap(about = "Build TIL ReadMe")]
pub(crate) struct BuildTilReadMe {
    #[clap(
        long,
        short,
        possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        parse(try_from_str = parse_markup_format_extension),
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
    extension: MarkupFormat,
    readme: bool,
    dir: &str,
) -> DoctaviousResult<()> {
    let file_name = title.to_lowercase();
    let path = Path::new(dir)
        .join(category)
        .join(file_name)
        .with_extension(extension.to_string());

    if path.exists() {
        eprintln!("File {} already exists", path.to_string_lossy());
    } else {
        let leading_char = extension.leading_header_character();

        let mut starting_content = format!("{} {}\n", leading_char, title);
        if tags.is_some() {
            starting_content.push_str("\ntags: ");
            starting_content.push_str(tags.unwrap().join(" ").as_str());
        }

        let edited = edit::edit(&starting_content)?;

        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, edited)?;

        if readme {
            build_til_readme(&dir)?;
        }
    }

    return Ok(());
}

// TODO: this should just build the content and return and not write
pub(crate) fn build_til_readme(dir: &str) -> DoctaviousResult<()> {
    let mut all_tils: BTreeMap<String, Vec<TilEntry>> = BTreeMap::new();
    for entry in WalkDir::new(&dir)
        .into_iter()
        .filter_map(Result::ok)
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
        let extension = parse_markup_format_extension(
            entry.path().extension().unwrap().to_str().unwrap(),
        )
        .unwrap();
        let file = match File::open(&entry.path()) {
            Ok(file) => file,
            Err(_) => panic!("Unable to read title from {:?}", entry.path()),
        };

        let buffer = BufReader::new(file);
        let title = title_string(buffer, extension);

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

    let ext = SETTINGS.get_til_template_extension(None);
    let readme_path = Path::new(dir)
        .join("README")
        .with_extension(&ext.extension());

    let ext = MARKUP_FORMAT_EXTENSIONS.get(&ext.extension()).unwrap();
    let template = get_template_content(&dir, ext, DEFAULT_TIL_TEMPLATE_PATH);
    let mut context = TemplateContext::new();
    context.insert("categories_count", &all_tils.keys().len());
    context.insert("til_count", &til_count);
    context.insert("tils", &all_tils);

    let rendered = Templates::one_off(template.as_str(), &context, false)?;
    fs::write(readme_path.as_path(), rendered)?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    use tempfile::{tempdir, tempfile, NamedTempFile};
    use crate::build_til_readme;

    #[test]
    fn markdown_til() {
        let dir = tempdir().unwrap();

        let r = build_til_readme("./docs/til/");
        match r {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }

        // init_adr(
        //     Some(dir.path().display().to_string()),
        //     FileStructure::default(),
        //     Some(MarkupFormat::default()),
        // );

    }

    #[test]
    fn asciidoc_til() {}
}
