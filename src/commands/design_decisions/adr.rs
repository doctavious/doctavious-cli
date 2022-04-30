use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use chrono::Utc;
use dotavious::{Dot, Edge, GraphBuilder, Node};
use git2::{Branches, BranchType, Direction, Repository};
use regex::Regex;

use crate::{edit, init_dir};
use crate::constants::{DEFAULT_ADR_DIR, DEFAULT_ADR_TEMPLATE_PATH, INIT_ADR_TEMPLATE_PATH};
use crate::file_structure::FileStructure;
use crate::file_structure::parse_file_structure;
use crate::git;
use crate::settings::{AdrSettings, load_settings, persist_settings, SETTINGS};
// use crate::templates::{
//     parse_template_extension, TemplateExtension,
// };
use crate::utils::{build_path, ensure_path, format_number, reserve_number};
use crate::doctavious_error::Result;
use tera::{
    Context as TeraContext,
    Result as TeraResult,
    Tera,
    Value,
};
use crate::commands::design_decisions::get_template;
use crate::markup_format::{MarkupFormat,parse_markup_format_extension,MARKUP_FORMAT_EXTENSIONS};

#[derive(Parser, Debug)]
#[clap(about = "Gathers ADR management commands")]
pub(crate) struct ADR {
    #[clap(subcommand)]
    pub adr_command: ADRCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum ADRCommand {
    Init(InitADR),
    Generate(GenerateADRs),
    List(ListADRs),
    Link(LinkADRs),
    New(NewADR),
    Reserve(ReserveADR),
}

#[derive(Parser, Debug)]
#[clap(name = "init", about = "Init ADR")]
pub(crate) struct InitADR {
    #[clap(long, short, help = "Directory to store ADRs")]
    pub directory: Option<String>,

    #[clap(
    arg_enum,
    long,
    short,
    default_value_t,
    // possible_values = &FileStructure::variants(),
    parse(try_from_str = parse_file_structure),
    help = "How ADRs should be structured"
    )]
    pub structure: FileStructure,

    #[clap(
    arg_enum,
    long,
    short,
    // possible_values = &TemplateExtension::variants(),
    parse(try_from_str = parse_markup_format_extension),
    help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,
}

// TODO: should number just be a string and allow people to add their own conventions like leading zeros?
#[derive(Parser, Debug)]
#[clap(name = "new", about = "New ADR")]
pub(crate) struct NewADR {
    #[clap(long, short, help = "ADR Number")]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[clap(long, short, help = "title of ADR")]
    pub title: String,

    #[clap(
    // arg_enum,
    long,
    short,
    possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
    parse(try_from_str = parse_markup_format_extension),
    help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,

    #[clap(
    long,
    short,
    help = "A reference (number or partial filename) of a previous decision that the new decision supercedes. A Markdown link to the superceded ADR is inserted into the Status section. The status of the superceded ADR is changed to record that it has been superceded by the new ADR."
    )]
    pub supercede: Option<Vec<String>>,

    // Links the new ADR to a previous ADR.
    // TARGET is a reference (number or partial filename) of a
    // previous decision.
    // LINK is the description of the link created in the new ADR.
    // REVERSE-LINK is the description of the link created in the
    // existing ADR that will refer to the new ADR.
    #[clap(long, short, help = "")]
    pub link: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
#[clap(name = "list", about = "List ADRs")]
pub(crate) struct ListADRs {}

#[derive(Parser, Debug)]
#[clap(name = "link", about = "Link ADRs")]
pub(crate) struct LinkADRs {
    #[clap(long, short, help = "Reference number of source ADR")]
    pub source: i32,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[clap(
    long,
    short,
    help = "Description of the link created in the new ADR"
    )]
    pub link: String,

    #[clap(long, short, help = "Reference number of target ADR")]
    pub target: i32,

    #[clap(
    long,
    short,
    help = "Description of the link created in the existing ADR that will refer to new ADR"
    )]
    pub reverse_link: String,
}

#[derive(Parser, Debug)]
#[clap(about = "Gathers generate ADR commands")]
pub(crate) struct GenerateADRs {
    #[clap(subcommand)]
    pub generate_adr_command: GenerateAdrsCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum GenerateAdrsCommand {
    Toc(AdrToc),
    Graph(AdrGraph),
}

#[derive(Parser, Debug)]
#[clap(about = "Generates ADR table of contents (Toc) to stdout")]
pub(crate) struct AdrToc {
    #[clap(long, short, help = "")]
    pub intro: Option<String>,

    #[clap(long, help = "")]
    pub outro: Option<String>,

    #[clap(long, short, help = "")]
    pub link_prefix: Option<String>,

    #[clap(long, short, parse(try_from_str = parse_markup_format_extension), help = "Output format")]
    pub format: Option<MarkupFormat>,
}

#[derive(Parser, Debug)]
#[clap(about = "Create ADR Graph")]
pub(crate) struct AdrGraph {
    #[clap(long, short, help = "Directory of ADRs")]
    pub directory: Option<String>,

    // TODO: what to default to?
    #[clap(long, short, help = "")]
    pub link_extension: Option<String>,

    #[clap(long, short, help = "")]
    pub link_prefix: Option<String>,
}

#[derive(Parser, Debug)]
#[clap(name = "reserve", about = "Reserve ADR")]
pub(crate) struct ReserveADR {
    #[clap(long, short, help = "ADR Number")]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[clap(long, short, help = "title of ADR")]
    pub title: String,

