use lazy_static::lazy_static;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use std::collections::HashMap;
use std::env;
use std::fmt::{Debug, Display, Formatter};
use std::fs::{self};
use std::io::ErrorKind;
use std::io::{self};
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use walkdir::WalkDir;

mod commands;
mod constants;
mod edit;
mod file_structure;
mod settings;
mod templates;
mod utils;

use crate::constants::{
    DEFAULT_ADR_DIR, DEFAULT_RFD_DIR,
};
use crate::file_structure::{
    FileStructure,
};
use crate::settings::{
    load_settings, persist_settings, AdrSettings, RFDSettings,
    TilSettings, SETTINGS,
};
use crate::templates::{
    TEMPLATE_EXTENSIONS,
};
use crate::utils::{
    format_number, is_valid_file, parse_enum,
};
use crate::commands::adr::{AdrCommand, GenerateAdrsCommand, Adr, new_adr};
use crate::commands::rfd::{RFD, RFDCommand, new_rfd, GenerateRFDsCommand};
use crate::commands::til::{Til, TilCommand, build_til_readme};
use crate::commands::{build_toc, get_leading_character};

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

// TODO: ADR/RFD ToC vs TiL readme

// TODO: Generate CSV for ADRs and RFDs?

// TODO: better code organization
// maybe we have a markup module with can contain markdown and asciidoc modules
// common things can link in markup module
// such as common structs and traits
// can we move common methods into traits as default impl?
// example would be ToC
// what things would impl the traits?
// Where should TiL live? markup as well? Can override default impl for example til toc/readme

#[derive(StructOpt, Debug)]
#[structopt(name = "Doctavious")]
pub struct Opt {
    #[structopt(
        long,
        help = "Prints a verbose output during the program execution",
        global = true
    )]
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
    fn default() -> Self {
        Output::Json
    }
}

lazy_static! {
    pub static ref DOCTAVIOUS_DIR: PathBuf = {
        let home_dir = dirs::home_dir()
            .expect("Unsupported platform: can't find home directory");
        Path::new(&home_dir).join(".doctavious")
    };
}

#[derive(StructOpt, Debug)]
enum Command {
    RFD(RFD),
    Adr(Adr),
    Til(Til),
    Presentation(Presentation),
}


#[derive(StructOpt, Debug)]
#[structopt(about = "Presentation commands")]
struct Presentation {
    #[structopt(
        long,
        help = "Output file path (or directory input-dir is passed)"
    )]
    output_dir: Option<String>,

    #[structopt(
        long,
        short,
        help = "The base directory to find markdown and theme CSS"
    )]
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
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
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
                Err(_) => Output::default(), // TODO: implement output via settings/config file
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

fn get_content(
    dir: &str,
    number: &str,
    file_structure: FileStructure,
) -> io::Result<String> {
    return match file_structure {
        FileStructure::Flat => {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                for key in TEMPLATE_EXTENSIONS.keys() {
                    let file_name =
                        entry.file_name().to_str().unwrap().to_owned();
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
            Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "invalid file",
            ))
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

            Err(std::io::Error::new(
                ErrorKind::InvalidData,
                "invalid file",
            ))
        }
    };
}

fn list(dir: &str, opt_output: Option<Output>) {
    match fs::metadata(&dir) {
        Ok(_) => {
            let mut f: Vec<_> = WalkDir::new(&dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .filter(|f| is_valid_file(&f.path()))
                .map(|f| {
                    String::from(
                        strip_current_dir(&f.path()).to_str().unwrap(),
                    )
                })
                .collect();

            f.sort();
            print_output(get_output(opt_output), List(f)).unwrap();
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("the {} directory should exist", dir)
            }
            _ => eprintln!("Error occurred: {:?}", e),
        },
    }
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
                        Err(_) => Default::default(),
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

                return new_adr(
                    Some(1),
                    "Record Architecture Decisions".to_string(),
                    SETTINGS.get_adr_template_extension(),
                );
            }

            AdrCommand::New(params) => {
                init_dir(SETTINGS.get_adr_dir())?;

                let extension = match params.extension {
                    Some(v) => v,
                    None => SETTINGS.get_adr_template_extension(),
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
                    SETTINGS.get_adr_structure(),
                );

                let z = Path::new(SETTINGS.get_adr_dir())
                    .join("temp-link")
                    .with_extension("md");
            }

            AdrCommand::Generate(generate) => {
                match generate.generate_adr_command {
                    GenerateAdrsCommand::Toc(params) => {
                        let dir = SETTINGS.get_adr_dir();
                        let extension = match params.format {
                            Some(v) => v,
                            None => SETTINGS.get_adr_template_extension(),
                        };

                        build_toc(
                            dir,
                            extension,
                            params.intro,
                            params.outro,
                            params.link_prefix,
                        );
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
        }

        Command::RFD(rfd) => match rfd.rfd_command {
            RFDCommand::Init(params) => {
                // TODO: need to handle initing multiple times better

                if params.should_persist_settings() {
                    let mut settings = match load_settings() {
                        Ok(settings) => settings,
                        Err(_) => Default::default(),
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

                // TODO: fix
                return new_rfd(
                    Some(1),
                    "Use RFDs ...".to_string(),
                    SETTINGS.get_rfd_template_extension(),
                );
            }

            RFDCommand::New(params) => {
                init_dir(SETTINGS.get_rfd_dir())?;

                let extension = match params.extension {
                    Some(v) => v,
                    None => SETTINGS.get_rfd_template_extension(),
                };

                return new_rfd(params.number, params.title, extension);
            }

            RFDCommand::List(_) => {
                list(SETTINGS.get_rfd_dir(), opt.output);
            }

            RFDCommand::Generate(generate) => {
                match generate.generate_rfd_command {
                    GenerateRFDsCommand::Toc(params) => {
                        let dir = SETTINGS.get_adr_dir();
                        let extension = match params.format {
                            Some(v) => v,
                            None => SETTINGS.get_adr_template_extension(),
                        };

                        build_toc(
                            dir,
                            extension,
                            params.intro,
                            params.outro,
                            params.link_prefix,
                        );
                    }

                    GenerateRFDsCommand::Graph(params) => {}
                }
            }
        },

        Command::Til(til) => match til.til_command {
            TilCommand::Init(params) => {
                let dir = match params.directory {
                    None => SETTINGS.get_til_dir(),
                    Some(ref d) => {
                        let mut settings = match load_settings() {
                            Ok(settings) => settings,
                            Err(_) => Default::default(),
                        };

                        let til_settings = TilSettings {
                            dir: Some(d.to_string()),
                            template_extension: None,
                        };
                        settings.til_settings = Some(til_settings);
                        persist_settings(settings)?;
                        d
                    }
                };

                init_dir(dir)?;
            }

            TilCommand::New(params) => {
                let dir = SETTINGS.get_til_dir();
                init_dir(&dir)?;

                let extension = match params.extension {
                    Some(v) => v,
                    None => SETTINGS.get_til_template_extension(),
                };

                let file_name = params.title.to_lowercase();
                let path = Path::new(dir)
                    .join(params.category)
                    .join(file_name)
                    .with_extension(extension.to_string());

                if path.exists() {
                    eprintln!(
                        "File {} already exists",
                        path.to_string_lossy()
                    );
                } else {
                    let leading_char = get_leading_character(extension);

                    let mut starting_content =
                        format!("{} {}\n", leading_char, params.title);
                    if params.tags.is_some() {
                        starting_content.push_str("\ntags: ");
                        starting_content
                            .push_str(params.tags.unwrap().join(" ").as_str());
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
        },
    };

    Ok(())
}
