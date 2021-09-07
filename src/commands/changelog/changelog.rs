// Idea from git-cliff
// TODO: flush this out more
// TODO: support conventional commits as well as custom
// TODO: custom example would be something like cockroachdb
// TODO: I like cockroachdb's release note and release justification style

use structopt::StructOpt;
use crate::commands::changelog::StripParts;
use std::path::PathBuf;
use crate::commands::changelog::parse_strip_parts;

// -v, --verbose       Increases the logging verbosity
// -i, --init          Writes the default configuration file to cliff.toml
// -l, --latest        Processes the commits starting from the latest tag
// -u, --unreleased    Processes the commits that do not belong to a tag
// -h, --help          Prints help information
// -V, --version       Prints version information

// -c, --config <PATH>        Sets the configuration file [env: CONFIG=]  [default: cliff.toml]
// -w, --workdir <PATH>       Sets the working directory [env: WORKDIR=]
// -r, --repository <PATH>    Sets the repository to parse commits from [env: REPOSITORY=]
// -p, --prepend <PATH>       Prepends entries to the given changelog file [env: PREPEND=]
// -o, --output <PATH>        Writes output to the given file [env: OUTPUT=]
// -t, --tag <TAG>            Sets the tag for the latest version [env: TAG=]
// -b, --body <TEMPLATE>      Sets the template for the changelog body [env: TEMPLATE=]
// -s, --strip <PART>         Strips the given parts from the changelog [possible values: header, footer, all]

// <RANGE>    Sets the commit range to process

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

impl InitChangelog {
    // pub fn should_persist_settings(&self) -> bool {
    //     return self.directory.is_some() || self.extension.is_some();
    // }
}

// rename_all_env = "screaming-snake"
#[derive(StructOpt, Debug)]
#[structopt(about = "Generate Changelog")]
pub(crate) struct GenerateChangeLog {
    // TODO: default value doctavious.toml
    #[structopt(long, short, help = "The configuration file to use.")]
    pub config: Option<String>,

    // wokdir?

    // repository
    // defaults to current directory. env::current_dir()

    #[structopt(long, short, value_name = "PATH", help = "The tag to use for the latest version")]
    pub prepend: Option<PathBuf>,

    // TODO: will have to change this as it conflicts with the global output
    // TODO: default to CHANGELOG.md?
    #[structopt(long, short, value_name = "PATH", help = "Writes output to the given file")]
    pub output: Option<PathBuf>,

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

    // -c, --config <PATH>        Sets the configuration file [env: CONFIG=]  [default: cliff.toml]
    // -w, --workdir <PATH>       Sets the working directory [env: WORKDIR=]
    // -r, --repository <PATH>    Sets the repository to parse commits from [env: REPOSITORY=]
    // -p, --prepend <PATH>       Prepends entries to the given changelog file [env: PREPEND=]
    // -o, --output <PATH>        Writes output to the given file [env: OUTPUT=]
    // -t, --tag <TAG>            Sets the tag for the latest version [env: TAG=]
    // -b, --body <TEMPLATE>      Sets the template for the changelog body [env: TEMPLATE=]
    // -s, --strip <PART>         Strips the given parts from the changelog [possible values


    // support
    // -u, --unreleased    Processes the commits that do not belong to a tag. last tag to HEAD
    // -l, --latest        Processes the commits starting from the latest tag. get last two tags
    // a range in the format of <commit>..<commit>

    // /// Processes the commits starting from the latest tag.
    // #[structopt(short, long)]
    // pub latest:     bool,
    // /// Processes the commits that do not belong to a tag.
    // #[structopt(short, long)]
    // pub unreleased: bool,
    // #[structopt(value_name = "RANGE")]
    // pub range:      Option<String>,

}
