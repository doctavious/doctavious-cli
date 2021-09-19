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
use git2::Repository;
use std::fs::{
    self,
    File,
};
use std::{io, env};
use crate::git;
use crate::doctavious_error::{
    DoctaviousError,
    Result,
};
use crate::commands::changelog::release::Release;
use crate::commands::changelog::commit::Commit;

use log::warn;


#[derive(StructOpt, Debug)]
#[structopt(about = "Gathers ADR management commands")]
pub(crate) struct Changelog {
    #[structopt(subcommand)]
    pub changelog_command: ChangelogCommand,
}

#[derive(StructOpt, Debug)]
pub(crate) enum ChangelogCommand {
    Init(InitChangelog),
    Generate(GenerateChangeLog)
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
        possible_values = &StripParts::variants(),
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

pub(crate) fn init_changelog() {

}

pub(crate) fn generate_changelog(
    args: GenerateChangeLog,
) -> Result<()>
{
    // TODO: get config/settings default if not present

    // Initialize the git repository.
    let repository =
        Repository::init(args.repository.unwrap_or(env::current_dir()?))?;

    // Parse tags.
    let mut tags = git::tags(&repository, &config.git.tag_pattern)?;

    // Parse commits.
    let mut commit_range = args.range;
    if args.unreleased {
        if let Some(last_tag) = tags.last().map(|(k, _)| k) {
            commit_range = Some(format!("{}..HEAD", last_tag));
        }
    } else if args.latest {
        if tags.len() < 2 {
            return Err(DoctaviousError::ChangelogError(String::from(
                "Latest tag cannot be processed",
            )));
        } else if let (Some(tag1), Some(tag2)) = (
            tags.get_index(tags.len() - 2).map(|(k, _)| k),
            tags.get_index(tags.len() - 1).map(|(k, _)| k),
        ) {
            commit_range = Some(format!("{}..{}", tag1, tag2));
        }
    }

    let commits = git::commits(&repository, commit_range)?;

    // Update tags.
    if let Some(tag) = args.tag {
        if let Some(commit_id) = commits.first().map(|c| c.id().to_string()) {
            match tags.get(&commit_id) {
                Some(tag) => {
                    warn!("There is already a tag ({}) for {}", tag, commit_id)
                }
                None => {
                    tags.insert(commit_id, tag);
                }
            }
        }
    }

    // Process releases.
    let mut releases = vec![Release::default()];
    let mut release_index = 0;
    let mut previous_release = Release::default();
    for git_commit in commits.into_iter().rev() {
        let commit = Commit::from(&git_commit);
        let commit_id = commit.id.to_string();
        releases[release_index].commits.push(commit);
        if let Some(tag) = tags.get(&commit_id) {
            releases[release_index].version = Some(tag.to_string());
            releases[release_index].commit_id = Some(commit_id);
            releases[release_index].timestamp = git_commit.time().seconds();
            previous_release.previous = None;
            releases[release_index].previous = Some(Box::new(previous_release));
            previous_release = releases[release_index].clone();
            releases.push(Release::default());
            release_index += 1;
        }
    }

    // Set the previous release if needed.
    if args.latest {
        if let Some((commit_id, version)) = tags.get_index(tags.len() - 2) {
            let previous_release = Release {
                commit_id: Some(commit_id.to_string()),
                version: Some(version.to_string()),
                ..Release::default()
            };
            releases[0].previous = Some(Box::new(previous_release));
        }
    }

    // Generate changelog.
    let changelog = Changelog::new(releases, &config)?;
    if let Some(path) = args.prepend {
        changelog.prepend(fs::read_to_string(&path)?, &mut File::create(path)?)
    } else if let Some(path) = args.output {
        changelog.generate(&mut File::create(path)?)
    } else {
        changelog.generate(&mut io::stdout())
    }

    return Ok(());
}
