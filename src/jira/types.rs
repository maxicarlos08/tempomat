use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Issue {
    pub id: String,
    pub fields: IssueFields,
}

#[derive(Deserialize, Debug)]
pub struct IssueFields {
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct JiraIssueKey {
    pub board: String,
    pub id: usize,
}

impl<'a> From<(&'a str, &'a str, usize)> for JiraIssueKey {
    fn from(value: (&'a str, &'a str, usize)) -> Self {
        Self {
            board: value.0.to_owned(),
            id: value.2,
        }
    }
}

impl ToString for JiraIssueKey {
    fn to_string(&self) -> String {
        format!("{}-{}", self.board, self.id)
    }
}
