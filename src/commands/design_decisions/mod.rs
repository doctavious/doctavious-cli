use std::fs;
use crate::markup_format::MarkupFormat;
use std::path::{Path, PathBuf};
use crate::DoctaviousResult;

pub mod adr;
pub mod rfd;
mod toc;

// TODO: This is wrong for ADRs init as it doesnt look for a custom init template
// does it need to take in name?
pub(crate) fn get_template(
    dir: &str,
    extension: &MarkupFormat,
    default_template_path: &str,
) -> PathBuf {
    let custom_template =
        Path::new(dir).join("template").with_extension(extension.to_string());

    let template = if custom_template.exists() {
        custom_template
    } else {
        Path::new(default_template_path)
            .with_extension(extension.to_string())
            .to_path_buf()
    };

    return template;
}

pub(crate) fn get_template_content(
    dir: &str,
    extension: &MarkupFormat,
    default_template_path: &str,
) -> String {
    let template_path = get_template(dir, extension, default_template_path);
    // TODO: we shouldnt panic here
    return fs::read_to_string(&template_path).expect(&format!(
        "failed to read file {}.",
        &template_path.to_string_lossy()
    ));
}
