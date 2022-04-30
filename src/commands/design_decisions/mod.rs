use crate::markup_format::MarkupFormat;
use std::path::{Path, PathBuf};
pub mod adr;
pub mod rfd;
mod toc;

pub(crate) fn get_template(
    dir: &str,
    extension: MarkupFormat,
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
