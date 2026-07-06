use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NamedRef {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectRef {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub identifier: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserRef {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Pagination {
    #[serde(default)]
    pub total_count: Option<i64>,
    #[serde(default)]
    pub offset: i64,
    #[serde(default)]
    pub limit: i64,
}
