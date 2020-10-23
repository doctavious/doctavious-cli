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

#[derive(StructOpt, Debug)]
#[structopt(
    name = "eventfully",
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
    
    // TODO: make option and default to README? 
    // this could also be a setting
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
    RFDToc(RFDToc),
    RFDGraph(RFDGraph)
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create RFD ToC")]
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
    Generate(GenerateADRs),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "init", about = "Init ADR")]
struct InitAdr {
    #[structopt(long, short, help = "Directory to store ADRs")]
    directory: Option<String>,

    // TODO: should we default here?
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
#[structopt(about = "Gathers generate ADR commands")]
struct GenerateADRs {
    #[structopt(subcommand)]
    generate_adr_command: GenerateAdrsCommand,
}

#[derive(StructOpt, Debug)]
enum GenerateAdrsCommand {
    AdrToc(AdrToc),
    AdrGraph(AdrGraph)
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create ADR ToC")]
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
struct TilEntry<'a> {
    topic: String,
    title: String,
    file_name: String,
    base_name: &'a str,
    description: &'a str,
    date: DateTime<Utc>,
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

// TODO: share more between get_next_number and is_number_reserved
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
    let formatted_reserved_number = format!("{:0>4}", reserve_number);
    let rfd_path = build_path(&dir, &title, &formatted_reserved_number, extension, SETTINGS.get_rfd_structure());
    ensure_path(&rfd_path)?;

    // TODO: supersceded
    // TODO: reverse links
    
    match fs::read_to_string(&template) {
        Err(e) => panic!("Error occurred reading template file {}. {}", template.to_string_lossy(), e),
        Ok(mut contents) => {
            contents = contents.replace("<NUMBER>", &formatted_reserved_number);
            contents = contents.replace("<TITLE>", &title);
        
            fs::write(&rfd_path, contents)?;
            return Ok(());
        }
    }

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
    let formatted_reserved_number = format!("{:0>4}", reserve_number);
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

    let mut contents = fs::read_to_string(template).expect("Something went wrong reading the file");
    contents = contents.replace("<NUMBER>", &reserve_number.to_string());
    contents = contents.replace("<TITLE>", &title);
    contents = contents.replace("<DATE>", &Utc::now().format("%Y-%m-%d").to_string());
    contents = contents.replace("<STATUS>", "Accepted");

    let mut file = File::create(adr_path)?;
    file.write_all(contents.as_bytes())?;

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

fn title_string<R>(mut rdr: R, extension: TemplateExtension) -> String
    where R: BufRead,
{
    let mut first_line = String::new();

    rdr.read_line(&mut first_line).expect("Unable to read line");

    let leading_char = match extension {
        TemplateExtension::Markdown => '#',
        TemplateExtension::Asciidoc => '='
    };

    let last_hash = first_line
        .char_indices()
        .skip_while(|&(_, c)| c == leading_char)
        .next()
        .map_or(0, |(idx, _)| idx);

    // Trim the leading hashes and any whitespace
    first_line[last_hash..].trim().into()
}


fn build_til_readme(dir: &str) -> io::Result<()> {
    // TODO: build readme
    // let mut current_topic = String::new();
    let mut all_tils: BTreeMap<String, Vec<TilEntry>> = BTreeMap::new();
    for entry in WalkDir::new(&dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file()) {
        
        // let f_name = String::from(entry.file_name().to_string_lossy());
        // // let f_name = entry.file_name().to_string_lossy();
        // if entry.file_type().is_dir() {
        //     // current_topic = entry.path().file_name().unwrap().to_string_lossy().into_owned();
        //     current_topic = entry.file_name().to_string_lossy().into_owned();
        //     // current_topic = entry.file_name().to_str().unwrap().to_string();
        //     if !all_tils.contains_key(current_topic.as_str()) {
        //         all_tils.insert(current_topic.as_str(), Vec::new());
        //     }
        // } else {

        //     // file_name returns an OsString and .to_str return Option<&str>
        //     // This indicates that the method will return a borrowed reference to a &str. 
        //     // The DirEntry struct is the owner of the string.
        //     // This means that any references into the DirEntry will no longer be valid.
        //     // String owns the string inside of it
        //     let file_name = entry.file_name().to_str().unwrap().into();
        //     let file = match fs::File::open(entry.path()) {
        //         Ok(file) => file,
        //         Err(_) => panic!("Unable to read title from {:?}", entry.path()),
        //     };
        //     let buffer = BufReader::new(file);
        //     let title = title_string(buffer);

        //     let tilEntry = TilEntry {
        //         topic: current_topic.to_string(),
        //         title: title,
        //         description: "",
        //         file_name: file_name,
        //         base_name: "",
        //         date: SystemTime::now() //entry.metadata()?.created()?
        //     };

        //     all_tils.get_mut(current_topic.as_str()).unwrap().push(tilEntry);
        // }
    
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
            description: "",
            file_name: file_name,
            base_name: "",
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
    lw.write_all(format!("* TILs: {}\n", til_count).as_bytes())?;
    lw.write_all(format!("* Topics: {}\n", all_tils.keys().len()).as_bytes())?;
    lw.write_all(b"\n")?;

    // TODO: do we want to list categories?

    for topic in all_tils.keys().cloned() {
        lw.write_all(format!("## {}\n\n", &topic).as_bytes())?;
        let mut tils = all_tils.get(&topic).unwrap().to_vec();
        tils.sort_by_key(|e| e.title.clone());
        for til in tils {
            lw.write_all(format!("* [{}]({}/{}) {} ({})", til.title, topic, til.file_name, til.description, til.date.format("%Y-%m-%d")).as_bytes())?;
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

            AdrCommand::Generate(generate) => match generate.generate_adr_command {
                GenerateAdrsCommand::AdrToc(params) => {
                    // -i INTRO  precede the table of contents with the given INTRO text.
                    // -o OUTRO  follow the table of contents with the given OUTRO text.
                    // -p LINK_PREFIX
                    //           prefix each decision file link with LINK_PREFIX.
                    
                    // Both INTRO and OUTRO must be in Markdown format.

                    // Generates a table of contents in Markdown format to stdout.
                }

                GenerateAdrsCommand::AdrGraph(params) => {
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
                GenerateRFDsCommand::RFDToc(params) => {

                }

                GenerateRFDsCommand::RFDGraph(params) => {

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

                    let leading_char = match extension {
                        TemplateExtension::Markdown => '#',
                        TemplateExtension::Asciidoc => '='
                    };

                    let mut content = format!("{} {}\n", leading_char, params.title);
                    if params.tags.is_some() {
                        content.push_str("\ntags: ");
                        content.push_str(params.tags.unwrap().join(" ").as_str());
                    }
    
                    // let edited = edit::edit(&content)?;
                    // println!("after editing: {}", edited);

                    fs::create_dir_all(path.parent().unwrap())?;
                    fs::write(&path, content)?;

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