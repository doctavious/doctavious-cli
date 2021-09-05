use crate::constants::{DEFAULT_RFD_TEMPLATE_PATH, DEFAULT_RFD_DIR};
use crate::{edit, init_dir, git};
use crate::file_structure::parse_file_structure;
use crate::file_structure::FileStructure;
use crate::settings::{SETTINGS, load_settings, RFDSettings, persist_settings};
use crate::templates::{
    get_template, parse_template_extension, TemplateExtension,
};
use crate::utils::{build_path, ensure_path, format_number, reserve_number};
use chrono::Utc;
use std::fs;
use structopt::StructOpt;
use git2::Repository;
use std::path::PathBuf;

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers RFD management commands")]
pub(crate) struct RFD {
    #[structopt(subcommand)]
    pub rfd_command: RFDCommand,
}

#[derive(StructOpt, Debug)]
pub(crate) enum RFDCommand {
    Init(InitRFD),
    New(NewRFD),
    List(ListRFDs),
    Generate(GenerateRFDs),
    Reserve(ReserveRFD),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Init RFD")]
pub(crate) struct InitRFD {
    #[structopt(long, short, help = "Directory to store RFDs")]
    pub directory: Option<String>,

    // TODO: should we default here?
    #[structopt(
        long,
        short,
        default_value,
        possible_values = &FileStructure::variants(),
        parse(try_from_str = parse_file_structure),
        help = "How RFDs should be structured"
    )]
    pub structure: FileStructure,

    #[structopt(
        long,
        short,
        default_value,
        possible_values = &TemplateExtension::variants(),
        parse(try_from_str = parse_template_extension),
        help = "Extension that should be used"
    )]
    pub extension: TemplateExtension,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "New RFD")]
pub(crate) struct NewRFD {
    #[structopt(long, short, help = "RFD number")]
    pub number: Option<i32>,

    #[structopt(long, short, help = "title of RFD")]
    pub title: String,

    #[structopt(
        long,
        short,
        possible_values = &TemplateExtension::variants(),
        parse(try_from_str = parse_template_extension),
        help = "Extension that should be used"
    )]
    pub extension: Option<TemplateExtension>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List RFDs")]
pub(crate) struct ListRFDs {}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers generate RFD commands")]
pub(crate) struct GenerateRFDs {
    #[structopt(subcommand)]
    pub generate_rfd_command: GenerateRFDsCommand,
}

#[derive(StructOpt, Debug)]
pub(crate) enum GenerateRFDsCommand {
    Toc(RFDToc),
    Graph(RFDGraph),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Generates RFD table of contents (Toc) to stdout")]
pub(crate) struct RFDToc {
    #[structopt(long, short, help = "")]
    pub intro: Option<String>,

    #[structopt(long, help = "")]
    pub outro: Option<String>,

    #[structopt(long, short, help = "")]
    pub link_prefix: Option<String>,

    #[structopt(
        long,
        short,
        possible_values = &TemplateExtension::variants(),
        parse(try_from_str = parse_template_extension),
        help = "Output format"
    )]
    pub format: Option<TemplateExtension>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create RFD Graph")]
pub(crate) struct RFDGraph {
    #[structopt(long, short, help = "")]
    pub intro: Option<String>,

    #[structopt(long, help = "")]
    pub outro: Option<String>,

    #[structopt(long, short, help = "")]
    pub link_prefix: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "reserve", about = "Reserve RFD")]
pub(crate) struct ReserveRFD {
    #[structopt(long, short, help = "RFD Number")]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[structopt(long, short, help = "title of RFD")]
    pub title: String,

    #[structopt(
        long,
        short,
        possible_values = &TemplateExtension::variants(),
        parse(try_from_str = parse_template_extension),
        help = "Extension that should be used"
    )]
    pub extension: Option<TemplateExtension>,
}

pub(crate) fn init_rfd(
    directory: Option<String>,
    structure: FileStructure,
    extension: TemplateExtension
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut settings = match load_settings() {
        Ok(settings) => settings,
        Err(_) => Default::default(),
    };

    let dir = match directory {
        None => DEFAULT_RFD_DIR,
        Some(ref d) => d,
    };

    let rfd_settings = RFDSettings {
        dir: Some(dir.to_string()),
        structure: Some(structure),
        template_extension: Some(extension),
    };
    settings.rfd_settings = Some(rfd_settings);

    persist_settings(settings)?;
    init_dir(dir)?;

    // TODO: fix
    return new_rfd(
        Some(1),
        "Use RFDs ...".to_string(),
        extension,
    );
}

pub(crate) fn new_rfd(
    number: Option<i32>,
    title: String,
    extension: TemplateExtension,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dir = SETTINGS.get_rfd_dir();
    let template = get_template(&dir, extension, DEFAULT_RFD_TEMPLATE_PATH);
    let reserve_number =
        reserve_number(&dir, number, SETTINGS.get_rfd_structure())?;
    let formatted_reserved_number = format_number(reserve_number);
    let rfd_path = build_path(
        &dir,
        &title,
        &formatted_reserved_number,
        extension,
        SETTINGS.get_rfd_structure(),
    );
    ensure_path(&rfd_path)?;

    // TODO: supersceded
    // TODO: reverse links

    let mut starting_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));
    starting_content =
        starting_content.replace("<NUMBER>", &formatted_reserved_number);
    starting_content = starting_content.replace("<TITLE>", &title);
    starting_content = starting_content
        .replace("<DATE>", &Utc::now().format("%Y-%m-%d").to_string());

    let edited = edit::edit(&starting_content)?;
    fs::write(&rfd_path, edited)?;

    return Ok(rfd_path);
}

pub(crate) fn reserve_rfd(
    number: Option<i32>,
    title: String,
    extension: TemplateExtension,
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = SETTINGS.get_rfd_dir();
    let reserve_number =
        reserve_number(&dir, number, SETTINGS.get_rfd_structure())?;

    let repo = Repository::open(".")?;
    if git::branch_exists(&repo, reserve_number) {
        return Err(String::from("branch already exists in remote. Please pull.").into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str());

    // TODO: revisit clones. Using it for now to resolve value borrowed here after move
    let created_result = new_rfd(number, title.clone(), extension);

    let message = format!("{}: Adding placeholder for RFD {}", reserve_number, title.clone());
    git::add_and_commit(&repo, created_result.unwrap().as_path(), message.as_str());
    git::push(&repo);

    return Ok(())
}
