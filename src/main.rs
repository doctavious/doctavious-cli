use lazy_static::lazy_static;
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};

use structopt::StructOpt;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File};
use std::fmt::{Debug, Display, Formatter};
use std::io::{self, Write};
use std::io::prelude::*;
use std::io::ErrorKind;
use std::io::BufReader;
use std::io::LineWriter;
use std::path::{Path, PathBuf};
use chrono:: {
    DateTime,
    prelude::*,
};
use walkdir::WalkDir;
use unidecode::unidecode;

mod edit;

use std::cell::RefCell;

use comrak::{Arena, parse_document, format_commonmark, format_html, ComrakOptions};
use comrak::nodes::{Ast, AstNode, NodeValue};
use comrak::arena_tree::Node;

use pulldown_cmark::{Parser, Options, html, Event};
use pulldown_cmark_to_cmark::cmark;

// https://stackoverflow.com/questions/32555589/is-there-a-clean-way-to-have-a-global-mutable-state-in-a-rust-plugin
// https://stackoverflow.com/questions/61159698/update-re-initialize-a-var-defined-in-lazy-static


// TODO: Automatically update readme TOC
// Update CVS file? From Oxide - we automatically update a CSV file of all the RFDs along with their state, links, and other information in the repo for easy parsing.
// TODO: configuration
// TODO: output options
// config file
// env var DOCTAVIOUS_DEFAULT_OUTPUT - overrides config
// --output  - overrides env var and config
// TODO: RFD / ADR meta frontmatter
// TODO: why do I get a % at the end when using json output
// Create ADR from RFD - essentially a link similar to linking ADRs to one another

// TODO: automatically update README(s) / CSVs
// or at the very least lint 
// - pass over readme to verify that the links are properly formed and ADR/RFD numbers appropriately used in the RFD table.
// - iterate over every RFD and make sure that it's in the table in README.md and in a matching state and with sane metadata.
// - https://github.com/joyent/rfd/blob/master/tools/rfdlint

// TODO: we can prompt user if they try to init multiple times
// https://github.com/sharkdp/bat/blob/5ef35a10cf880c56b0e1c1ca7598ec742030eee1/src/bin/bat/config.rs#L17

// executable path
// https://github.com/rust-lang/rust-clippy/blob/master/src/main.rs#L120

// clippy dogfood
// https://github.com/rust-lang/rust-clippy/blob/master/src/main.rs#L132

// TODO: review https://github.com/simeg/eureka
// some good ideas here

// TODO: review https://github.com/jakedeichert/mask
// TODO: review https://github.com/sharkdp/bat/tree/master/src

// TODO: architecture diagrams as code
// If we do some sort of desktop app we should have preview function to see as you code

// TODO: Today I learned CLI 
// https://github.com/danielecook/til-tool
// https://github.com/danielecook/til

// ADR: build ToC / readme / graph (dot/graphviz)
// RFC: build ToC / readme / graph
// Til: build ToC / readme

// serial number

// TODO: share structs between ADR and RFD


#[derive(StructOpt, Debug)]
#[structopt(
    name = "Doctavious",
)]
pub struct Opt {
    #[structopt(long, help = "Prints a verbose output during the program execution", global = true)]
    debug: bool,

    #[structopt(long, short, parse(try_from_str = parse_output), help = "How a command output should be rendered", global = true)]
    output: Option<Output>,

    #[structopt(subcommand)]
    cmd: Command,
}


// TODO: 
// should text be the following?
// The text format organizes the CLI output into tab-delimited lines. 
// It works well with traditional Unix text tools such as grep, sed, and awk, and the text processing performed by PowerShell.
// The text output format follows the basic structure shown below. 
// The columns are sorted alphabetically by the corresponding key names of the underlying JSON object.
// What about table?
// The table format produces human-readable representations of complex CLI output in a tabular form.
#[derive(Debug, Copy, Clone)]
pub enum Output {
    Json,
    Text,
    Table,
}

impl Default for Output {
    fn default() -> Self { Output::Json }
}

// TODO: is there a better name for this?
// TODO: can these enums hold other attributes? extension value (adoc / md), leading char (= / #), etc
#[derive(Clone, Copy, Debug)]
pub enum TemplateExtension {
    Markdown,
    Asciidoc,
}

impl Default for TemplateExtension {
    fn default() -> Self { TemplateExtension::Markdown }
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

lazy_static! {
    static ref TEMPLATE_EXTENSIONS: HashMap<&'static str, TemplateExtension> = {
        let mut map = HashMap::new();
        map.insert("md", TemplateExtension::Markdown);
        map.insert("adoc", TemplateExtension::Asciidoc);
        map
    };

}

#[derive(Clone, Copy, Debug)]
pub enum FileStructure {
    Flat,
    Nested,
}

impl Default for FileStructure {
    fn default() -> Self { FileStructure::Flat }
}

impl Display for FileStructure {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match *self {
            FileStructure::Flat => write!(f, "flat"),
            FileStructure::Nested => write!(f, "nested"),
        }
    }
}

impl Serialize for FileStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match *self {
            FileStructure::Flat => "flat",
            FileStructure::Nested => "nested",
        };

        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FileStructure {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let structure = match parse_file_structure(&s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Error when parsing {}, fallback to default settings. Error: {}\n", s, e);
                FileStructure::default()
            }
        };
        Ok(structure)
    }
}

lazy_static! {
    static ref FILE_STRUCTURES: HashMap<&'static str, FileStructure> = {
        let mut map = HashMap::new();
        map.insert("flat", FileStructure::Flat);
        map.insert("nested", FileStructure::Nested);
        map
    };
}

