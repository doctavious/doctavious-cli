use lazy_static::lazy_static;
use serde::ser::SerializeSeq;
use serde::{Serialize, Serializer};

use crate::constants::DEFAULT_CONFIG_NAME;
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
mod git;
mod output;

use crate::commands::adr::{new_adr, Adr, AdrCommand, GenerateAdrsCommand, init_adr, reserve_adr};
use crate::commands::rfd::{new_rfd, GenerateRFDsCommand, RFDCommand, RFD, init_rfd, reserve_rfd};
use crate::commands::til::{build_til_readme, Til, TilCommand, new_til, init_til};
use crate::commands::{build_toc, get_leading_character};
use crate::constants::{DEFAULT_ADR_DIR, DEFAULT_RFD_DIR};
use crate::file_structure::FileStructure;
use crate::settings::{
    load_settings, persist_settings, AdrSettings, RFDSettings, TilSettings,
    SETTINGS,
};
use crate::templates::{TemplateExtension, TEMPLATE_EXTENSIONS};
use crate::utils::{format_number, is_valid_file, parse_enum};
use std::error::Error;
use crate::output::Output;

#[derive(StructOpt, Debug)]
#[structopt(name = "Doctavious")]
pub struct Opt {
    #[structopt(
        long,
        help = "Prints a verbose output during the program execution",
        global = true
    )]
    debug: bool,

    #[structopt(
        long,
        short,
        parse(try_from_str = parse_output),
        help = "How a command output should be rendered",
        global = true
    )]
    pub(crate) output: Option<Output>,

    #[structopt(subcommand)]
    cmd: Command,
}

lazy_static! {
    pub static ref DOCTAVIOUS_DIR: PathBuf = {
        let home_dir = dirs::home_dir()
            .expect("Unsupported platform: can't find home directory");
        Path::new(&home_dir).join(DEFAULT_CONFIG_NAME)
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
/// config file overrides output default -- TODO: implement
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
            Err(std::io::Error::new(ErrorKind::InvalidData, "invalid file"))
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

            Err(std::io::Error::new(ErrorKind::InvalidData, "invalid file"))
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
                    String::from(strip_current_dir(&f.path()).to_str().unwrap())
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
        std::env::set_var("RUST_LOG", "doctavious=debug");
        env_logger::init();
    }

    match opt.cmd {
        Command::Adr(adr) => match adr.adr_command {
            AdrCommand::Init(params) => {
                // https://stackoverflow.com/questions/32788915/changing-the-return-type-of-a-function-returning-a-result
                return match init_adr(params.directory, params.structure, params.extension) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err)
                };
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
                        let extension = SETTINGS.get_adr_template_extension(params.format);

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

            AdrCommand::New(params) => {
                init_dir(SETTINGS.get_adr_dir())?;

                let extension = SETTINGS.get_adr_template_extension(params.extension);
                return match new_adr(params.number, params.title, extension) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err)
                };
            }

            AdrCommand::Reserve(params) => {
                let extension = SETTINGS.get_adr_template_extension(params.extension);
                return reserve_adr(params.number, params.title, extension);
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
                return match init_rfd(params.directory, params.structure, params.extension) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err)
                };
            }

            RFDCommand::New(params) => {
                init_dir(SETTINGS.get_rfd_dir())?;

                let extension = SETTINGS.get_rfd_template_extension(params.extension);
                return match new_rfd(params.number, params.title, extension) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(err)
                };
            }

            RFDCommand::List(_) => {
                list(SETTINGS.get_rfd_dir(), opt.output);
            }

            RFDCommand::Generate(generate) => {
                match generate.generate_rfd_command {
                    GenerateRFDsCommand::Toc(params) => {
                        let dir = SETTINGS.get_adr_dir();
                        let extension= SETTINGS.get_adr_template_extension(params.format);

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

            RFDCommand::Reserve(params) => {
                let extension = SETTINGS.get_rfd_template_extension(params.extension);
                return reserve_rfd(params.number, params.title, extension);
            }
        },

        Command::Til(til) => match til.til_command {
            TilCommand::Init(params) => {
                return init_til(params.directory, params.extension);
            }

            TilCommand::New(params) => {
                let dir = SETTINGS.get_til_dir();
                init_dir(&dir)?;

                let extension = SETTINGS.get_til_template_extension(params.extension);

                return new_til(params.title, params.category, params.tags, extension, params.readme, dir);
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
