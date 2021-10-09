use crate::doctavious_error::{
    DoctaviousError,
    Result,
};
use git2::Commit as GitCommit;
use git_conventional::Commit as ConventionalCommit;
use serde::ser::{
    Serialize,
    SerializeStruct,
    Serializer,
};
use crate::commands::changelog::CommitParser;


/// Common commit object that is parsed from a repository.
#[derive(Debug, Clone, PartialEq, serde_derive::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit<'a> {
    /// Commit ID.
    pub id: String,
    /// Commit message including title, description and summary.
    pub message: String,
    /// Conventional commit.
    #[serde(skip_deserializing)]
    pub conventional: Option<ConventionalCommit<'a>>,
    /// Commit category based on a category parser or its conventional type.
    pub category: Option<String>,
}

impl<'a> From<&GitCommit<'a>> for Commit<'a> {
    fn from(commit: &GitCommit<'a>) -> Self {
        Self::new(
            commit.id().to_string(),
            commit.message().unwrap_or_default().to_string(),
        )
    }
}

impl Commit<'_> {
    /// Constructs a new instance.
    pub fn new(id: String, message: String) -> Self {
        Self {
            id,
            message,
            conventional: None,
            category: None,
        }
    }

    /// Processes the commit.
    ///
    /// * converts commit to a conventional commit
    /// * sets the group for the commit
    pub fn process(
        &self,
        parsers: Option<&Vec<CommitParser>>,
        filter_commits: bool,
        conventional_commits: bool,
    ) -> Result<Self> {
        let mut commit = self.clone();
        if conventional_commits {
            commit = commit.into_conventional()?;
        }
        if let Some(parsers) = parsers {
            commit = commit.into_category(parsers, filter_commits)?;
        }
        Ok(commit)
    }

    /// Returns the commit with its conventional type set.
    pub fn into_conventional(mut self) -> Result<Self> {
        match ConventionalCommit::parse(Box::leak(
            self.message.to_string().into_boxed_str(),
        )) {
            Ok(conventional) => {
                self.conventional = Some(conventional);
                Ok(self)
            }
            Err(e) => Err(DoctaviousError::ParseError(e)),
        }
    }

    /// Returns the commit with its category set.
    pub fn into_category(
        mut self,
        parsers: &[CommitParser],
        filter: bool,
    ) -> Result<Self> {
        for parser in parsers {
            for regex in vec![parser.message.as_ref(), parser.body.as_ref()]
                .into_iter()
                .flatten()
            {
                if regex.is_match(&self.message) {
                    return if parser.skip != Some(true) {
                        self.category = parser.category.as_ref().cloned();
                        Ok(self)
                    } else {
                        Err(DoctaviousError::CategoryError(String::from("Skipping commit")))
                    }
                }
            }
        }
        if !filter {
            Ok(self)
        } else {
            Err(DoctaviousError::CategoryError(String::from(
                "Commit does not belong to any category",
            )))
        }
    }
}

