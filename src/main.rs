#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use serde_derive::Serialize;
use serde_derive::Deserialize;

use structopt::StructOpt;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use chrono::prelude::*;

use std::env;


// TODO: for ADR and RFC make sure template can be either markdown or asciidoc
// TODO: Automatically update readme TOC
// Update CVS file? From Oxide - we automatically update a CSV file of all the RFDs along with their state, links, and other information in the repo for easy parsing.
// TODO: configuration
// TODO: output options
// config file
// env var DOCTAVIOUS_DEFAULT_OUTPUT - overrides config
// --output  - overrides env var and config
// TODO: RFC / ADR meta frontmatter

// TODO: why do I get a % at the end when using json output

// TODO: add configuration option for whether to use md or adoc
// probably makes sense to make enum. default to md

// TODO: review https://github.com/joyent/rfd/blob/master/tools/rfdlint

// TODO: do we need an init for RFC? What would it include? init should at least create .doctavious


#[derive(StructOpt, Debug)]
#[structopt(
    name = "eventfully",
)]
pub struct Opt {
    #[structopt(long, help = "Prints a verbose output during the program execution", global = true)]
    debug: bool,

    #[structopt(long, short, default_value = "json", parse(try_from_str = parse_output), help = "How a command output should be rendered", global = true)]
    output: Output,

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

#[derive(Debug, Copy, Clone)]
pub enum TemplateExtension {
    Markdown,
    Asciidoc,
}

impl Default for TemplateExtension {
    fn default() -> Self { TemplateExtension::Markdown }
}

lazy_static! {
    static ref TEMPLATE_EXTENSIONS: HashMap<&'static str, TemplateExtension> = {
        let mut map = HashMap::new();
        map.insert("md", TemplateExtension::Markdown);
        map.insert("asciidoc", TemplateExtension::Asciidoc);
        map
    };
}

// TODO: better way to do this? Do we want to keep a default settings file in doctavious dir?
pub static DEFAULT_ADR_DIR: &str = "docs/adr";
pub static DEFAULT_RFC_DIR: &str = "docs/rfc";

// TODO: should this include output?
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Settings {
    adr_dir: Option<String>,
    rfc_dir: Option<String>,
}

impl Settings {

    fn get_adr_dir(&self) -> &str {
        // an alternative to the below is 
        // return self.adr_dir.as_deref().or(Some(DEFAULT_ADR_DIR)).unwrap();
        if let Some(adr_dir) = &self.adr_dir {
            return adr_dir;
        } else {
            return DEFAULT_ADR_DIR;
        }
    }

    fn get_rfc_dir(&self) -> &str {
        // an alternative to the below is 
        // return self.rfc_dir.as_deref().or(Some(DEFAULT_RFC_DIR)).unwrap();
        if let Some(rfc_dir) = &self.rfc_dir {
            return rfc_dir;
        } else {
            return DEFAULT_RFC_DIR;
        }
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

#[derive(StructOpt, Debug)]
enum Command {
    Rfc(Rfc),
    Adr(Adr),
}

#[derive(StructOpt, Debug)]
#[structopt(
    about = "Gathers RFC management commands"
)]
struct Rfc {
    #[structopt(subcommand)]
    rfc_command: RfcCommand,
}

#[derive(StructOpt, Debug)]
enum RfcCommand {
    Init(InitRfc),
    New(NewRfc),
    List(ListRfcs),
    Generate(GenerateRfcs),
}


#[derive(StructOpt, Debug)]
#[structopt(about = "Init RFC")]
struct InitRfc {
    #[structopt(long, short, help = "Directory to store RFCs")]
    directory: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "New RFC")]
struct NewRfc {
    #[structopt(long, short, help = "RFC number")]
    number: Option<i32>,
    
    #[structopt(long, short, help = "title of RFC")]
    title: String,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "List RFCs")]
struct ListRfcs {
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers generate RFC commands")]
struct GenerateRfcs {
    #[structopt(subcommand)]
    generate_rfc_command: GenerateRfcsCommand,
}

#[derive(StructOpt, Debug)]
enum GenerateRfcsCommand {
    RfcToc(RfcToc),
    RfcGraph(RfcGraph)
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create RFC ToC")]
struct RfcToc {
    #[structopt(long, short, help = "")]
    intro: Option<String>,

    #[structopt(
        long,
        help = "Set this parameter if you don't want to give your password safely (non-interactive)"
    )]
    outro: Option<String>,

    #[structopt(long, short, help = "")]
    link_prefix: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Create RFC Graph")]
struct RfcGraph {
    #[structopt(long, short, help = "")]
    intro: Option<String>,

