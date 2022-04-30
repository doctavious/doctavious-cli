use std::path::{Path, PathBuf};
use crate::TemplateExtension;

// TODO: put adrs and rfds in this module. add specific TOC logic in this module
pub mod rfd;
pub mod adr;
mod toc;


pub(crate) fn get_template(
    dir: &str,
    extension: TemplateExtension,
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
