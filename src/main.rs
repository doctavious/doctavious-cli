#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate serde_derive;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use structopt::StructOpt;
// use clap::{Arg, App, SubCommand};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path;

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

// TODO: what outputs to support? plantext/json/...?
#[derive(Debug, Copy, Clone)]
pub enum Output {
    Json,
    Default,
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
    #[structopt(long, short, help = "title of RFC")]
    title: Option<String>,
    
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
    title: Option<String>,
    
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
        map.insert("default", Output::Default);

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
        Output::Default => {
            println!("{:?}", value);
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



fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let matches = App::new("Doctavious")
    //     .version("0.0.1")
    //     .about("Doctavious CLI")
    //     .arg("-c, --config=[FILE] 'Sets a custom config file'")
    //     .arg("<output> 'Sets an optional output file'")
    //     .arg("-d... 'Turn debugging information on'")
    //     .subcommand(
    //         App::new("rfc")
    //             .about("related to RFCs")
    //             .arg(Arg::new("init")
    //                 .short('i')
    //                 .about("
    //                 Initialises the directory of request for comments (RFCs):
    //                  * creates a subdirectory of the current working directory
    //                  * creates the first RFC in that subdirectory, recording the decision to record request for comments.

    //                  If the DIRECTORY is not given, the RFcs are stored in the directory `doc/rfc`.
    //                 ")
    //                 .arg(Arg::new("directory")
    //                     .short('d')
    //                     .about("Directory to store RFCs")
    //                 )
    //             )
    //             .arg(Arg::new("new")
    //                 .short('n')
    //                 .about("Create new RFC")
    //             )
    //         )
    //     )
    //     .get_matches();

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

                fs::copy(path::Path::new("./init.md"), path::PathBuf::from(dir).join("file.md"))?;

                // let file_path = path::PathBuf::from(dir).join("file.md");
                // let write_result = fs::write(file_path, "lets us ADRs");
                // match write_result {
                //     Ok(_) => println!("RFC init successful"),
                //     Err(e) => println!("error parsing header: {:?}", e)
                // }

            }

            AdrCommand::New(params) => {
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

                // TODO: strip_prefix .
                let files = fs::read_dir(dir).expect("Directory should exist");
                files
                    .filter_map(Result::ok)
                    .filter(|f| f.path().extension().unwrap() == "md")
                    .for_each(|f| {
                        println!("{}", f.path().as_path().display());
                    });

                
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
