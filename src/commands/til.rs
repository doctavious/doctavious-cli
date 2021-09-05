use crate::settings::SETTINGS;
use crate::templates::{parse_template_extension, TemplateExtension};
use chrono::{Utc, DateTime};
use std::{fs, io};
use structopt::StructOpt;
use std::path::Path;
use std::fs::File;
use std::io::{LineWriter, BufReader, Write};
use std::collections::BTreeMap;
use walkdir::WalkDir;
use crate::commands::title_string;

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers Today I Learned (TIL) management commands")]
pub(crate) struct Til {
    #[structopt(subcommand)]
    pub til_command: TilCommand,
}

#[derive(StructOpt, Debug)]
pub(crate) enum TilCommand {
    Init(InitTil),
    New(NewTil),
    List(ListTils),
    Readme(BuildTilReadMe),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Init TIL")]
pub(crate) struct InitTil {
    #[structopt(long, short, help = "Directory to store TILs")]
    pub directory: Option<String>,

    #[structopt(long, short, default_value, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    pub extension: TemplateExtension,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "New TIL")]
pub(crate) struct NewTil {
    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    // TODO: change to category
    #[structopt(
    short,
    long,
    help = "TIL category. Represents the directory to place TIL entry under"
    )]
    pub category: String,

    #[structopt(long, short, help = "title of the TIL entry")]
    pub title: String,

    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    #[structopt(
    short = "T",
    long,
    help = "Additional tags associated with the TIL entry"
    )]
    pub tags: Option<Vec<String>>,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    pub extension: Option<TemplateExtension>,

    // TODO: should this also be a setting in TilSettings?
    #[structopt(
    short,
    long,
    help = "Whether to build a README after a new TIL is added"
    )]
    pub readme: bool,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List TILs")]
pub(crate) struct ListTils {}

#[derive(StructOpt, Debug)]
#[structopt(about = "Build TIL ReadMe")]
pub(crate) struct BuildTilReadMe {
    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    pub extension: Option<TemplateExtension>,
}

#[derive(Clone, Debug)]
struct TilEntry {
    topic: String,
    title: String,
    file_name: String,
    date: DateTime<Utc>,
}


pub(crate) fn build_til_readme(dir: &str) -> io::Result<()> {
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
        let extension = parse_template_extension(
            entry.path().extension().unwrap().to_str().unwrap(),
        )
            .unwrap();
        let file = match fs::File::open(&entry.path()) {
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

    let readme_path = Path::new(dir)
        .join("README")
        .with_extension(SETTINGS.get_til_template_extension().to_string());
    let file = File::create(readme_path)?;

    // TODO: better alternative than LineWriter?
    let mut lw = LineWriter::new(file);

    lw.write_all(b"# TIL\n\n> Today I Learned\n\n")?;
    lw.write_all(
        format!("* Categories: {}\n", all_tils.keys().len()).as_bytes(),
    )?;
    lw.write_all(format!("* TILs: {}\n", til_count).as_bytes())?;
    lw.write_all(b"\n")?;

    // TODO: do we want to list categories?

    for category in all_tils.keys().cloned() {
        lw.write_all(format!("## {}\n\n", &category).as_bytes())?;
        let mut tils = all_tils.get(&category).unwrap().to_vec();
        tils.sort_by_key(|e| e.title.clone());
        for til in tils {
            // TODO: should we just use title instead of file_name for link?
            lw.write_all(
                format!(
                    "* [{}]({}/{}) {} ({})",
                    til.file_name,
                    category,
                    til.file_name,
                    til.title,
                    til.date.format("%Y-%m-%d")
                )
                    .as_bytes(),
            )?;
            lw.write_all(b"\n")?;
        }

        lw.write_all(b"\n")?;
    }

    // TODO: handle this
    return lw.flush();
}
