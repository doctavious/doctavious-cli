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
    pub id:      String,
    /// Commit message including title, description and summary.
    pub message: String,
    /// Conventional commit.
    #[serde(skip_deserializing)]
    pub conv:    Option<ConventionalCommit<'a>>,

    // TODO: I like category instead of group
    /// Commit group based on a group parser or its conventional type.
    pub group:   Option<String>,
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
            conv: None,
            group: None,
        }
    }

    /// Processes the commit.
    ///
    /// * converts commit to a conventional commit
    /// * sets the group for the commit
    pub fn process(
        &self,
        parsers: Option<&Vec<CommitParser>>,
        filter_commits: Option<bool>,
        conventional_commits: bool,
    ) -> Result<Self> {
        let mut commit = self.clone();
        if conventional_commits {
            commit = commit.into_conventional()?;
        }
        if let Some(parsers) = parsers {
            commit =
                commit.into_grouped(parsers, filter_commits.unwrap_or(false))?;
        }
        Ok(commit)
    }

    /// Returns the commit with its conventional type set.
    pub fn into_conventional(mut self) -> Result<Self> {
        match ConventionalCommit::parse(Box::leak(
            self.message.to_string().into_boxed_str(),
        )) {
            Ok(conv) => {
                self.conv = Some(conv);
                Ok(self)
            }
            Err(e) => Err(DoctaviousError::ParseError(e)),
        }
    }

    /// Returns the commit with its group set.
    pub fn into_grouped(
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
                        self.group = parser.group.as_ref().cloned();
                        Ok(self)
                    } else {
                        Err(DoctaviousError::GroupError(String::from("Skipping commit")))
                    }
                }
            }
        }
        if !filter {
            Ok(self)
        } else {
            Err(DoctaviousError::GroupError(String::from(
                "Commit does not belong to any group",
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
        match &self.conv {
            Some(conv) => {
                commit.serialize_field("message", conv.description())?;
                commit.serialize_field("body", &conv.body())?;
                commit.serialize_field(
                    "footers",
                    &conv
                        .footers()
                        .to_vec()
                        .iter()
                        .map(|f| f.value())
                        .collect::<Vec<&str>>(),
                )?;
                commit.serialize_field(
                    "group",
                    self.group.as_ref().unwrap_or(&conv.type_().to_string()),
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
                commit.serialize_field("group", &self.group)?;
            }
        }
        commit.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use regex::Regex;
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
            .into_grouped(
                &[CommitParser {
                    message: Regex::new("test*").ok(),
                    body:    None,
                    group:   Some(String::from("test_group")),
                    skip:    None,
                }],
                false,
            )
            .unwrap();
        assert_eq!(Some(String::from("test_group")), commit.group);
    }
}