// TODO: better way to do this? Do we want to keep a default settings file in doctavious dir?
pub static DEFAULT_ADR_DIR: &str = "docs/adr";
pub static DEFAULT_ADR_TEMPLATE_PATH: &str = "templates/adr/template";
pub static DEFAULT_RFD_DIR: &str = "docs/rfd";
pub static DEFAULT_RFD_TEMPLATE_PATH: &str = "templates/rfd/template";
// TODO: do we want this to defautl to the current directory?
pub static DEFAULT_TIL_DIR: &str = "til";


// TODO: should this include output?
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    template_extension: Option<TemplateExtension>,
    adr_settings: Option<AdrSettings>,
    rfd_settings: Option<RFDSettings>,
    til_settings: Option<TilSettings>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AdrSettings {
    dir: Option<String>,
    structure: Option<FileStructure>,
    template_extension: Option<TemplateExtension>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RFDSettings {
    dir: Option<String>,
    structure: Option<FileStructure>,
    template_extension: Option<TemplateExtension>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TilSettings {
    dir: Option<String>,
    template_extension: Option<TemplateExtension>,
}

impl Settings {

    fn get_adr_dir(&self) -> &str {
        if let Some(settings) = &self.adr_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }
        
        return DEFAULT_ADR_DIR;
    }

    fn get_adr_structure(&self) -> FileStructure {
        if let Some(settings) = &self.adr_settings {
            if let Some(structure) = settings.structure {
                return structure;
            }
        }
        
        return FileStructure::default();
    }

    fn get_adr_template_extension(&self) -> TemplateExtension {
        if let Some(settings) = &self.adr_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }
        
        return TemplateExtension::default();
    }

    fn get_rfd_dir(&self) -> &str {
        if let Some(settings) = &self.rfd_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }
        
        return DEFAULT_ADR_DIR;
    }

    fn get_rfd_structure(&self) -> FileStructure {
        if let Some(settings) = &self.rfd_settings {
            if let Some(structure) = settings.structure {
                return structure;
            }
        }
        
        return FileStructure::default();
    }

    fn get_rfd_template_extension(&self) -> TemplateExtension {
        if let Some(settings) = &self.rfd_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }
        
        return TemplateExtension::default();
    }

    fn get_til_dir(&self) -> &str {
        if let Some(settings) = &self.til_settings {
            if let Some(dir) = &settings.dir {
                return dir;
            }
        }
        
        return DEFAULT_ADR_DIR;
    }

    fn get_til_template_extension(&self) -> TemplateExtension {
        if let Some(settings) = &self.til_settings {
            if let Some(template_extension) = settings.template_extension {
                return template_extension;
            }
        }

        if let Some(template_extension) = self.template_extension {
            return template_extension;
        }
        
        return TemplateExtension::default();
    }

 }

lazy_static! {
    pub static ref DOCTAVIOUS_DIR: PathBuf = {
        let home_dir = dirs::home_dir().expect("Unsupported platform: can't find home directory");
        Path::new(&home_dir).join(".doctavious")
    };

    // TODO: doctavious config will live in project directory
    // do we also want a default settings file
    pub static ref SETTINGS_FILE: PathBuf = PathBuf::from(".doctavious");

    pub static ref SETTINGS: Settings = {
        match load_settings() {
            Ok(settings) => settings,
            Err(e) => {
                if std::path::Path::new(SETTINGS_FILE.as_path()).exists() {
                    eprintln!(
                        "Error when parsing {}, fallback to default settings. Error: {}\n",
                        SETTINGS_FILE.as_path().display(),
                        e
                    );
                }
                Default::default()
            }
        }
    };
}

fn load_settings() -> Result<Settings, Box<dyn std::error::Error>> {
    let bytes = std::fs::read(SETTINGS_FILE.as_path())?;
    let settings: Settings = toml::from_slice(&bytes)?;
    Ok(settings)
}

// outside of Settings because we dont want to initialize load given we are using lazy_static
fn persist_settings(settings: Settings) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string(&settings)?;
    fs::write(SETTINGS_FILE.as_path(), content)?;

    Ok(())
}

#[derive(StructOpt, Debug)]
enum Command {
    RFD(RFD),
    Adr(Adr),
    Til(Til),
    Presentation(Presentation),
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Gathers RFD management commands"
)]
struct RFD {
    #[structopt(subcommand)]
    rfd_command: RFDCommand,
}

#[derive(StructOpt, Debug)]
enum RFDCommand {
    Init(InitRFD),
    New(NewRFD),
    List(ListRFDs),
    Generate(GenerateRFDs),
}


#[derive(StructOpt, Debug)]
#[structopt(about = "Init RFD")]
struct InitRFD {
    #[structopt(long, short, help = "Directory to store RFDs")]
    directory: Option<String>,

    // TODO: should we default here?
    #[structopt(long, short, parse(try_from_str = parse_file_structure), help = "How RFDs should be structured")]
    structure: Option<FileStructure>,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    extension: Option<TemplateExtension>,
}

impl InitRFD {
    pub fn should_persist_settings(&self) -> bool {
        return self.directory.is_some() || self.extension.is_some();
    }
}

#[derive(StructOpt, Debug)]
#[structopt(about = "New RFD")]
struct NewRFD {
    #[structopt(long, short, help = "RFD number")]
    number: Option<i32>,
    
    #[structopt(long, short, help = "title of RFD")]
    title: String,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    extension: Option<TemplateExtension>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List RFDs")]
struct ListRFDs {
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers generate RFD commands")]
struct GenerateRFDs {
    #[structopt(subcommand)]
    generate_rfd_command: GenerateRFDsCommand,
}

#[derive(StructOpt, Debug)]
enum GenerateRFDsCommand {
    Toc(RFDToc),
    Graph(RFDGraph)
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Generates RFD table of contents (Toc) to stdout")]
struct RFDToc {
    #[structopt(long, short, help = "")]
    intro: Option<String>,

