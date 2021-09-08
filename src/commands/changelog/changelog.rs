// Idea from git-cliff
// TODO: flush this out more
// TODO: support conventional commits as well as custom
// TODO: custom example would be something like cockroachdb
// TODO: I like cockroachdb's release note and release justification style

use structopt::StructOpt;
use crate::commands::changelog::StripParts;
use std::path::PathBuf;
use crate::commands::changelog::parse_strip_parts;
use crate::constants::DEFAULT_CONFIG_NAME;

#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers ADR management commands")]
pub(crate) struct Changelog {
    #[structopt(subcommand)]
    pub changelog_command: ChangelogCommand,
}

#[derive(StructOpt, Debug)]
pub(crate) enum ChangelogCommand {
    Init(InitChangelog),
}

#[derive(StructOpt, Debug)]
#[structopt(about = "Init Changelog")]
pub(crate) struct InitChangelog {
    #[structopt(
        long,
        short,
        help = "Header text that will be added to the beginning of the changelog."
    )]
    pub header: Option<String>,

    #[structopt(
        long,
        short,
        help = "Body template that represents a single release in the changelog."
    )]
    pub body: Option<String>,

    #[structopt(
        long,
        short,
        help = "Footer text that will be added to the end of the changelog."
    )]
    pub footer: Option<String>,

    #[structopt(
        long,
        short,
        help = "If set to true, leading and trailing whitespaces are removed from the body."
    )]
    pub trim: bool,
}

// rename_all_env = "screaming-snake"
#[derive(StructOpt, Debug)]
#[structopt(about = "Generate Changelog")]
pub(crate) struct GenerateChangeLog {

    /// Sets the configuration file.
    #[structopt(
        long,
        short,
        default_value = DEFAULT_CONFIG_NAME,
        help = "The configuration file to use."
    )]
    pub config: PathBuf,

    /// Sets the working directory.
    #[structopt(short, long, value_name = "PATH")]
    pub workdir: Option<PathBuf>,

    /// Sets the repository to parse commits from.
    #[structopt(short, long, value_name = "PATH")]
    pub repository: Option<PathBuf>,
    // defaults to current directory. env::current_dir()

    // TODO: this can just be a boolean
    #[structopt(long, short, value_name = "PATH", help = "Prepends entries to the given changelog file.")]
    pub prepend: Option<PathBuf>,

    #[structopt(
        long,
        short,
        value_name = "PATH",
        default_value = "CHANGELOG.md"
        help = "Writes output to the given file"
    )]
    pub file: Option<PathBuf>,

    // Sets the tag for the latest version [env: TAG=]
    #[structopt(long, short, help = "The tag to use for the latest version")]
    pub tag: Option<String>,

    #[structopt(
        long,
        short,
        value_name = "TEMPLATE",
        help = "The template for the changelog body. Overrides body set by environment variable and config"
    )]
    pub body: Option<String>,

    // TODO: include possible values
    #[structopt(
        long,
        short,
        parse(try_from_str = parse_strip_parts),
        help = "The configuration file to use."
    )]
    pub strip: Option<StripParts>,

    /// Processes the commits starting from the latest tag.
    #[structopt(short, long)]
    pub latest: bool,

    /// Processes the commits that do not belong to a tag.
    #[structopt(short, long)]
    pub unreleased: bool,

    /// Sets the commit range to process.
    #[structopt(value_name = "RANGE")]
    pub range: Option<String>,
}
