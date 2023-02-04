use std::fs;
use std::path::{Path, PathBuf};

use crate::commands::design_decisions::get_template;
use crate::constants::{
    DEFAULT_ADR_DIR, DEFAULT_ADR_TEMPLATE_PATH, INIT_ADR_TEMPLATE_PATH,
};
use crate::doctavious_error::Result;
use crate::file_structure::FileStructure;
use crate::{get_content, git};
use crate::markup_format::{
    MarkupFormat, MARKUP_FORMAT_EXTENSIONS,
};
use crate::settings::{load_settings, persist_settings, AdrSettings, SETTINGS};
use crate::templates::{TemplateContext, Templates};
use crate::utils::{build_path, ensure_path, format_number, list, reserve_number};
use crate::{edit, init_dir};
use chrono::Utc;
use clap::Parser;
use dotavious::{Dot, Edge, GraphBuilder, Node};
use git2::Repository;
use crate::commands::build_toc;
use crate::file_structure::parse_file_structure;
use crate::output::Output;

// TODO: this should probably be ADRCommand and below should be ADRSubCommands
#[derive(Parser, Debug)]
#[command(about = "Gathers ADR management commands")]
pub(crate) struct ADR {
    #[command(subcommand)]
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
#[command(name = "init", about = "Init ADR")]
pub(crate) struct InitADR {
    #[arg(long, short, help = "Directory to store ADRs")]
    pub directory: Option<String>,

    #[arg(
        value_enum,
        long,
        short,
        default_value_t,
        value_parser = parse_file_structure,
        help = "How ADRs should be structured"
    )]
    pub structure: FileStructure,

    #[arg(
        long,
        short,
        // value_parser = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        value_parser,
        help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,
}

// TODO: should number just be a string and allow people to add their own conventions like leading zeros?
#[derive(Parser, Debug)]
#[command(name = "new", about = "New ADR")]
pub(crate) struct NewADR {
    #[arg(long, short, help = "ADR Number")]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[arg(long, short, help = "title of ADR")]
    pub title: String,

    #[arg(
        long,
        short,
        // possible_values = MARKUP_FORMAT_EXTENSIONS.keys(),
        // value_parser = parse_markup_format_extension,
        value_parser,
        help = "Extension that should be used"
    )]
    pub extension: Option<MarkupFormat>,

    #[arg(
        long,
        short,
        help = "A reference (number or partial filename) of a previous decision that the new \
                decision supercedes. A Markdown link to the superceded ADR is inserted into the \
                Status section. The status of the superceded ADR is changed to record that it has \
                been superceded by the new ADR."
    )]
    pub supercede: Option<Vec<String>>,

    // Links the new ADR to a previous ADR.
    // TARGET is a reference (number or partial filename) of a
    // previous decision.
    // LINK is the description of the link created in the new ADR.
    // REVERSE-LINK is the description of the link created in the
    // existing ADR that will refer to the new ADR.
    #[arg(long, short, help = "")]
    pub link: Option<Vec<String>>,
}

#[derive(Parser, Debug)]
#[command(name = "list", about = "List ADRs")]
pub(crate) struct ListADRs {}

#[derive(Parser, Debug)]
#[command(name = "link", about = "Link ADRs")]
pub(crate) struct LinkADRs {
    #[arg(long, short, help = "Reference number of source ADR")]
    pub source: i32,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[arg(
        long,
        short,
        help = "Description of the link created in the new ADR"
    )]
    pub link: String,

    #[arg(long, short, help = "Reference number of target ADR")]
    pub target: i32,

    #[arg(
        long,
        short,
        help = "Description of the link created in the existing ADR that will refer to new ADR"
    )]
    pub reverse_link: String,
}

#[derive(Parser, Debug)]
#[command(about = "Gathers generate ADR commands")]
pub(crate) struct GenerateADRs {
    #[command(subcommand)]
    pub generate_adr_command: GenerateAdrsCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum GenerateAdrsCommand {
    Toc(AdrToc),
    Graph(AdrGraph),
}

#[derive(Parser, Debug)]
#[command(about = "Generates ADR table of contents (Toc) to stdout")]
pub(crate) struct AdrToc {
    #[arg(long, short, help = "")]
    pub intro: Option<String>,

    #[arg(long, help = "")]
    pub outro: Option<String>,

    #[arg(long, short, help = "")]
    pub link_prefix: Option<String>,

    #[arg(
        long,
        short,
        // value_parser = parse_markup_format_extension,
        value_parser,
        help = "Output format"
    )]
    pub format: Option<MarkupFormat>,
}

