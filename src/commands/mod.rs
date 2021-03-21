use crate::templates::TemplateExtension;
use std::io::{BufRead, BufReader, ErrorKind};
use walkdir::WalkDir;
use std::fs;
use crate::utils::is_valid_file;

pub mod adr;
pub mod init;
pub mod login;
pub mod rfd;
pub mod telemetry;
pub mod til;


pub(crate) fn title_string<R>(rdr: R, extension: TemplateExtension) -> String
    where
        R: BufRead,
{
    // TODO: swap this implementation for AST when ready
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

    panic!("Unable to find title for file");
}

// TODO: pass in header
pub(crate) fn build_toc(
    dir: &str,
    extension: TemplateExtension,
    intro: Option<String>,
    outro: Option<String>,
    link_prefix: Option<String>,
) {
    let leading_char = get_leading_character(extension);
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

pub(crate) fn get_leading_character(extension: TemplateExtension) -> char {
    return match extension {
        TemplateExtension::Markdown => '#',
        TemplateExtension::Asciidoc => '=',
    };
}