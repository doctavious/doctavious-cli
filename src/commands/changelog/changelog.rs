// Idea from git-cliff
// TODO: flush this out more
// TODO: support conventional commits as well as custom
// TODO: custom example would be something like cockroachdb
// TODO: I like cockroachdb's release note and release justification style

use crate::commands::changelog::commit::Commit;
use crate::commands::changelog::parse_strip_parts;
use crate::commands::changelog::release::Release;
use crate::commands::changelog::{ChangelogConfig, StripParts};
use crate::constants::DEFAULT_CONFIG_NAME;
use crate::doctavious_error::{DoctaviousError, Result};
use crate::git;
use crate::settings::{
    load_settings, persist_settings, AdrSettings, ChangelogSettings,
    RFDSettings, TilSettings, SETTINGS,
};
use crate::templates::{TemplateContext, Templates};
use clap::Parser;
use git2::Repository;
use log::warn;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::{env, io};

#[derive(Parser, Debug)]
#[clap(about = "Gathers Changelog management commands")]
pub(crate) struct ChangelogOpt {
    #[clap(subcommand)]
    pub changelog_command: ChangelogCommand,
}

#[derive(Parser, Debug)]
pub(crate) enum ChangelogCommand {
    Init(InitChangelog),
    Generate(GenerateChangeLog),
}

#[derive(Parser, Debug)]
#[clap(about = "Init Changelog")]
pub(crate) struct InitChangelog {
    #[clap(
        long,
        short,
        help = "Header text that will be added to the beginning of the changelog."
    )]
    pub header: Option<String>,

    #[clap(
        long,
        short,
        help = "Body template that represents a single release in the changelog."
    )]
    pub body: Option<String>,

    #[clap(
        long,
        short,
        help = "Footer text that will be added to the end of the changelog."
    )]
    pub footer: Option<String>,

    #[clap(
        long,
        short,
        help = "If set to true, leading and trailing whitespaces are removed from the body."
    )]
    pub trim: bool,
}

// rename_all_env = "screaming-snake"
#[derive(Parser, Debug)]
#[clap(about = "Generate Changelog")]
pub(crate) struct GenerateChangeLog {
    /// Sets the configuration file.
    #[clap(
        long,
        short,
        default_value = DEFAULT_CONFIG_NAME,
        help = "The configuration file to use."
    )]
    pub config: PathBuf,

    /// Sets the working directory.
    #[clap(short, long, value_name = "PATH")]
    pub workdir: Option<PathBuf>,

    // TODO: support multiple
    /// Sets the repository to parse commits from.
    #[clap(short, long, value_name = "PATH")]
    pub repository: Option<PathBuf>,
    // defaults to current directory. env::current_dir()

    // TODO: this can just be a boolean
    #[clap(
        long,
        short,
        value_name = "PATH",
        help = "Prepends entries to the given changelog file."
    )]
    pub prepend: Option<PathBuf>,

    #[clap(
        long,
        short,
        value_name = "PATH",
        help = "Writes output to the given file"
    )]
    pub file: Option<PathBuf>,

    // Sets the tag for the latest version [env: TAG=]
    #[clap(long, short, help = "The tag to use for the latest version")]
    pub tag: Option<String>,

    #[clap(
        long,
        short,
        value_name = "TEMPLATE",
        help = "The template for the changelog body. Overrides body set by environment variable and config"
    )]
    pub body: Option<String>,

    // TODO: include possible values
    #[clap(
        arg_enum,
        long,
        short,
        // possible_values = StripParts::possible_values(),
        parse(try_from_str = parse_strip_parts),
        help = "The configuration file to use."
    )]
    pub strip: Option<StripParts>,

    /// Processes the commits starting from the latest tag.
    #[clap(short, long)]
    pub latest: bool,

    /// Processes the commits that do not belong to a tag.
    #[clap(short, long)]
    pub unreleased: bool,

    /// Sets the commit range to process.
    #[clap(value_name = "RANGE")]
    pub range: Option<String>,
}

/// Changelog generator.
#[derive(Debug)]
pub struct Changelog<'a> {
    releases: Vec<Release<'a>>,
    template: Templates,
    // config:   &'a Config,
    // config: ChangelogConfig
    config: &'a ChangelogSettings,
}

