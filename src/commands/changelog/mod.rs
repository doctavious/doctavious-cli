use regex::Regex;

mod commit;
mod release;
mod changelog;

/// Changelog configuration.
#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ChangelogConfig {
    /// Changelog header.
    pub header: Option<String>,
    /// Changelog body, template.
    pub body:   String,
    /// Changelog footer.
    pub footer: Option<String>,
    /// Trim the template.
    pub trim:   Option<bool>,
}

/// Git configuration
#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct GitConfig {
    /// Whether to enable conventional commits.
    pub conventional_commits: bool,
    /// Git commit parsers.
    pub commit_parsers:       Option<Vec<CommitParser>>,
    /// Whether to filter out commits.
    pub filter_commits:       Option<bool>,
    /// Blob pattern for git tags.
    pub tag_pattern:          Option<String>,
    #[serde(with = "serde_regex", default)]
    /// Regex to skip matched tags.
    pub skip_tags:            Option<Regex>,
}

/// Parser for grouping commits.
#[derive(Debug, Clone, serde_derive::Serialize, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitParser {
    /// Regex for matching the commit message.
    #[serde(with = "serde_regex", default)]
    pub message: Option<Regex>,
    /// Regex for matching the commit body.
    #[serde(with = "serde_regex", default)]
    pub body:    Option<Regex>,
    /// Group of the commit.
    pub group:   Option<String>,
    /// Whether to skip this commit group.
    pub skip:    Option<bool>,
}
