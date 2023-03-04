// use crate::templates::{get_leading_character, TemplateExtension};
use crate::markup_format::MarkupFormat;
use crate::utils::is_valid_file;
use std::fs;
use std::io::{BufRead, BufReader, ErrorKind};
use walkdir::WalkDir;


pub mod build;
mod bump;
mod cdg;
pub mod changelog;
mod deploy;
pub mod design_decisions;
pub mod githooks;
pub mod init;
pub mod login;
mod presentation;
mod release;
mod service_directory;
mod snippets;
mod software_template;
mod tag;
pub mod telemetry;
pub mod til;
mod link;


// TODO: not a fan of the list ToC for ADRs and RFDs
// TODO: pass in header
pub(crate) fn build_toc(
    dir: &str,
    extension: MarkupFormat,
    intro: Option<String>,
    outro: Option<String>,
    link_prefix: Option<String>,
) {
    let leading_char = extension.leading_header_character();
    let mut content = String::new();
    content.push_str(&format!(
        "{} {}\n\n",
        leading_char, "Architecture Decision Records"
    ));

    if intro.is_some() {
        content.push_str(&intro.unwrap());
        content.push_str("\n\n");
    }

    match fs::metadata(&dir) {
        Ok(_) => {
            let link_prefix = link_prefix.unwrap_or(String::new());
            for entry in WalkDir::new(&dir)
                .sort_by(|a, b| a.file_name().cmp(b.file_name()))
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .filter(|f| is_valid_file(&f.path()))
            {
                let file = match fs::File::open(&entry.path()) {
                    Ok(file) => file,
                    Err(_) => panic!("Unable to read file {:?}", entry.path()),
                };

                let buffer = BufReader::new(file);
                let title = title_string(buffer, extension);

                content.push_str(&format!(
                    "* [{}]({}{})\n",
                    title,
                    link_prefix,
                    entry.path().display()
                ));
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => {
                eprintln!("the {} directory should exist", dir)
            }
            _ => eprintln!("Error occurred: {:?}", e),
        },
    }

    if outro.is_some() {
        content.push_str(&outro.unwrap());
    }

    print!("{}", content);
}

pub(crate) fn title_string<R>(rdr: R, markup_format: MarkupFormat) -> String
where
    R: BufRead,
{
    // TODO: swap this implementation for AST when ready
    let leading_char = markup_format.leading_header_character();
    for line in rdr.lines() {
        let line = line.unwrap();
        if line.starts_with(leading_char) {
            let last_hash = line
                .char_indices()
                .skip_while(|&(_, c)| c == leading_char)
                .next()
                .map_or(0, |(idx, _)| idx);

            // Trim the leading hashes and any whitespace
            return line[last_hash..].trim().to_string();
        }
    }

    // TODO: dont panic. default to filename if cant get title
    panic!("Unable to find title for file");
}
