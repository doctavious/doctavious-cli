use crate::commands::design_decisions::get_template;
use crate::constants::{DEFAULT_RFD_DIR, DEFAULT_RFD_TEMPLATE_PATH};
use crate::doctavious_error::Result;
use crate::markup_format::{
    MarkupFormat, MARKUP_FORMAT_EXTENSIONS,
};
use crate::settings::{load_settings, persist_settings, RFDSettings, SETTINGS};
use crate::templates::{TemplateContext, Templates};
use crate::utils::{build_path, ensure_path, format_number, reserve_number};
use crate::{edit, git, init_dir, FileStructure};
use chrono::Utc;
use clap::{Parser, Subcommand};
use dotavious::{Dot, Edge, GraphBuilder, Node};
use git2::Repository;
use std::fs;
use std::path::PathBuf;
use crate::file_structure::parse_file_structure;


#[derive(Parser, Debug)]
#[command(about = "Gathers RFD management commands")]
pub(crate) struct RFD {
    #[command(subcommand)]
    pub rfd_command: RFDCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RFDCommand {
    Init(InitRFD),
    New(NewRFD),
    List(ListRFDs),
    Generate(GenerateRFDs),
    Reserve(ReserveRFD),
}

#[derive(Parser, Debug)]
#[command(about = "Init RFD")]
pub(crate) struct InitRFD {
    #[arg(long, short, help = "Directory to store RFDs")]
    pub directory: Option<String>,

    // TODO: should we default here?
    #[arg(
        value_enum,
        long,
        short,
        default_value_t,
        value_parser = parse_file_structure,
        help = "How RFDs should be structured"
    )]
    pub structure: FileStructure,

    #[arg(
        long,
        short,
        default_value_t,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
        help = "Extension that should be used"
    )]
    pub extension: MarkupFormat,
}

#[derive(Parser, Debug)]
#[command(about = "New RFD")]
pub(crate) struct NewRFD {
    #[arg(long, short, help = "RFD number")]
    pub number: Option<i32>,

    #[arg(long, short, help = "title of RFD")]
    pub title: String,

    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
        help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,
}

#[derive(Parser, Debug)]
#[command(about = "List RFDs")]
pub(crate) struct ListRFDs {}

#[derive(Parser, Debug)]
#[command(about = "Gathers generate RFD commands")]
pub(crate) struct GenerateRFDs {
    #[command(subcommand)]
    pub generate_rfd_command: GenerateRFDsCommand,
}

// TODO: flush this out more?
// keeping ToC is probably fine
// but also want to generate CSV
// Generate README / index file
// Update README with table (maybe even list)
#[derive(Parser, Debug)]
pub(crate) enum GenerateRFDsCommand {
    Toc(RFDToc), // template, csv file. what is the snippet?
    Csv(RFDCsv),
    File(RFDFile),
    // TODO: CSV - path, if exists just update. What about supporting it in another branch/remote. what about committing to that branch? flag for commit and commit message?
    // TODO: File - // template and path to where file should be created
    Graph(RFDGraph),
}

// optional file means to stdout
// add overwrite flag to not modify existing
// remote? commit message?
#[derive(Parser, Debug)]
#[command(about = "Generates RFD CSV")]
pub(crate) struct RFDCsv {
    #[arg(long, short, help = "Directory of RFDs")]
    pub directory: Option<String>,

    // #[clap(parse(from_os_str)] -> #[clap(value_parser)]
    // output_path
    #[arg(value_parser, long, short, help = "")]
    pub path: Option<PathBuf>, // where to write file to. stdout if not provided

    #[arg(long, short, help = "")]
    pub fields: Vec<String>, // which fields to include? default to all (*). should this just be a comma separate list?

    #[arg(long, short, help = "")]
    pub overwrite: bool,
}

#[derive(Parser, Debug)]
#[command(about = "Generates RFD File")]
pub(crate) struct RFDFile {
    #[arg(long, short, help = "Directory of RFDs")]
    pub directory: Option<String>,