    #[structopt(
        long,
        help = ""
    )]
    outro: Option<String>,

    #[structopt(long, short, help = "")]
    link_prefix: Option<String>,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Output format")]
    format: Option<TemplateExtension>
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create RFD Graph")]
struct RFDGraph {
    #[structopt(long, short, help = "")]
    intro: Option<String>,

    #[structopt(
        long,
        help = ""
    )]
    outro: Option<String>,

    #[structopt(long, short, help = "")]
    link_prefix: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Gathers ADR management commands"
)]
struct Adr {
    #[structopt(subcommand)]
    adr_command: AdrCommand,
}

#[derive(StructOpt, Debug)]
enum AdrCommand {
    Init(InitAdr),
    New(NewAdr),
    List(ListAdrs),
    Link(LinkAdrs),
    Generate(GenerateADRs),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "init", about = "Init ADR")]
struct InitAdr {
    #[structopt(long, short, help = "Directory to store ADRs")]
    directory: Option<String>,

    #[structopt(long, short, parse(try_from_str = parse_file_structure), help = "How ADRs should be structured")]
    structure: Option<FileStructure>,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    extension: Option<TemplateExtension>,
}

impl InitAdr {
    pub fn should_persist_settings(&self) -> bool {
        return self.directory.is_some() || self.extension.is_some();
    }
}

// TODO: should number just be a string and allow people to add their own conventions like leading zeros?
#[derive(StructOpt, Debug)]
#[structopt(name = "new", about = "New ADR")]
struct NewAdr {
    #[structopt(long, short, help = "ADR Number")]
    number: Option<i32>,
    
    // TODO: can we give title index so we dont have to specify --title or -t?
    #[structopt(long, short, help = "title of ADR")]
    title: String,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    extension: Option<TemplateExtension>,

    #[structopt(long, short, help = "A reference (number or partial filename) of a previous decision that the new decision supercedes. A Markdown link to the superceded ADR is inserted into the Status section. The status of the superceded ADR is changed to record that it has been superceded by the new ADR.")]
    supercede: Option<Vec<String>>,

    // Links the new ADR to a previous ADR.
    // TARGET is a reference (number or partial filename) of a
    // previous decision.
    // LINK is the description of the link created in the new ADR.
    // REVERSE-LINK is the description of the link created in the
    // existing ADR that will refer to the new ADR.
    #[structopt(long, short, help = "")]
    link: Option<Vec<String>>,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "list", about = "List ADRs")]
struct ListAdrs {

}

#[derive(StructOpt, Debug)]
#[structopt(name = "link", about = "Link ADRs")]
struct LinkAdrs {
    #[structopt(long, short, help = "Reference number of source ADR")]
    source: i32,
    
    // TODO: can we give title index so we dont have to specify --title or -t?
    #[structopt(long, short, help = "Description of the link created in the new ADR")]
    link: String,

    #[structopt(long, short, help = "Reference number of target ADR")]
    target: i32,

    #[structopt(long, short, help = "Description of the link created in the existing ADR that will refer to new ADR")]
    reverse_link: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers generate ADR commands")]
struct GenerateADRs {
    #[structopt(subcommand)]
    generate_adr_command: GenerateAdrsCommand,
}

#[derive(StructOpt, Debug)]
enum GenerateAdrsCommand {
    Toc(AdrToc),
    Graph(AdrGraph)
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Generates ADR table of contents (Toc) to stdout")]
struct AdrToc {
    #[structopt(long, short, help = "")]
    intro: Option<String>,

    #[structopt(
        long,
        help = ""
    )]
    outro: Option<String>,

    #[structopt(long, short, help = "")]
    link_prefix: Option<String>,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Output format")]
    format: Option<TemplateExtension>,

}


#[derive(StructOpt, Debug)]
#[structopt(about = "Create ADR Graph")]
struct AdrGraph {
    #[structopt(long, short, help = "")]
    intro: Option<String>,

    #[structopt(
        long,
        help = ""
    )]
    outro: Option<String>,

    #[structopt(long, short, help = "")]
    link_prefix: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Gathers Today I Learned (TIL) management commands"
)]
struct Til {
    #[structopt(subcommand)]
    til_command: TilCommand,
}

#[derive(StructOpt, Debug)]
enum TilCommand {
    Init(InitTil),
    New(NewTil),
    List(ListTils),
    Readme(BuildTilReadMe),
}


#[derive(StructOpt, Debug)]
#[structopt(about = "Init TIL")]
struct InitTil {
    #[structopt(long, short, help = "Directory to store TILs")]
    directory: Option<String>,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    extension: Option<TemplateExtension>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "New TIL")]
struct NewTil {
    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    // TODO: change to category
    #[structopt(short, long, help = "TIL category. Represents the directory to place TIL entry under")]
    category: String,

    #[structopt(long, short, help = "title of the TIL entry")]
    title: String,

    // TODO: what should the short be? We cant use the default 't' as it conflicts with title
    #[structopt(short = "T", long, help = "Additional tags associated with the TIL entry")]
    tags: Option<Vec<String>>,

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    extension: Option<TemplateExtension>,

    // TODO: should this also be a setting in TilSettings?
    #[structopt(short, long, help = "Whether to build a README after a new TIL is added")]
    readme: bool,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List TILs")]
struct ListTils {
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Build TIL ReadMe")]
struct BuildTilReadMe {

    #[structopt(long, short, parse(try_from_str = parse_template_extension), help = "Extension that should be used")]
    extension: Option<TemplateExtension>,

}

#[derive(Clone, Debug)]
struct TilEntry {
    topic: String,
    title: String,
    file_name: String,
    date: DateTime<Utc>,
}


#[derive(StructOpt, Debug)]
#[structopt(
    about = "Presentation commands"
)]
struct Presentation {