impl Serialize for Commit<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut commit = serializer.serialize_struct("Commit", 8)?;
        commit.serialize_field("id", &self.id)?;
        match &self.conventional {
            Some(conv) => {
                commit.serialize_field("message", conv.description())?;
                commit.serialize_field("body", &conv.body())?;
                commit.serialize_field(
                    "footers",
                    &conv
                        .footers()
                        .to_vec()// TODO: is this necessary?
                        .iter()
                        .map(|f| f.value())
                        .collect::<Vec<&str>>(),
                )?;
                commit.serialize_field(
                    "category",
                    self.category.as_ref().unwrap_or(&conv.type_().to_string()),
                )?;
                commit.serialize_field(
                    "breaking_description",
                    &conv.breaking_description(),
                )?;
                commit.serialize_field("breaking", &conv.breaking())?;
                commit.serialize_field("scope", &conv.scope())?;
            }
            None => {
                commit.serialize_field("message", &self.message)?;
                commit.serialize_field("category", &self.category)?;
            }
        }
        commit.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use regex::{Regex, RegexBuilder};
    #[test]
    fn conventional_commit() {
        let test_cases = vec![
            (
                Commit::new(
                    String::from("123123"),
                    String::from("test(commit): add test"),
                ),
                true,
            ),
            (
                Commit::new(String::from("124124"), String::from("xyz")),
                false,
            ),
        ];
        for (commit, is_conventional) in &test_cases {
            assert_eq!(is_conventional, &commit.clone().into_conventional().is_ok())
        }
        let commit = test_cases[0]
            .0
            .clone()
            .into_category(
                &[CommitParser {
                    message: Regex::new("test*").ok(),
                    body: None,
                    category: Some(String::from("test_category")),
                    skip: None,
                }],
                false,
            )
            .unwrap();
        assert_eq!(Some(String::from("test_category")), commit.category);
    }

    // # The following release note formats have been seen in the wild:
    // #
    // # Release note (xxx): yyy    <- canonical
    // # Release Notes: None
    // # Release note (xxx): yyy
    // # Release note (xxx) : yyy
    // # Release note: (xxx): yyy
    // # Release note: xxx: yyy
    // # Release note: (xxx) yyy
    // # Release note: yyy (no category)
    // # Release note (xxx, zzz): yyy
    // norelnote = re.compile(r'^[rR]elease [nN]otes?: *[Nn]one', flags=re.M)
    // # Captures :? (xxx) ?: yyy
    // form1 = r':? *\((?P<cat1>[^)]*)\) *:?'
    // # Captures : xxx: yyy - this must be careful not to capture too much, we just accept one or two words
    // form2 = r': *(?P<cat2>[^ ]+(?: +[^ ]+)?) *:'
    // # Captures : yyy - no category
    // form3 = r':(?P<cat3>)'
    // relnote = re.compile(r'(?:^|[\n\r])[rR]elease [nN]otes? *(?:' + form1 + '|' + form2 + '|' + form3 + r') *(?P<note>.*)$', flags=re.S)
    //
    // coauthor = re.compile(r'^Co-authored-by: (?P<name>[^<]*) <(?P<email>.*)>', flags=re.M)
    // fixannot = re.compile(r'^([fF]ix(es|ed)?|[cC]lose(d|s)?) #', flags=re.M)
    #[test]
    fn commit_release_note() {
        // Release justification: testing only
        // Release justification: Bug fixes and low-risk updates to new
        // functionality
        //
        // Release note: None
        // Release note (enterprise change): cloud storage sinks are no longer experimental

        // no release note
        // RegexBuilder::new("^[rR]elease [nN]otes?: *[Nn]one").multi_line(true).build()
        // Captures :? (xxx) ?: yyy
        // form1 = Regex::new(":? *\((?P<cat1>[^)]*)\) *:?").ok()
        // Captures : xxx: yyy - this must be careful not to capture too much, we just accept one or two words
        // form2 = r': *(?P<cat2>[^ ]+(?: +[^ ]+)?) *:'
        // form2 = Regex::new(": *(?P<cat2>[^ ]+(?: +[^ ]+)?) *:").ok()
        // Captures : yyy - no category
        // form3 = r':(?P<cat3>)'
        // formt3 = Regex::new(":(?P<cat3>)").ok()
        // relnote = re.compile(r'(?:^|[\n\r])[rR]elease [nN]otes? *(?:' + form1 + '|' + form2 + '|' + form3 + r') *(?P<note>.*)$', flags=re.S)
        // release_note =
        // RegexBuilder::new("(?:^|[\n\r])[rR]elease [nN]otes? *(?:' + form1 + '|' + form2 + '|' + form3 + r").dot_matches_new_line(true).build()
        // commit_parsers:       Some(vec![
        //     CommitParser {
        //         message: Regex::new("^[rR]elease [nN]otes?: *[Nn]one").ok(),
        //         body:    None,
        //         category:   Some(String::from("New features")),
        //         skip:    None,
        //     },
        //     CommitParser {
        //         message: Regex::new("fix*").ok(),
        //         body:    None,
        //         category:   Some(String::from("Bug Fixes")),
        //         skip:    None,
        //     },
        //     CommitParser {
        //         message: Regex::new(".*").ok(),
        //         body:    None,
        //         category:   Some(String::from("Other")),
        //         skip:    None,
        //     },
        // ])
    }
}
