use glob::PatternError;
use thiserror::Error;

// #[non_exhaustive]
#[derive(Error, Debug)]
pub enum DoctaviousError {
    /// Generic error
    #[error("{0}")]
    Msg(String),

    /// Error that may occur while I/O operations.
    #[error("IO error: `{0}`")]
    IoError(#[from] std::io::Error),
    // /// Error that may occur when attempting to interpret a sequence of u8 as a
    // /// string.
    // #[error("UTF-8 error: `{0}`")]
    // Utf8Error(#[from] std::str::Utf8Error),
    /// Error variant that represents errors coming out of libgit2.
    #[error("Git error: `{0}`")]
    GitError(#[from] git2::Error),
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
    /// Error that may occur while generating changelog.
    #[error("Changelog error: `{0}`")]
    ChangelogError(String),

    /// Error that may occur while parsing the template.
    #[error("Template parse error:\n{0}")]
    TemplateParseError(String),
    /// Error that may occur while template operations such as parse and render.
    #[error("Template error: `{0}`")]
    TemplateError(#[from] tera::Error),

    // /// Error that may occur while parsing the command line arguments.
    // #[error("Argument error: `{0}`")]
    // ArgumentError(String),
    // /// Error that may occur while extracting the embedded content.
    // #[error("Embedded error: `{0}`")]
    // EmbeddedError(String),

    // TODO: fix toml serde error names
    /// Errors that may occur when deserializing types from TOML format.
    #[error("Cannot parse TOML: `{0}`")]
    DeserializeError(#[from] toml::de::Error),

    /// Errors that may occur when serializing types from TOML format.
    #[error("Cannot parse TOML: `{0}`")]
    SerializeError(#[from] toml::ser::Error),

    // /// Error that may occur while converting to enum.
    // #[error("Enum error: `{0}`")]
    // EnumError(String),
    /// Error that may occur while converting to enum.
    #[error("Enum error: `{0}`")]
    EnumParseStringError(#[from] EnumError),

    #[error("Serde json error: `{0}`")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Serde yaml error: `{0}`")]
    SerdeYaml(#[from] serde_yaml::Error),

    #[error("Serde xml error: `{0}`")]
    SerdeXml(#[from] serde_xml_rs::Error),

    #[error("Pattern error: `{0}`")]
    PatternError(#[from] PatternError),

    /// Error that may occur while reserving ADR/RFD number.
    #[error("{0} has already been reserved")]
    ReservedNumberError(i32),

    #[error("walkdir error")]
    WalkdirError(#[from] walkdir::Error),

    // // TODO: figure out what to do here
    // #[error("not sure")]
    // AnyhowError(#[from] anyhow::Error),

    // TODO: fix this
    #[error("{0}")]
    NoConfirmation(String),
}

#[derive(serde::Deserialize, Error, Debug)]
#[error("Enum error: {message}")]
pub struct EnumError {
    pub message: String,
}

// TODO: dont rename result and just use DoctaviousResult
pub type Result<T> = core::result::Result<T, DoctaviousError>;
// pub type Result<T, E = Error> = std::result::Result<T, E>;
