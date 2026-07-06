use serde::{Deserialize, Serialize};

use super::common::NamedRef;

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectList {
    pub projects: Vec<Project>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub identifier: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub is_public: Option<bool>,
    #[serde(default)]
    pub created_on: Option<String>,
    #[serde(default)]
    pub updated_on: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusList {
    pub issue_statuses: Vec<NamedRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerList {
    pub trackers: Vec<NamedRef>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct EnumerationList {
    #[serde(default)]
    pub issue_priorities: Vec<NamedRef>,
    #[serde(default)]
    pub time_entry_activities: Vec<NamedRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CurrentUser {
    pub user: User,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserList {
    pub users: Vec<User>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i64,
    #[serde(default)]
    pub login: String,
    #[serde(default)]
    pub firstname: String,
    #[serde(default)]
    pub lastname: String,
    #[serde(default)]
    pub mail: Option<String>,
}

impl User {
    pub fn display_name(&self) -> String {
        let full = format!("{} {}", self.firstname, self.lastname);
        let full = full.trim();
        if full.is_empty() {
            self.login.clone()
        } else {
            full.to_string()
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
pub struct UserRef {
    pub id: i64,
    pub name: String,
}

impl From<&User> for UserRef {
    fn from(u: &User) -> Self {
        UserRef {
            id: u.id,
            name: u.display_name(),
        }
    }
}
