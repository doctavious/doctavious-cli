use crate::constants::{DEFAULT_ADR_TEMPLATE_PATH, DEFAULT_ADR_DIR};
use crate::{edit, init_dir};
use crate::file_structure::parse_file_structure;
use crate::file_structure::FileStructure;
use crate::settings::{SETTINGS, load_settings, AdrSettings, persist_settings};
use crate::templates::{
    get_template, parse_template_extension, TemplateExtension,
};
use crate::utils::{build_path, ensure_path, format_number, reserve_number};
use chrono::Utc;
use std::fs;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers ADR management commands")]
pub(crate) struct Adr {
    #[structopt(subcommand)]
    pub adr_command: AdrCommand,
}

#[derive(StructOpt, Debug)]
pub(crate) enum AdrCommand {
    Init(InitAdr),
    New(NewAdr),
    List(ListAdrs),
    Link(LinkAdrs),
    Generate(GenerateADRs),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "init", about = "Init ADR")]
pub(crate) struct InitAdr {
    #[structopt(long, short, help = "Directory to store ADRs")]
    pub directory: Option<String>,

    #[structopt(long, short, default_value, parse(try_from_str = parse_file_structure), help = "How ADRs should be structured")]
    pub structure: FileStructure,

    #[structopt(long, short, default_value, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    pub extension: TemplateExtension,
}

// TODO: should number just be a string and allow people to add their own conventions like leading zeros?
#[derive(StructOpt, Debug)]
#[structopt(name = "new", about = "New ADR")]
pub(crate) struct NewAdr {
    #[structopt(long, short, help = "ADR Number")]
    pub number: Option<i32>,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[structopt(long, short, help = "title of ADR")]
    pub title: String,

    #[structopt(
        long,
        short,
        parse(try_from_str = parse_template_extension),
        help = "Extension that should be used"
    )]
    pub extension: Option<TemplateExtension>,

    #[structopt(
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
    #[structopt(long, short, help = "")]
    pub link: Option<Vec<String>>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "list", about = "List ADRs")]
pub(crate) struct ListAdrs {}

#[derive(StructOpt, Debug)]
#[structopt(name = "link", about = "Link ADRs")]
pub(crate) struct LinkAdrs {
    #[structopt(long, short, help = "Reference number of source ADR")]
    pub source: i32,

    // TODO: can we give title index so we dont have to specify --title or -t?
    #[structopt(
        long,
        short,
        help = "Description of the link created in the new ADR"
    )]
    pub link: String,

    #[structopt(long, short, help = "Reference number of target ADR")]
    pub target: i32,

    #[structopt(
        long,
        short,
        help = "Description of the link created in the existing ADR that will refer to new ADR"
    )]
    pub reverse_link: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers generate ADR commands")]
pub(crate) struct GenerateADRs {
    #[structopt(subcommand)]
    pub generate_adr_command: GenerateAdrsCommand,
}

#[derive(StructOpt, Debug)]
pub(crate) enum GenerateAdrsCommand {
    Toc(AdrToc),
    Graph(AdrGraph),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Generates ADR table of contents (Toc) to stdout")]
pub(crate) struct AdrToc {
    #[structopt(long, short, help = "")]
    pub intro: Option<String>,

    #[structopt(long, help = "")]
    pub outro: Option<String>,

    #[structopt(long, short, help = "")]
    pub link_prefix: Option<String>,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Output format")]
    pub format: Option<TemplateExtension>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create ADR Graph")]
pub(crate) struct AdrGraph {
    #[structopt(long, short, help = "")]
    pub intro: Option<String>,

    #[structopt(long, help = "")]
    pub outro: Option<String>,

    #[structopt(long, short, help = "")]
    pub link_prefix: Option<String>,
}

pub(crate) fn init_adr(
    directory: Option<String>,
    structure: FileStructure,
    extension: TemplateExtension
) -> Result<(), Box<dyn std::error::Error>> {
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
        template_extension: Some(extension),
    };

    settings.adr_settings = Some(adr_settings);

    persist_settings(settings)?;
    init_dir(dir)?;

    return new_adr(
        Some(1),
        "Record Architecture Decisions".to_string(),
        extension,
    );
}

pub(crate) fn new_adr(
    number: Option<i32>,
    title: String,
    extension: TemplateExtension,
    // supercedes: Option<Vec<String>>,
    // links: Option<Vec<String>>
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = SETTINGS.get_adr_dir();
    let template = get_template(&dir, extension, DEFAULT_ADR_TEMPLATE_PATH);
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

    let edited = edit::edit(&starting_content)?;
    fs::write(&adr_path, edited)?;

    return Ok(());
}


#[cfg(test)]
mod tests {
    use tempfile::{tempdir, tempfile, NamedTempFile};
    use std::fs::File;
    use std::io::{self, Write, Read};
    use crate::commands::adr::init_adr;
    use crate::file_structure::FileStructure;
    use crate::templates::TemplateExtension;

    // init default
    #[test]
    fn init() {
        let dir = tempdir()?;

        init_adr( dir.as_path().display().to_string(), FileStructure::default(), TemplateExtension::default());

        dir.close()?;
    }

    // init options

    // init override existing

    // new w/o init

    // new with init
}