impl<'a> Changelog<'a> {
    /// Constructs a new instance.
    pub fn new(
        releases: Vec<Release<'a>>,
        config: &'a ChangelogSettings,
    ) -> Result<Self> {
        let mut changelog = Self {
            releases,
            template: Templates::new_with_templates({
                let mut template = config.body.to_string();
                if config.trim {
                    template = template
                        .lines()
                        .map(|v| v.trim())
                        .collect::<Vec<&str>>()
                        .join("\n")
                }
                HashMap::from([("release", template)])
            })?,
            config,
        };
        changelog.process_commits();
        changelog.process_releases();
        Ok(changelog)
    }

    /// Processes the commits and omits the ones that doesn't match the
    /// criteria set by configuration file.
    fn process_commits(&mut self) {
        log::debug!("Processing the commits...");
        let config = &self.config.git;
        self.releases.iter_mut().for_each(|release| {
            release.commits = release
                .commits
                .iter()
                .filter_map(|commit| {
                    match commit.process(
                        config.commit_parsers.as_ref(),
                        config.filter_commits,
                        config.conventional_commits,
                    ) {
                        Ok(commit) => Some(commit),
                        Err(e) => {
                            log::trace!(
                                "{} - {:?} ({})",
                                commit.id[..7].to_string(),
                                e,
                                commit
                                    .message
                                    .lines()
                                    .next()
                                    .unwrap_or_default()
                                    .trim()
                            );
                            None
                        }
                    }
                })
                .collect::<Vec<Commit>>();
        });
    }

    /// Processes the releases and filters them out based on the configuration.
    fn process_releases(&mut self) {
        log::debug!("Processing the releases...");
        let skip_regex = self.config.git.skip_tags.as_ref();
        self.releases = self
            .releases
            .clone()
            .into_iter()
            .rev()
            .filter(|release| {
                if release.commits.is_empty() {
                    if let Some(version) = release.version.as_ref().cloned() {
                        log::trace!(
                            "Release doesn't have any commits: {}",
                            version
                        );
                    }
                    false
                } else if let Some(version) = &release.version {
                    !skip_regex
                        .map(|r| {
                            let skip_tag = r.is_match(version);
                            if skip_tag {
                                log::trace!("Skipping release: {}", version)
                            }
                            skip_tag
                        })
                        .unwrap_or_default()
                } else {
                    true
                }
            })
            .collect();
    }

    /// Generates the changelog and writes it to the given output.
    pub fn generate<W: Write>(&self, out: &mut W) -> Result<()> {
        log::debug!("Generating changelog...");
        if let Some(header) = &self.config.header {
            write!(out, "{}", header)?;
        }
        for release in &self.releases {
            let s = self.template.render("release", &TemplateContext::from_serialize(release)?)?;
            write!(out, "{}", s)?;
        }
        if let Some(footer) = &self.config.footer {
            write!(out, "{}", footer)?;
        }
        Ok(())
    }

    /// Generates a changelog and prepends it to the given changelog.
    pub fn prepend<W: Write>(
        &self,
        mut changelog: String,
        out: &mut W,
    ) -> Result<()> {
        log::debug!("Generating changelog and prepending...");
        if let Some(header) = &self.config.header {
            changelog = changelog.replacen(header, "", 1);
        }
        self.generate(out)?;
        write!(out, "{}", changelog)?;
        Ok(())
    }
}

pub(crate) fn init_changelog() {}

pub(crate) fn generate_changelog(args: GenerateChangeLog) -> Result<()> {
    // TODO: get config/settings default if not present

    // TODO: get repository from arg/env
    // Initialize the git repository.
    let repository =
        Repository::init(args.repository.unwrap_or(env::current_dir()?))?;

    let config = load_settings()?.changelog_settings.ok_or(
        DoctaviousError::ChangelogError(String::from(
            "changelog configuration not found",
        )),
    )?;

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
    } else if let Some(path) = args.file {
        changelog.generate(&mut File::create(path)?)
    } else {
        changelog.generate(&mut io::stdout())
    }
}