    #[structopt(
        long,
        help = "Set this parameter if you don't want to give your password safely (non-interactive)"
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
}

#[derive(StructOpt, Debug)]
#[structopt(name = "new", about = "New ADR")]
struct NewAdr {
    #[structopt(long, short, help = "title of ADR")]
    number: Option<i32>,
    
    #[structopt(long, short, help = "title of ADR")]
    title: String,
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
        help = "Set this parameter if you don't want to give your password safely (non-interactive)"
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
        help = "Set this parameter if you don't want to give your password safely (non-interactive)"
    )]
    outro: Option<String>,

    #[structopt(long, short, help = "")]
    link_prefix: Option<String>,
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

impl<A> std::fmt::Debug for List<A>
where
    A: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for value in self.0.iter() {
            writeln!(f, "{:?}", value)?;
        }

        Ok(())
    }
}

impl<A> std::fmt::Display for List<A>
where
    A: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename)
        .extension()
        .and_then(OsStr::to_str)
}


/// Remove the `./` prefix from a path.
fn strip_current_dir(path: &Path) -> &Path {
    return path.strip_prefix(".").unwrap_or(path);
}


/// 
fn is_valid_file(path: &Path) -> bool {
    return TEMPLATE_EXTENSIONS.contains_key(&path.extension().unwrap().to_str().unwrap());
}

// TODO: terrible name
///
fn get_max_id(dir: &str) -> i32 {
    let files = fs::read_dir(dir).expect("Directory should exist");
    return files
        .filter_map(Result::ok)
        .filter(|f| is_valid_file(&f.path()))
        .map(|f| f.file_name())
        .map(|s| {
            // The only way I can get this to pass the borrow checker is first mapping 
            // to file_name and then doing the rest. I'm probably doing this wrong and
            // should review later
            let ss = s.to_str().unwrap();
            let first_space_index = ss.find(" ").expect("didnt find a space");
            let num:String = ss.chars().take(first_space_index).collect();
            return num.parse::<i32>().unwrap();
        })
        .max()
        .unwrap();
}