    #[structopt(long, help = "Output file path (or directory input-dir is passed)")]
    output_dir: Option<String>,

    #[structopt(long, short, help = "The base directory to find markdown and theme CSS")]
    input_dir: Option<String>,

// https://github.com/marp-team/marp-cli#options
// Marp CLI can be configured options with file, such as marp.config.js, marp.config.cjs, .marprc (JSON / YAML), and marp section of package.json
// package.json
// {
//     "marp": {
//       "inputDir": "./slides",
//       "output":" ./public",
//       "themeSet": "./themes"
//     }
//   }

// # .marprc.yml
// allowLocalFiles: true
// options:
//   looseYAML: false
//   markdown:
//     breaks: false
// pdf: true

// -c, --config-file, --config        Specify path to a configuration file
//                                            [string]
// --no-config-file, --no-config  Prevent looking up for a configuration file
//                                           [boolean]

    #[structopt(long, short, help = "Watch input markdowns for changes")]
    watch: bool,

    #[structopt(long, short, help = "Enable server mode")]
    server: bool,

    #[structopt(long, short, help = "Open preview window")]
    preview: bool,

}



lazy_static! {
    static ref OUTPUT_TYPES: HashMap<&'static str, Output> = {
        let mut map = HashMap::new();
        map.insert("json", Output::Json);
        map.insert("text", Output::Text);
        map
    };
}

fn parse_output(src: &str) -> Result<Output, String> {
    parse_enum(&OUTPUT_TYPES, src)
}

fn parse_file_structure(src: &str) -> Result<FileStructure, String> {
    parse_enum(&FILE_STRUCTURES, src)
}

fn parse_template_extension(src: &str) -> Result<TemplateExtension, String> {
    parse_enum(&TEMPLATE_EXTENSIONS, src)
}

fn parse_enum<A: Copy>(env: &'static HashMap<&'static str, A>, src: &str) -> Result<A, String> {
    match env.get(src) {
        Some(p) => Ok(*p),
        None => {
            let supported: Vec<&&str> = env.keys().collect();
            Err(format!(
                "Unsupported value: \"{}\". Supported values: {:?}",
                src, supported
            ))
        }
    }
}

fn print_output<A: std::fmt::Display + Serialize>(
    output: Output,
    value: A,
) -> Result<(), Box<dyn std::error::Error>> {
    match output {
        Output::Json => {
            serde_json::to_writer_pretty(std::io::stdout(), &value)?;
            Ok(())
        }
        Output::Text => {
            println!("{}", value);
            Ok(())
        }
        Output::Table => {
            todo!()
        }
    }
}


struct List<A>(Vec<A>);

impl<A> Debug for List<A>
where
    A: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for value in self.0.iter() {
            writeln!(f, "{:?}", value)?;
        }

        Ok(())
    }
}

impl<A> Display for List<A>
where
    A: Display,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for value in self.0.iter() {
            writeln!(f, "{}", value)?;
        }

        Ok(())
    }
}

impl<A> serde::ser::Serialize for List<A>
where
    A: serde::ser::Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for elem in self.0.iter() {
            seq.serialize_element(elem)?;
        }

        seq.end()
    }
}

/// Remove the `./` prefix from a path.
fn strip_current_dir(path: &Path) -> &Path {
    return path.strip_prefix(".").unwrap_or(path);
}


/// 
fn is_valid_file(path: &Path) -> bool {
    return TEMPLATE_EXTENSIONS.contains_key(&path.extension().unwrap().to_str().unwrap());
}

fn get_next_number(dir: &str, file_structure: FileStructure) -> i32 {
    if let Some(max) = get_allocated_numbers(dir, file_structure).iter().max() {
        return max + 1;
    } else {
        return 1;
    }
        
    // TODO: revisit iterator
    // return get_allocated_numbers(dir)
    //     .max()
    //     .unwrap() + 1;
}

fn is_number_reserved(dir: &str, number: i32, file_structure: FileStructure) -> bool {
    return get_allocated_numbers(dir, file_structure).contains(&number);
    
    // TODO: revisit iterator
    // return get_allocated_numbers(dir)
    //     .find(|n| n == &number)
    //     .is_some();
}

fn get_allocated_numbers(dir: &str, file_structure: FileStructure) -> Vec<i32> {
    match file_structure {
        FileStructure::Flat => {
            get_allocated_numbers_via_flat_files(dir)
        },

        FileStructure::Nested => {
            get_allocated_numbers_via_nested(dir)
        }
    }
}

// TODO: do we want a ReservedNumber type?
// TODO: would be nice to do this via an Iterator but having trouble with empty
// expected struct `std::iter::Map`, found struct `std::iter::Empty`
// using vec for now
fn get_allocated_numbers_via_nested(dir: &str) -> Vec<i32> {
    match fs::read_dir(dir) {
        Ok(files) => {
            return files
                .filter_map(Result::ok)
                .filter_map(|e| {
                    // TODO: is there a better way to do this?
                    if e.file_type().is_ok() && e.file_type().unwrap().is_dir() {
                        return Some(e.file_name().to_string_lossy().parse::<i32>().unwrap());
                    } else {
                        None
                    }
                })
                .collect();
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // return std::iter::empty();
            return Vec::new();
        }
        Err(e) => {
            panic!("Error reading directory {}. Error: {}", dir, e);
        }  
    }
}

