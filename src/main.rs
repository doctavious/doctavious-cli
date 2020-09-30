#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;

use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use structopt::StructOpt;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

use std::io;
use std::path::{Path, PathBuf};
use std::ffi::OsStr;

use chrono::prelude::*;

// TODO: for ADR and RFC make sure template can be either markdown or asciidoc
// TODO: Automatically update readme TOC
// TODO: configuration
// TODO: output options
// config file
// env var DOCTAVIOUS_DEFAULT_OUTPUT - overrides config
// --output  - overrides env var and config
// TODO: RFC / ADR meta frontmatter

#[derive(StructOpt, Debug)]
#[structopt(
    name = "eventfully"
)]
pub struct Opt {
    #[structopt(long, help = "Prints a verbose output during the program execution")]
    debug: bool,

    #[structopt(long, short, default_value = "default", parse(try_from_str = parse_output), help = "How a command output should be rendered")]
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


fn print_output<A: std::fmt::Debug + Serialize>(
    output: Output,
    value: A,
) -> Result<(), Box<dyn std::error::Error>> {
    match output {
        Output::Json => {
            serde_json::to_writer_pretty(std::io::stdout(), &value)?;
            Ok(())
        }
        Output::Text => {
            println!("{:?}", value);
            Ok(())
        }
        Output::Table => {
            Ok(())
        }
    }
}


// TODO: review this code
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

// impl<A> std::fmt::Display for List<A>
// where
//     A: std::fmt::Display,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         for value in self.0.iter() {
//             writeln!(f, "{}", value)?;
//         }

//         Ok(())
//     }
// }

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

            }

            RfcCommand::List(params) => {
                
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
                
                let dir = match params.directory {
                    None => "./doc/adr",
                    Some(ref x) => x,
                };

                println!("RFC directory {}", dir);

                // TODO: create_dir_all doesnt appear to throw AlreadyExists.
                // I think this is fine just need to make sure that we dont overwrite initial file
                let create_dir_result = fs::create_dir_all(dir);
                match create_dir_result {
                    Ok(_) => println!("Created {}", dir),
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                        println!("{} already exists", dir);
                        // TODO: return
                    }
                    Err(e) => {
                        println!("Other error: {}", e);
                        // TODO: return
                    }
                    
                }

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

                // TODO: get directory from default or from configuration / .adr-dir
                let dir = "./doc/adr";

                let custom_template = Path::new(dir).join("template.md");
                
                // TODO: fallback to /usr/local/... or whatever the installation dir is
                let template = if custom_template.exists() { custom_template } else { Path::new("./template.md").to_path_buf() };

                let maxid;
                if let Some(i) = params.number {
                    maxid = i;
                } else {
                    let files = fs::read_dir(dir).expect("Directory should exist");
                    maxid = files
                    .filter_map(Result::ok)
                    .filter(|f| f.path().extension().unwrap() == "md")
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

                // TODO: get directory from default or from configuration / .adr-dir
                let dir = "./doc/adr";

                // read_dir does not guarantee any specific order
                // in order to sort would need to read into a Vec and sort there


                // let mut paths: Vec<_> = fs::read_dir("/").unwrap()
                // .map(|r| r.unwrap())
                // .collect();

                // TODO: strip_prefix .
                let files = fs::read_dir(dir).expect("Directory should exist");
                let paths = files
                    .filter_map(Result::ok)
                    .filter(|f| f.path().extension().unwrap() == "md")
                    .map(|f| f.path())
                    .collect();


                print_output(opt.output, List(paths))?;
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
