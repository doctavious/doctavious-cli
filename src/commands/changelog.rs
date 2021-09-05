// Idea from git-cliff
// TODO: flush this out more

use structopt::StructOpt;

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