// TODO: would be nice to do this via an Iterator but having trouble with empty
// expected struct `std::iter::Map`, found struct `std::iter::Empty`
// using vec for now
fn get_allocated_numbers_via_flat_files(dir: &str) -> Vec<i32> { //impl Iterator<Item = i32> {
    
    let mut allocated_numbers = Vec::new();
    for entry in WalkDir::new(&dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter(|e| is_valid_file(&e.path())) {
        // The only way I can get this to pass the borrow checker is first mapping 
        // to file_name and then doing the rest. 
        // I'm probably doing this wrong and should review later
        let file_name = entry.file_name();
        let ss = file_name.to_str().unwrap();
        let first_space_index = ss.find("-").expect("didnt find a hyphen");
        let num: String = ss.chars().take(first_space_index).collect();
        allocated_numbers.push(num.parse::<i32>().unwrap());
        
    }

    return allocated_numbers;
}

/// get output based on following order of precednece
/// output argument (--output)
/// env var DOCTAVIOUS_DEFAULT_OUTPUT
/// config file overrides output default -- TOOD: implement
/// Output default
fn get_output(opt_output: Option<Output>) -> Output {
    match opt_output {
        Some(o) => o,
        None => {
            match env::var("DOCTAVIOUS_DEFAULT_OUTPUT") {
                Ok(val) => parse_output(&val).unwrap(), // TODO: is unwrap ok here?
                Err(_) => Output::default() // TODO: implement output via settings/config file
            }
        }
    }
}

fn init_dir(dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: create_dir_all doesnt appear to throw AlreadyExists. Confirm this
    // I think this is fine just need to make sure that we dont overwrite initial file
    let create_dir_result = fs::create_dir_all(dir);
    match create_dir_result {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => {
            eprintln!("the directory {} already exists", dir);
            return Err(e.into());
        }
        Err(e) => {
            eprintln!("Error occurred creating directory {}: {}", dir, e);
            return Err(e.into());
        }   
    }
}

// TODO: is there a more concise way to do this?
fn build_path(
    dir: &str, 
    title: &str, 
    reserved_number: &str, 
    extension: TemplateExtension, 
    file_structure: FileStructure
) -> PathBuf {
    match file_structure {
        FileStructure::Flat => {
            let slug = slugify(&title);
            let file_name = format!("{}-{}", reserved_number, slug);
            return Path::new(dir)
                .join(file_name)
                .with_extension(extension.to_string());
        }

        FileStructure::Nested => {
            return Path::new(dir)
                .join(&reserved_number)
                .join("README.")
                .with_extension(extension.to_string());
        }
    };
}

fn get_content(
    dir: &str, 
    number: &str,
    file_structure: FileStructure
) -> io::Result<String> {
    match file_structure {
        FileStructure::Flat => {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                for key in TEMPLATE_EXTENSIONS.keys() {    
                    let file_name = entry.file_name().to_str().unwrap().to_owned();
                    let formatted_number = format!("{}-", number);
                    if file_name.starts_with(&formatted_number) {
                        return fs::read_to_string(path.as_path());
                    }
                    // if let Some(ref file_name) = file_name {
                    //     if file_name.starts_with(&number) {
                    //         return fs::read_to_string(path.as_path());
                    //     }
                    // }
                    // if file_name.starts_with(format!("{}-", number)) {
                    //     return fs::read_to_string(path.as_path());
                    // }
                    // if path.file_name().to_string_lossy().starts_with(format!("{}-", number)) {
                    //     return fs::read_to_string(path.as_path());
                    // }
                }
            }
            return Err(std::io::Error::new(ErrorKind::InvalidData, "invalid file"));
        }

        FileStructure::Nested => {
            for key in TEMPLATE_EXTENSIONS.keys() {
                let p = Path::new(dir)
                    .join(&number)
                    .join("README.")
                    .with_extension(key);
                if p.exists() {
                    return fs::read_to_string(p);
                }
            }

            return Err(std::io::Error::new(ErrorKind::InvalidData, "invalid file"));
        }
    };
}

fn ensure_path(path: &PathBuf)-> Result<(), Box<dyn std::error::Error>> {
    if path.exists() {
        println!("File already exists at: {}", path.to_string_lossy());
        print!("Overwrite? (y/N): ");
        io::stdout().flush()?;
        let mut decision = String::new();
        io::stdin().read_line(&mut decision)?;
        if decision.trim().eq_ignore_ascii_case("Y") {
            return Ok(());
        } else {
            return Err(format!("Unable to write config file to: {}", path.to_string_lossy()).into());
        }
    } else {
        let parent_dir = path.parent();
        match parent_dir {
            Some(path) => {
                fs::create_dir_all(path)?;
                Ok(())
            },
            None => Err(format!("Unable to write file to: {}", path.to_string_lossy()).into())
        }
    }
}

fn reserve_number(dir: &str, number: Option<i32>, file_structure: FileStructure) -> Result<i32, Box<dyn std::error::Error>> {
    if let Some(i) = number {
        if is_number_reserved(dir, i, file_structure) {
            // TODO: the prompt to overwrite be here?
            // TODO: return custom error NumberAlreadyReservedErr(number has already been reserved);
            eprintln!("{} has already been reserved", i);
            return Err(format!("{} has already been reserved", i).into());
        }
        return Ok(i);
    } else {
        return Ok(get_next_number(dir, file_structure));
    }
}

fn get_template(dir: &str, extension: TemplateExtension, default_template_path: &str) -> PathBuf {
    let custom_template = Path::new(dir)
        .join("template")
        .with_extension(extension.to_string());

    let template = if custom_template.exists() { 
        custom_template 
    } else {
        Path::new(default_template_path)
            .with_extension(extension.to_string())
            .to_path_buf() 
    };

    return template;
}