#[derive(Parser, Debug)]
#[command(about = "Create ADR Graph")]
pub(crate) struct AdrGraph {
    #[arg(long, short, help = "Directory of ADRs")]
    pub directory: Option<String>,

    // TODO: what to default to?
    #[arg(long, short, help = "")]
    pub link_extension: Option<String>,

    #[arg(long, short, help = "")]
    pub link_prefix: Option<String>,
}

#[derive(Parser, Debug)]
#[command(name = "reserve", about = "Reserve ADR")]
pub(crate) struct ReserveADR {
    #[arg(long, short, help = "ADR Number")]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[arg(long, short, help = "title of ADR")]
    pub title: String,

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

pub(crate) fn handle_adr_command(command: ADR, output: Option<Output>) -> Result<()> {
    match command.adr_command {
        ADRCommand::Init(params) => {
            // https://stackoverflow.com/questions/32788915/changing-the-return-type-of-a-function-returning-a-result
            return match init_adr(
                params.directory,
                params.structure,
                params.extension,
            ) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            };
        }

        ADRCommand::List(_) => {
            list(SETTINGS.get_adr_dir(), output);
        }

        ADRCommand::Link(params) => {
            // get file. needs to support both structures and extensions
            let source_content = get_content(
                SETTINGS.get_adr_dir(),
                &format_number(params.source),
                SETTINGS.get_adr_structure(),
            );

            let z = Path::new(SETTINGS.get_adr_dir())
                .join("temp-link")
                .with_extension("md");
        }

        ADRCommand::Generate(generate) => {
            match generate.generate_adr_command {
                GenerateAdrsCommand::Toc(params) => {
                    let dir = SETTINGS.get_adr_dir();
                    let extension =
                        SETTINGS.get_adr_template_extension(params.format);

                    build_toc(
                        dir,
                        extension,
                        params.intro,
                        params.outro,
                        params.link_prefix,
                    );
                }

                GenerateAdrsCommand::Graph(params) => {
                    graph_adrs();
                    // Generates a visualisation of the links between decision records in
                    // Graphviz format.  This can be piped into the graphviz tools to
                    // generate a an image file.

                    // Each node in the graph represents a decision record and is linked to
                    // the decision record document.

                    // Options:

                    // -e LINK-EXTENSION
                    //         the file extension of the documents to which generated links refer.
                    //         Defaults to `.html`.

                    // -p LINK_PREFIX
                    //         prefix each decision file link with LINK_PREFIX.

                    // E.g. to generate a graph visualisation of decision records in SVG format:

                    //     adr generate graph | dot -Tsvg > graph.svg

                    // E.g. to generate a graph visualisation in PDF format, in which all links
                    // are to .pdf files:

                    //    adr generate graph -e .pdf | dot -Tpdf > graph.pdf
                }
            }
        }

        ADRCommand::New(params) => {
            init_dir(SETTINGS.get_adr_dir())?;

            let extension =
                SETTINGS.get_adr_template_extension(params.extension);
            return match new_adr(
                params.number,
                params.title,
                extension,
                DEFAULT_ADR_TEMPLATE_PATH,
            ) {
                Ok(_) => Ok(()),
                Err(err) => Err(err),
            };
        }

        ADRCommand::Reserve(params) => {
            let extension =
                SETTINGS.get_adr_template_extension(params.extension);
            return reserve_adr(params.number, params.title, extension);
        }
    }

    Ok(())
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

    // TODO: This seems a bit unnecessary for init which is pretty much static content outside of date
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
    let template = get_template(&dir, &extension.extension(), template_path);
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

    let starting_content = fs::read_to_string(&template).expect(&format!(
        "failed to read file {}.",
        &template.to_string_lossy()
    ));

    let mut context = TemplateContext::new();
    context.insert("number", &reserve_number);
    context.insert("title", &title);
    // TODO: allow date to be customized
    context.insert("date", &Utc::now().format("%Y-%m-%d").to_string());

    let rendered =
        Templates::one_off(starting_content.as_str(), &context, false)?;

    let edited = edit::edit(&rendered)?;
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
        return Err(git2::Error::from_str(
            "branch already exists in remote. Please pull.",
        )
        .into());
    }

    git::checkout_branch(&repo, reserve_number.to_string().as_str());

    // TODO: revisit clones. Using it for now to resolve value borrowed here after move
    let created_result =
        new_adr(number, title.clone(), extension, DEFAULT_ADR_TEMPLATE_PATH);

    let message = format!(
        "{}: Adding placeholder for ADR {}",
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

    use tempfile::{tempdir, tempfile, NamedTempFile};

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