    #[clap(
    long,
    short,
    // possible_values = &TemplateExtension::variants(),
    parse(try_from_str = parse_markup_format_extension),
    help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,
}

pub(crate) fn init_adr(
    directory: Option<String>,
    structure: FileStructure,
    extension: Option<MarkupFormat>,
) -> Result<PathBuf> {
    let mut settings = match load_settings() {
        Ok(settings) => settings,
        Err(_) => Default::default(),
    };

    let dir = match directory {
        None => DEFAULT_ADR_DIR,
        Some(ref d) => d,
    };

    let adr_settings = AdrSettings {
        dir: Some(dir.to_string()),
        structure: Some(structure),
        template_extension: extension,
    };

    settings.adr_settings = Some(adr_settings);

    persist_settings(settings)?;
    init_dir(dir)?;

    return new_adr(
        Some(1),
        "Record Architecture Decisions".to_string(),
        SETTINGS.get_adr_template_extension(extension),
        INIT_ADR_TEMPLATE_PATH,
    );
}

pub(crate) fn new_adr(
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
    template_path: &str,
    // supercedes: Option<Vec<String>>,
    // links: Option<Vec<String>>
) -> Result<PathBuf> {
    let dir = SETTINGS.get_adr_dir();
    let template = get_template(&dir, extension, template_path);
    let reserve_number =
        reserve_number(&dir, number, SETTINGS.get_adr_structure())?;
    let formatted_reserved_number = format_number(reserve_number);
    let adr_path = build_path(
        &dir,
        &title,
        &formatted_reserved_number,
        extension,
        SETTINGS.get_adr_structure(),
    );
    ensure_path(&adr_path)?;

    // TODO: supersceded
    // if let Some(targets) = supercedes {
    //     for target in targets {
    //         // "$adr_bin_dir/_adr_add_link" "$target" "Superceded by" "$dstfile"
    //         // "$adr_bin_dir/_adr_remove_status" "Accepted" "$target"
    //         // "$adr_bin_dir/_adr_add_link" "$dstfile" "Supercedes" "$target"
    //     }
    // }

    // TODO: reverse links
    // if let Some(others) = links {
    //     for other in others {
    //         // target="$(echo $l | cut -d : -f 1)"
    //         // forward_link="$(echo $l | cut -d : -f 2)"
    //         // reverse_link="$(echo $l | cut -d : -f 3)"

    //         // "$adr_bin_dir/_adr_add_link" "$dstfile" "$forward_link" "$target"
    //         // "$adr_bin_dir/_adr_add_link" "$target" "$reverse_link" "$dstfile"
    //     }
    // }

    let mut starting_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));
    starting_content =
        starting_content.replace("<NUMBER>", &reserve_number.to_string());
    starting_content = starting_content.replace("<TITLE>", &title);
    starting_content = starting_content
        .replace("<DATE>", &Utc::now().format("%Y-%m-%d").to_string());
    starting_content = starting_content.replace("<STATUS>", "Accepted");

    let mut context = TeraContext::new();
    context.insert("number", &reserve_number);
    context.insert("title", &title);
    context.insert("date", &Utc::now().format("%Y-%m-%d").to_string());
    context.insert("status", "Accepted");

    // tera.render("template", &TeraContext::from_serialize(release)?)?;
    // Tera::one_off(input, context, autoescape)


    let edited = edit::edit(&starting_content)?;
    fs::write(&adr_path, edited)?;
    return Ok(adr_path);
}


// implement ADR / RFD reserve command
// 1. get latest number
// 2. verify it doesnt exist
// git branch -rl *0042
// 3. checkout
// $ git checkout -b 0042
// 4. create the placeholder
// 5. Push your RFD branch remotely
// $ git add rfd/0042/README.md
// $ git commit -m '0042: Adding placeholder for RFD <Title>'
// $ git push origin 0042
// 6. Update README in main branch
// After your branch is pushed, the table in the README on the master branch will update
// automatically with the new RFD. If you ever change the name of the RFD in the future,
// the table will update as well. Whenever information about the state of the RFD changes,
// this updates the table as well. The single source of truth for information about the RFD comes
// from the RFD in the branch until it is merged.
// I think this would be implemented as a    git hook
pub(crate) fn reserve_adr(
    number: Option<i32>,
    title: String,
    extension: MarkupFormat,
) -> Result<()> {
    let dir = SETTINGS.get_adr_dir();
    let reserve_number =
        reserve_number(&dir, number, SETTINGS.get_adr_structure())?;

    // TODO: support more than current directory
    let repo = Repository::open(".")?;
    if git::branch_exists(&repo, reserve_number) {
        return Err(git2::Error::from_str("branch already exists in remote. Please pull.").into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str());

    // TODO: revisit clones. Using it for now to resolve value borrowed here after move
    let created_result = new_adr(number, title.clone(), extension, DEFAULT_ADR_TEMPLATE_PATH);

    let message = format!("{}: Adding placeholder for ADR {}", reserve_number, title.clone());
    git::add_and_commit(&repo, created_result.unwrap().as_path(), message.as_str());
    git::push(&repo);

    return Ok(())
}

pub(crate) fn generate_csv() {

}

pub(crate) fn graph_adrs() {
    let graph = GraphBuilder::new_named_directed("example")
        .add_node(Node::new("N0"))
        .add_node(Node::new("N1"))
        .add_edge(Edge::new("N0", "N1"))
        .build()
        .unwrap();

    let dot = Dot { graph };
}


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{self, Read, Write};

    use tempfile::{NamedTempFile, tempdir, tempfile};

    use crate::file_structure::FileStructure;
    use crate::init_adr;
    use crate::markup_format::MarkupFormat;

    // init default
    #[test]
    fn init() {
        let dir = tempdir().unwrap();

        init_adr(
            Some(dir.path().display().to_string()),
            FileStructure::default(),
            Some(MarkupFormat::default()),
        );

        dir.close().unwrap();
    }

    // init options

    // init override existing

    // new w/o init

    // new with init
}