fn slugify(string: &str) -> String {
    let separator_char = '-';
    let separator = separator_char.to_string();

    let string: String = unidecode(string.into())
        .to_lowercase()
        .trim_matches(separator_char)
        .replace(' ', &separator);

    let mut slug = Vec::with_capacity(string.len());
    let mut is_sep = true;

    for x in string.chars() {
        match x {
            'a'..='z' | '0'..='9' => {
                is_sep = false;
                slug.push(x as u8);
            }
            _ => {
                if !is_sep {
                    is_sep = true;
                    slug.push(separator_char as u8);
                }
            }
        }
    }

    if slug.last() == Some(&(separator_char as u8)) {
        slug.pop();
    }

    let s = String::from_utf8(slug).unwrap();
    s.trim_end_matches(separator_char).to_string();
    s
}

fn new_rfd(number: Option<i32>, title: String, extension: TemplateExtension) -> Result<(), Box<dyn std::error::Error>> {

    let dir = SETTINGS.get_rfd_dir();
    let template = get_template(&dir, extension, DEFAULT_RFD_TEMPLATE_PATH);
    let reserve_number = reserve_number(&dir, number, SETTINGS.get_rfd_structure())?;
    let formatted_reserved_number = format_number(reserve_number);
    let rfd_path = build_path(&dir, &title, &formatted_reserved_number, extension, SETTINGS.get_rfd_structure());
    ensure_path(&rfd_path)?;

    // TODO: supersceded
    // TODO: reverse links
    
    let mut starting_content = fs::read_to_string(&template).expect(&format!("failed to read file {}.", &template.to_string_lossy()));
    starting_content = starting_content.replace("<NUMBER>", &formatted_reserved_number);
    starting_content = starting_content.replace("<TITLE>", &title);
    starting_content = starting_content.replace("<DATE>", &Utc::now().format("%Y-%m-%d").to_string());
    
    let edited = edit::edit(&starting_content)?;
    fs::write(&rfd_path, edited)?;

    return Ok(())
}

fn get_leading_character(extension: TemplateExtension) -> char {
    return match extension {
        TemplateExtension::Markdown => '#',
        TemplateExtension::Asciidoc => '='
    };
}

fn format_number(number: i32) -> String {
    return format!("{:0>4}", number);
}

fn new_adr(
    number: Option<i32>, 
    title: String, 
    extension: TemplateExtension
    // supercedes: Option<Vec<String>>, 
    // links: Option<Vec<String>>
) -> Result<(), Box<dyn std::error::Error>> {
    let dir = SETTINGS.get_adr_dir();
    let template = get_template(&dir, extension, DEFAULT_ADR_TEMPLATE_PATH);
    let reserve_number = reserve_number(&dir, number, SETTINGS.get_adr_structure())?;
    let formatted_reserved_number = format_number(reserve_number);
    let adr_path = build_path(&dir, &title, &formatted_reserved_number, extension, SETTINGS.get_adr_structure());
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

    let mut starting_content = fs::read_to_string(&template).expect(&format!("failed to read file {}.", &template.to_string_lossy()));
    starting_content = starting_content.replace("<NUMBER>", &reserve_number.to_string());
    starting_content = starting_content.replace("<TITLE>", &title);
    starting_content = starting_content.replace("<DATE>", &Utc::now().format("%Y-%m-%d").to_string());
    starting_content = starting_content.replace("<STATUS>", "Accepted");

    let edited = edit::edit(&starting_content)?;
    fs::write(&adr_path, edited)?;

    return Ok(())
}

