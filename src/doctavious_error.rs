use thiserror::Error;

#[derive(Error, Debug)]
pub enum DoctaviousError {

    // /// Error that may occur while I/O operations.
    // #[error("IO error: `{0}`")]
    // IoError(#[from] std::io::Error),
    // /// Error that may occur when attempting to interpret a sequence of u8 as a
    // /// string.
    // #[error("UTF-8 error: `{0}`")]
    // Utf8Error(#[from] std::str::Utf8Error),
    // /// Error variant that represents errors coming out of libgit2.
    // #[error("Git error: `{0}`")]
    // GitError(#[from] git2::Error),
    // /// Error that may occur while parsing the config file.
    // #[error("Cannot parse config: `{0}`")]
    // ConfigError(#[from] config::ConfigError),
    /// When commit's not follow the conventional commit structure we throw this
    /// error.
    #[error("Cannot parse the commit: `{0}`")]
    ParseError(#[from] git_conventional::Error),
    /// Error that may occur while grouping commits.
    #[error("Category error: `{0}`")]
    CategoryError(String),
    // /// Error that may occur while generating changelog.
    // #[error("Changelog error: `{0}`")]
    // ChangelogError(String),
    // /// Error that may occur while template operations such as parse and render.
    // #[error("Template error: `{0}`")]
    // TemplateError(#[from] tera::Error),
    // /// Error that may occur while parsing the command line arguments.
    // #[error("Argument error: `{0}`")]
    // ArgumentError(String),
    // /// Error that may occur while extracting the embedded content.
    // #[error("Embedded error: `{0}`")]
    // EmbeddedError(String),
    // /// Errors that may occur when deserializing types from TOML format.
    // #[error("Cannot parse TOML: `{0}`")]
    // DeserializeError(#[from] toml::de::Error),


}


pub type Result<T> = core::result::Result<T, DoctaviousError>;
