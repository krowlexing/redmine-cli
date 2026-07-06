use serde::{Deserialize, Serialize};

use super::common::{NamedRef, ProjectRef, UserRef};

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct IssueList {
    pub issues: Vec<Issue>,
    #[serde(default)]
    pub total_count: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Issue {
    pub id: i64,
    #[serde(default)]
    pub project: Option<ProjectRef>,
    #[serde(default)]
    pub tracker: Option<NamedRef>,
    #[serde(default)]
    pub status: Option<NamedRef>,
    #[serde(default)]
    pub priority: Option<NamedRef>,
    #[serde(default)]
    pub author: Option<UserRef>,
    #[serde(default)]
    pub assigned_to: Option<UserRef>,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub done_ratio: Option<i64>,
    #[serde(default)]
    pub estimated_hours: Option<f64>,
    #[serde(default)]
    pub created_on: String,
    #[serde(default)]
    pub updated_on: String,
    #[serde(default)]
    pub closed_on: Option<String>,
    #[serde(default)]
    pub journals: Option<Vec<Journal>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Journal {
    pub id: i64,
    #[serde(default)]
    pub user: Option<UserRef>,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub created_on: String,
    #[serde(default)]
    pub details: Vec<JournalDetail>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JournalDetail {
    pub property: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub old_value: Option<serde_json::Value>,
    #[serde(default)]
    pub new_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IssueCreate {
    pub project_id: i64,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracker_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct IssueUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_to_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub done_ratio: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueCreateResponse {
    pub issue: Issue,
}