    #[arg(
        long,
        short,
        help = "Template that will be used to generate file. \
                If not present use value from config otherwise default template based on \
                output_path extension will be used. See <location> for default template"
    )]
    pub template: Option<String>, // optional. use config, use provided here. use default

    // output_path
    #[arg(
        long,
        short,
        value_parser,
        help = "Path to file which to write table of contents to. File must contain snippet. \
                If not present ToC will be written to stdout"
    )]
    pub path: PathBuf, // where to write file to. required
}

#[derive(Parser, Debug)]
#[command(about = "Generates RFD table of contents (Toc) to stdout")]
pub(crate) struct RFDToc {
    #[arg(long, short, help = "Directory of RFDs")]
    pub directory: Option<String>,

    #[arg(
        long,
        short,
        help = "Template that will be used to generate file. \
                If not present use value from config otherwise default template based on \
                output_path extension will be used. See <location> for default template"
    )]
    pub template: Option<String>, // optional. use config, use provided here. use default

    #[arg(
        long,
        short,
        value_parser,
        help = "Path to file which to write table of contents to. File must contain snippet. \
                If not present ToC will be written to stdout"
    )]
    pub output_path: PathBuf, // where to write file to. required

    #[arg(long, short, help = "")]
    pub intro: Option<String>,

    #[arg(long, help = "")]
    pub outro: Option<String>,

    #[arg(long, short, help = "")]
    pub link_prefix: Option<String>,

    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
        help = "Output format"
    )]
    pub format: Option<MarkupFormat>,
}

#[derive(Parser, Debug)]
#[command(about = "Create RFD Graph")]
pub(crate) struct RFDGraph {
    #[arg(long, short, help = "Directory of RFDs")]
    pub directory: Option<String>,

    // TODO: what to default to?
    #[arg(long, short, help = "")]
    pub link_extension: Option<String>,

    #[arg(long, short, help = "")]
    pub link_prefix: Option<String>,
}

#[derive(Parser, Debug)]
#[command(name = "reserve", about = "Reserve RFD")]
pub(crate) struct ReserveRFD {
    #[arg(long, short, help = "RFD Number")]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[arg(long, short, help = "title of RFD")]
    pub title: String,

    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // parse(try_from_str = parse_markup_format_extension),
        value_parser,
        help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,
}

pub(crate) fn init_rfd(
    directory: Option<String>,
    structure: FileStructure,
    extension: MarkupFormat,
) -> Result<PathBuf> {
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
    return new_rfd(Some(1), "Use RFDs ...".to_string(), extension);
}

pub(crate) fn new_rfd(
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
) -> Result<PathBuf> {
    let dir = SETTINGS.get_rfd_dir();
    let template = get_template(&dir, &extension.extension(), DEFAULT_RFD_TEMPLATE_PATH);
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

    let starting_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));

    let mut context = TemplateContext::new();
    context.insert("number", &reserve_number);
    context.insert("title", &title);
    context.insert("date", &Utc::now().format("%Y-%m-%d").to_string());

    let rendered =
        Templates::one_off(starting_content.as_str(), &context, false)?;

    let edited = edit::edit(&rendered)?;
    fs::write(&rfd_path, edited)?;

    return Ok(rfd_path);
}

pub(crate) fn reserve_rfd(
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
) -> Result<()> {
    let dir = SETTINGS.get_rfd_dir();
    let reserve_number =
        reserve_number(&dir, number, SETTINGS.get_rfd_structure())?;

    // TODO: support more than current directory
    let repo = Repository::open(".")?;
    if git::branch_exists(&repo, reserve_number) {
        return Err(git2::Error::from_str(
            "branch already exists in remote. Please pull.",
        )
        .into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str());

    // TODO: revisit clones. Using it for now to resolve value borrowed here after move
    let created_result = new_rfd(number, title.clone(), extension);

    let message = format!(
        "{}: Adding placeholder for RFD {}",
        reserve_number,
        title.clone()
    );
    git::add_and_commit(
        &repo,
        created_result.unwrap().as_path(),
        message.as_str(),
    );
    git::push(&repo);

    return Ok(());
}

pub(crate) fn generate_csv() {}

pub(crate) fn graph_rfds() {
    let graph = GraphBuilder::new_named_directed("example")
        .add_node(Node::new("N0"))
        .add_node(Node::new("N1"))
        .add_edge(Edge::new("N0", "N1"))
        .build()
        .unwrap();

    let dot = Dot { graph };
}
