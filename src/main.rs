use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::{self};
use std::io::{self};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use clap::Parser;
use lazy_static::lazy_static;
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;

use crate::commands::build::{BuildCommand, handle_build_command};
use crate::commands::design_decisions::adr::{ADR, handle_adr_command};
use crate::commands::design_decisions::rfd::{handle_rfd_command, RFD};
use crate::commands::til::{handle_til_command, Til};
use crate::constants::{DEFAULT_ADR_TEMPLATE_PATH, DEFAULT_CONFIG_NAME};
use crate::constants::{DEFAULT_ADR_DIR, DEFAULT_RFD_DIR};
use crate::doctavious_error::Result as DoctaviousResult;
use crate::file_structure::FileStructure;
use crate::markup_format::MARKUP_FORMAT_EXTENSIONS;
use crate::output::{Output, parse_output, print_output};
use crate::utils::{parse_enum};

mod commands;
mod constants;
mod doctavious_error;
mod edit;
mod file_structure;
mod frontmatter;
mod git;
mod keyring;
mod markdown;
mod markup_format;
mod output;
mod scm;
mod settings;
mod templates;
mod utils;
mod files;

#[derive(Parser, Debug)]
#[command(name = "Doctavious")]
pub struct Opt {
    #[arg(
        long,
        help = "Prints a verbose output during the program execution",
        global = true
    )]
    debug: bool,

    #[arg(
        long,
        short,
        value_parser = parse_output,
        help = "How a command output should be rendered",
        global = true
    )]
    pub(crate) output: Option<Output>,

    #[command(subcommand)]
    cmd: Command,
}

lazy_static! {
    pub static ref DOCTAVIOUS_DIR: PathBuf = {
        let home_dir = dirs::home_dir()
            .expect("Unsupported platform: can't find home directory");
        Path::new(&home_dir).join(DEFAULT_CONFIG_NAME)
    };
}

#[derive(Parser, Debug)]
enum Command {
    Adr(ADR),
    Build(BuildCommand),
    Presentation(Presentation),
    RFD(RFD),
    Til(Til),
}

#[derive(Parser, Debug)]
#[command(about = "Presentation commands")]
struct Presentation {
    #[arg(long, help = "Output file path (or directory input-dir is passed)")]
    output_dir: Option<String>,

    #[arg(
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
    #[arg(long, short, help = "Watch input markdowns for changes")]
    watch: bool,

    #[arg(long, short, help = "Enable server mode")]
    server: bool,

    #[arg(long, short, help = "Open preview window")]
    preview: bool,
}

fn init_dir(dir: &str) -> DoctaviousResult<()> {
    // TODO: create_dir_all doesnt appear to throw AlreadyExists. Confirm this
    // I think this is fine just need to make sure that we dont overwrite initial file
    println!("{}", format!("creating dir {}", dir));
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
                for key in MARKUP_FORMAT_EXTENSIONS.keys() {
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
            for key in MARKUP_FORMAT_EXTENSIONS.keys() {
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

fn main() -> DoctaviousResult<()> {
    let opt = Opt::parse();
    if opt.debug {
        env::set_var("RUST_LOG", "debug");
        env_logger::init();
    }

    // TODO: could handle be a trait?
    match opt.cmd {
        Command::Adr(adr) => return handle_adr_command(adr, opt.output),

        Command::Build(cmd) => return handle_build_command(cmd, opt.output),

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

        Command::RFD(rfd) => return handle_rfd_command(rfd, opt.output),

        Command::Til(til) => return handle_til_command(til, opt.output),
    };

    Ok(())
}
