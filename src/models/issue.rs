use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct Issue {
    issue_type: IssueType,
    code: String,
    message: String,
    help: Option<String>,
    note: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) enum IssueType{ Error, Warning, Suggestion }
