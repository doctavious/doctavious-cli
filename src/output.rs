use crate::doctavious_error::{EnumError, Result as DoctaviousResult};
use crate::utils::parse_enum;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::env;

lazy_static! {
    static ref OUTPUT_TYPES: HashMap<&'static str, Output> = {
        let mut map = HashMap::new();
        map.insert("json", Output::Json);
        map.insert("text", Output::Text);
        map
    };
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

pub(crate) fn parse_output(src: &str) -> Result<Output, EnumError> {
    parse_enum(&OUTPUT_TYPES, src)
}

pub(crate) fn print_output<A: std::fmt::Display + Serialize>(
    output: Output,
    value: A,
) -> DoctaviousResult<()> {
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

/// get output based on following order of precednece
/// output argument (--output)
/// env var DOCTAVIOUS_DEFAULT_OUTPUT
/// config file overrides output default -- TODO: implement
/// Output default
pub(crate) fn get_output(opt_output: Option<Output>) -> Output {
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