/// get output based on following order of precednece
/// Output default
/// config file overrides output default -- TOOD: implement
/// env var DOCTAVIOUS_DEFAULT_OUTPUT - overrides config
/// --output  - overrides env var and config
fn get_output(opt_output: Option<Output>) -> Output {
    match opt_output {
        Some(o) => o,
        None => {
            let key = "DOCTAVIOUS_DEFAULT_OUTPUT";
            match env::var(key) {
                Ok(val) => parse_output(&val).unwrap(), // TODO: is unwrap ok here?
                Err(_) => Output::default() // TODO: implement output via settings/config file
            }
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let opt = Opt::from_args();
    if opt.debug {
        std::env::set_var("RUST_LOG", "eventfully=debug");
        env_logger::init();
    }

    match opt.cmd {
        Command::Rfc(rfc) => match rfc.rfc_command {
            RfcCommand::Init(params) => {
                
            }

            RfcCommand::New(params) => {
                // allocate new RFC number
                // create directory with rfc number (left pad 2 zeros)
                // create readme file within directory via template

            }

            RfcCommand::List(_) => {
                 let dir = SETTINGS.get_rfc_dir();
                 
                 // TODO: do we want to order?
                 // read_dir does not guarantee any specific order
                 // in order to sort would need to read into a Vec and sort there
 
                 match fs::read_dir(dir) {
                    Ok(files) => {
                        let paths: Vec<_> = files
                            .filter_map(Result::ok)
                            .filter(|f| is_valid_file(&f.path()))
                            .map(|f| String::from(strip_current_dir(&f.path()).to_str().unwrap()))
                            .collect();
    
                        print_output(opt.output, List(paths))?;
                    },
                    Err(e) => match e.kind() {
                        ErrorKind::NotFound => eprintln!("the {} directory should exist", dir),
                        _ => eprintln!("Error occurred: {:?}", e)
                    }
                };


                //  let paths: Vec<_> = files
                //      .filter_map(Result::ok)
                //      .filter(|f| is_valid_file(&f.path()))
                //      .map(|f| String::from(strip_current_dir(&f.path()).to_str().unwrap()))
                //      .collect();
 
                //  print_output(opt.output, List(paths))?;
            }
            
            RfcCommand::Generate(generate) => match generate.generate_rfc_command {
                GenerateRfcsCommand::RfcToc(params) => {

                }

                GenerateRfcsCommand::RfcGraph(params) => {

                }
            }
        },

        Command::Adr(adr) => match adr.adr_command {
            AdrCommand::Init(params) => {

                //     Initialises the directory of architecture decision records:
                //     * creates a subdirectory of the current working directory
                //     * creates the first ADR in that subdirectory, recording the decision to
                //       record architectural decisions with ADRs.
                //    If the DIRECTORY is not given, the ADRs are stored in the directory `doc/adr`.


                // TODO: if directory is provided use it otherwise default
                // TODO: see if adr directory aleady exists
                // if it does prompt user
                // TODO: create the first ADR in that subdirectory, recording the decision to record architectural decisions with ADRs.
                // TODO: would like to avoid the .to_string

                let dir = SETTINGS.get_adr_dir();

                // TODO: create_dir_all doesnt appear to throw AlreadyExists.
                // I think this is fine just need to make sure that we dont overwrite initial file
                match fs::read_dir(dir) {
                    Ok(files) => {
                        let paths: Vec<_> = files
                            .filter_map(Result::ok)
                            .filter(|f| is_valid_file(&f.path()))
                            .map(|f| String::from(strip_current_dir(&f.path()).to_str().unwrap()))
                            .collect();
    
                        print_output(opt.output, List(paths))?;
                    },
                    Err(_) => eprintln!("Directory should exist"),
                };

                // TODO: variables for the following
                // adr_bin_dir "/usr/local/Cellar/adr-tools/3.0.0/bin"'
                // adr_template_dir "/usr/local/Cellar/adr-tools/3.0.0"'


                // TODO: call ADRCommand::new to create first ADR with record-architecture-decisions

                fs::copy(Path::new("./init.md"), PathBuf::from(dir).join("file.md"))?;

                // let file_path = path::PathBuf::from(dir).join("file.md");
                // let write_result = fs::write(file_path, "lets us ADRs");
                // match write_result {
                //     Ok(_) => println!("RFC init successful"),
                //     Err(e) => println!("error parsing header: {:?}", e)
                // }

            }

            // TODO: should be able to call new even if you dont init 
            AdrCommand::New(params) => {

                let dir = SETTINGS.get_adr_dir();

                let custom_template = Path::new(dir).join("template.md");
                
                // TODO: fallback to /usr/local/... or whatever the installation dir is
                let template = if custom_template.exists() { custom_template } else { Path::new("./template.md").to_path_buf() };

                
                let maxid;
                if let Some(i) = params.number {
                    // TODO: see if its already taken
                    maxid = i;
                } else {
                    maxid = get_max_id(dir);
                }

                println!("found max id: {}", maxid);

                // TODO: convert title to slug
                // to_lower_case vs to_ascii_lowercase
                let slug = params.title.to_lowercase();

                // TODO: replace following in template - number, title, date, status
                println!("{:?}", template);
                let mut contents = fs::read_to_string(template).expect("Something went wrong reading the file");
                contents = contents.replace("NUMBER", &maxid.to_string());
                contents = contents.replace("TITLE", &params.title);
                contents = contents.replace("DATE", &Utc::now().format("%Y-%m-%d").to_string());
                contents = contents.replace("STATUS", "Accepted");

                // TODO: supersceded
                // TODO: reverse links
                

                let mut file = File::create(slug + ".md")?;
                file.write_all(contents.as_bytes())?;

                // TODO:
                // If the ADR directory contains a template.md file it will be used as the template for the new ADR
                // Otherwise the following file is used:
                // <path to eventfully>/<version>/template.md
                // /usr/local/Cellar/adr-tools/3.0.0/template.md
                // This template follows the style described by Michael Nygard in this article.
                // http://thinkrelevance.com/blog/2011/11/15/documenting-architecture-decisions
            }

            AdrCommand::List(_) => {

                let dir = SETTINGS.get_adr_dir();

                // TODO: do we want to order?
                // read_dir does not guarantee any specific order
                // in order to sort would need to read into a Vec and sort there

                match fs::read_dir(dir) {
                    Ok(files) => {
                        let paths: Vec<_> = files
                            .filter_map(Result::ok)
                            .filter(|f| is_valid_file(&f.path()))
                            .map(|f| String::from(strip_current_dir(&f.path()).to_str().unwrap()))
                            .collect();
    
                        print_output(opt.output, List(paths))?;
                    },
                    Err(e) => match e.kind() {
                        ErrorKind::NotFound => eprintln!("the {} directory should exist", dir),
                        _ => eprintln!("Error occurred: {:?}", e)
                    }
                }
            }

            AdrCommand::Generate(generate) => match generate.generate_adr_command {
                GenerateAdrsCommand::AdrToc(params) => {

                }

                GenerateAdrsCommand::AdrGraph(params) => {

                }
            }
        }

    };

    Ok(())

}