fn list(dir: &str, opt_output: Option<Output>) {

    match fs::metadata(&dir) {
        Ok(_) => {
            let mut f: Vec<_> = WalkDir::new(&dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .filter(|f| is_valid_file(&f.path()))
                .map(|f| String::from(strip_current_dir(&f.path()).to_str().unwrap()))
                .collect();

            f.sort();
            print_output(get_output(opt_output), List(f)).unwrap();
        },
        Err(e) => match e.kind() {
            ErrorKind::NotFound => eprintln!("the {} directory should exist", dir),
            _ => eprintln!("Error occurred: {:?}", e)
        }
    }
}

// TODO: this doesnt work with frontmatter
fn title_string<R>(rdr: R, extension: TemplateExtension) -> String
    where R: BufRead,
{
    let leading_char = get_leading_character(extension);
    for line in rdr.lines() {
        let line = line.unwrap();
        if line.starts_with(&format!("{} ", leading_char)) {
            let last_hash = line
            .char_indices()
            .skip_while(|&(_, c)| c == leading_char)
            .next()
            .map_or(0, |(idx, _)| idx);
    
            // Trim the leading hashes and any whitespace
            return line[last_hash..].trim().to_string();
        }
    }

    // TOOD: need file name
    panic!("Unable to find title for file");

    // let mut first_line = String::new();

    // rdr.read_line(&mut first_line).expect("Unable to read line");

    // let leading_char = get_leading_character(extension);

    // let last_hash = first_line
    //     .char_indices()
    //     .skip_while(|&(_, c)| c == leading_char)
    //     .next()
    //     .map_or(0, |(idx, _)| idx);

    // // Trim the leading hashes and any whitespace
    // first_line[last_hash..].trim().into()
}

// TOOD: pass in header
fn build_toc(
    dir: &str, 
    extension: TemplateExtension, 
    intro: Option<String>, 
    outro: Option<String>, 
    link_prefix: Option<String>
) {
    let leading_char = get_leading_character(extension);
    let mut content = String::new();
    content.push_str(&format!("{} {}\n", leading_char, "Architecture Decision Records"));
    content.push_str("\n");
    
    if intro.is_some() {
        content.push_str(&intro.unwrap());
        content.push_str("\n\n");
    }

    match fs::metadata(&dir) {
        Ok(_) => {
            let link_prefix = link_prefix.unwrap_or(String::new());
            for entry in WalkDir::new(&dir)
                .sort_by(|a,b| a.file_name().cmp(b.file_name()))
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .filter(|f| is_valid_file(&f.path())) {

                let file = match fs::File::open(&entry.path()) {
                    Ok(file) => file,
                    Err(_) => panic!("Unable to read file {:?}", entry.path()),
                };
        
                let buffer = BufReader::new(file);
                let title = title_string(buffer, extension);

                // TODO: should this be a relative path or just the file name?
                // adr tools has just the file name
                content.push_str(&format!("* [{}]({}{})", title, link_prefix, entry.path().display()));
                content.push_str("\n");
            }
        },
        Err(e) => match e.kind() {
            ErrorKind::NotFound => eprintln!("the {} directory should exist", dir),
            _ => eprintln!("Error occurred: {:?}", e)
        }
    }

    if outro.is_some() {
        content.push_str(&outro.unwrap());
    }

    print!("{}", content);
}

fn build_til_readme(dir: &str) -> io::Result<()> {
    let mut all_tils: BTreeMap<String, Vec<TilEntry>> = BTreeMap::new();
    for entry in WalkDir::new(&dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file()) {
    
        // skip files that are under til dir
        if Path::new(dir) == entry.path().parent().unwrap() {
            continue;
        }

        // TODO: handle unwraps better
        let topic = entry.path()
                .parent().unwrap()
                .file_name().unwrap().to_string_lossy().into_owned();
        
        if !all_tils.contains_key(&topic) {
            // TODO: is there a way to avoid this clone?
            all_tils.insert(topic.clone(), Vec::new());
        }

        let file_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
        let extension = parse_template_extension(entry.path().extension().unwrap().to_str().unwrap()).unwrap();
        let file = match fs::File::open(&entry.path()) {
            Ok(file) => file,
            Err(_) => panic!("Unable to read title from {:?}", entry.path()),
        };

        let buffer = BufReader::new(file);
        let title = title_string(buffer, extension);

        all_tils.get_mut(&topic).unwrap().push(TilEntry {
            topic: topic,
            title: title,
            file_name: file_name,
            date: DateTime::from(entry.metadata()?.created()?)
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
    lw.write_all(format!("* Categories: {}\n", all_tils.keys().len()).as_bytes())?;
    lw.write_all(format!("* TILs: {}\n", til_count).as_bytes())?;
    lw.write_all(b"\n")?;

    // TODO: do we want to list categories?

    for category in all_tils.keys().cloned() {
        lw.write_all(format!("## {}\n\n", &category).as_bytes())?;
        let mut tils = all_tils.get(&category).unwrap().to_vec();
        tils.sort_by_key(|e| e.title.clone());
        for til in tils {
            // TODO: should we just use title instead of file_name for link?
            lw.write_all(format!("* [{}]({}/{}) {} ({})", til.file_name, category, til.file_name, til.title, til.date.format("%Y-%m-%d")).as_bytes())?;
            lw.write_all(b"\n")?;
        }

        lw.write_all(b"\n")?;
    }

    // TODO: handle this
    return lw.flush();
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let opt = Opt::from_args();
    if opt.debug {
        std::env::set_var("RUST_LOG", "eventfully=debug");
        env_logger::init();
    }

    match opt.cmd {
        Command::Adr(adr) => match adr.adr_command {
            AdrCommand::Init(params) => {

                // TODO: need to handle initing multiple times better

                if params.should_persist_settings() {
                    let mut settings = match load_settings() {
                        Ok(settings) => settings,
                        Err(_) => Default::default()
                    };

                    let adr_settings = AdrSettings {
                        dir: params.directory.clone(),
                        structure: params.structure,
                        template_extension: params.extension,
                    };
                    settings.adr_settings = Some(adr_settings);
                    persist_settings(settings)?;
                }

                let dir = match params.directory {
                    None => DEFAULT_ADR_DIR,
                    Some(ref d) => d,
                };

                init_dir(dir)?;

                return new_adr(Some(1), "Record Architecture Decisions".to_string(), SETTINGS.get_adr_template_extension());
            }

            AdrCommand::New(params) => {
                init_dir(SETTINGS.get_adr_dir())?;

                let extension = match params.extension {
                    Some(v) => v,
                    None => SETTINGS.get_adr_template_extension()
                };

                return new_adr(params.number, params.title, extension);
            }

            AdrCommand::List(_) => {
                list(SETTINGS.get_adr_dir(), opt.output);
            }

            AdrCommand::Link(params) => {
                // get file. needs to support both structures and extensions
                let source_content = get_content(
                    SETTINGS.get_adr_dir(), 
                    &format_number(params.source),
                    SETTINGS.get_adr_structure()
                );

                let arena = Arena::new();

                let root = parse_document(
                    &arena,
                    &source_content?, //"This is my input.\n\n1. Also my input.\n2. Certainly my input.\n", 
                    &ComrakOptions::default());
                
                fn iter_nodes<'a, F>(node: &'a AstNode<'a>, f: &F)
                    where F : Fn(&'a AstNode<'a>) {
                    f(node);

                    for c in node.children() {
                        // println!("{:?}", c.data);
                        iter_nodes(c, f);
                    }
                }
                
                iter_nodes(root, &|node| {
                    match &mut node.data.borrow_mut().value {
                        &mut NodeValue::Text(ref mut text) => {
                            let orig = std::mem::replace(text, vec![]);
                            let t = String::from_utf8(orig).unwrap();
                            *text = t.as_bytes().to_vec();

                            // flush this out to make it more robust. 
                            // such as check parent is a header
                            // check sibling is paragraph/text
                            if "Status" == t {
                                match node.parent() {
                                    Some(parent) => {
                                        let para = Ast::new(NodeValue::Paragraph);
                                        let p_node = arena.alloc(Node::new(RefCell::new(para)));
                                        
                                        let e = Ast::new(NodeValue::Text("new entry into status".as_bytes().to_vec()));
                                        let node = arena.alloc(Node::new(RefCell::new(e)));

                                        p_node.append(node);
                                        
                                        parent.next_sibling().unwrap().insert_after(p_node);
                                    },
                                    None => {
                                        println!("node parent was none???");
                                    }
                                }
                            }

                            
                        }
                        _ => (),
                    }
                });

                let z = Path::new(SETTINGS.get_adr_dir())
                    .join("temp-link")
                    .with_extension("md");

                let mut buffer = File::create(z)?;
                // let mut markdown = vec![];
                format_commonmark(root, &ComrakOptions::default(), &mut buffer).unwrap();
                // fs::write(&adr_path, markdown)?;

                let mut html = vec![];
                format_html(root, &ComrakOptions::default(), &mut html).unwrap();

                println!("{}", String::from_utf8(html).unwrap());
            }

            AdrCommand::Generate(generate) => match generate.generate_adr_command {
                GenerateAdrsCommand::Toc(params) => {
                    let dir = SETTINGS.get_adr_dir();

                    // # Architecture Decision Records

                    // * [1. hi](0001-hi.md)
                    // * [2. lo](0002-lo.md)
                    
                    let extension = match params.format {
                        Some(v) => v,
                        None => SETTINGS.get_adr_template_extension()
                    };

                    build_toc(dir, extension, params.intro, params.outro, params.link_prefix);
                }

                GenerateAdrsCommand::Graph(params) => {
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
        },

        Command::Presentation(params) => {
            // TODO: implement
            let output_dir = match params.output_dir {
                None => "",
                Some(ref o) => o,
            };

            let input_dir = match params.input_dir {
                None => "",
                Some(ref i) => i,
            };
    
        },

        Command::RFD(rfd) => match rfd.rfd_command {
            RFDCommand::Init(params) => {

                // TODO: need to handle initing multiple times better

                if params.should_persist_settings() {
                    let mut settings = match load_settings() {
                        Ok(settings) => settings,
                        Err(_) => Default::default()
                    };

                    let rfd_settings = RFDSettings {
                        dir: params.directory.clone(),
                        structure: params.structure,
                        template_extension: params.extension,
                    };
                    settings.rfd_settings = Some(rfd_settings);
                    persist_settings(settings)?;
                }

                let dir = match params.directory {
                    None => DEFAULT_RFD_DIR,
                    Some(ref d) => d,
                };

                init_dir(dir)?;

                return new_rfd(Some(1), "Use RFDs ...".to_string(), SETTINGS.get_rfd_template_extension());
            }

            RFDCommand::New(params) => {
                init_dir(SETTINGS.get_rfd_dir())?;

                let extension = match params.extension {
                    Some(v) => v,
                    None => SETTINGS.get_rfd_template_extension()
                };

                return new_rfd(params.number, params.title, extension);
            }

            RFDCommand::List(_) => {
                list(SETTINGS.get_rfd_dir(), opt.output);
            }
            
            RFDCommand::Generate(generate) => match generate.generate_rfd_command {
                GenerateRFDsCommand::Toc(params) => {
                    let dir = SETTINGS.get_adr_dir();

                    // # Architecture Decision Records

                    // * [1. hi](0001-hi.md)
                    // * [2. lo](0002-lo.md)
                    
                    let extension = match params.format {
                        Some(v) => v,
                        None => SETTINGS.get_adr_template_extension()
                    };

                    build_toc(dir, extension, params.intro, params.outro, params.link_prefix);
                }

                GenerateRFDsCommand::Graph(params) => {

                }
            }
        },

        Command::Til(til) => match til.til_command {
            TilCommand::Init(params) => {
                let dir = match params.directory {
                    None => {
                        SETTINGS.get_til_dir()
                    },
                    Some(ref d) => {
                        let mut settings = match load_settings() {
                            Ok(settings) => settings,
                            Err(_) => Default::default()
                        };

                        let til_settings = TilSettings {
                            dir: Some(d.to_string()),
                            template_extension: None,
                        };
                        settings.til_settings = Some(til_settings);
                        persist_settings(settings)?;
                        d
                    },
                };

                init_dir(dir)?;
            }

            TilCommand::New(params) => {
                let dir = SETTINGS.get_til_dir();
                init_dir(&dir)?;

                let extension = match params.extension {
                    Some(v) => v,
                    None => SETTINGS.get_til_template_extension()
                };

                let file_name = params.title.to_lowercase();
                let path = Path::new(dir)
                        .join(params.category)
                        .join(file_name)
                        .with_extension(extension.to_string());

                if path.exists() {
                    eprintln!("File {} already exists", path.to_string_lossy());
                } else {

                    let leading_char = get_leading_character(extension);

                    let mut starting_content = format!("{} {}\n", leading_char, params.title);
                    if params.tags.is_some() {
                        starting_content.push_str("\ntags: ");
                        starting_content.push_str(params.tags.unwrap().join(" ").as_str());
                    }
    
                    let edited = edit::edit(&starting_content)?;

                    fs::create_dir_all(path.parent().unwrap())?;
                    fs::write(&path, edited)?;

                    if params.readme {
                        build_til_readme(&dir)?;
                    }
                }
                
            }

            TilCommand::List(_) => {
                list(SETTINGS.get_til_dir(), opt.output);
            }

            TilCommand::Readme(_) => {
                build_til_readme(SETTINGS.get_til_dir())?;
            }
        }

    };

    Ok(())

}