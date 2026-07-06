use crate::client::RedmineClient;
use crate::error::{Error, Result};
use crate::models::common::NamedRef;
use crate::models::issue::Issue;
use crate::models::project::{EnumerationList, Project, ProjectList, StatusList, TrackerList, User, UserList};

pub struct Resolver {
    statuses: Vec<NamedRef>,
    trackers: Vec<NamedRef>,
    priorities: Vec<NamedRef>,
    projects: Vec<Project>,
    users: Vec<User>,
    current_user_id: Option<i64>,
}

impl Resolver {
    pub fn empty() -> Self {
        Self {
            statuses: Vec::new(),
            trackers: Vec::new(),
            priorities: Vec::new(),
            projects: Vec::new(),
            users: Vec::new(),
            current_user_id: None,
        }
    }

    pub fn current_user_id(&mut self, client: &RedmineClient) -> Result<i64> {
        if let Some(id) = self.current_user_id {
            return Ok(id);
        }
        let id = client.current_user()?;
        self.current_user_id = Some(id);
        Ok(id)
    }

    fn ensure_ref_data(&mut self, client: &RedmineClient) -> Result<()> {
        if self.statuses.is_empty() {
            let s: StatusList = client.get("/issue_statuses.json", &[])?;
            self.statuses = s.issue_statuses;
        }
        if self.priorities.is_empty() {
            let p: EnumerationList = client.get("/enumerations.json", &[])?;
            self.priorities = p.issue_priorities;
        }
        Ok(())
    }

    fn ensure_trackers(&mut self, client: &RedmineClient) -> Result<()> {
        if self.trackers.is_empty() {
            let t: TrackerList = client.get("/trackers.json", &[])?;
            self.trackers = t.trackers;
        }
        Ok(())
    }

    fn ensure_projects(&mut self, client: &RedmineClient) -> Result<()> {
        if self.projects.is_empty() {
            let p: ProjectList = client.get("/projects.json", &[("limit", "200")])?;
            self.projects = p.projects;
        }
        Ok(())
    }

    fn ensure_users(&mut self, client: &RedmineClient) -> Result<()> {
        if self.users.is_empty() {
            let u: UserList = client.get("/users.json", &[("limit", "200")])?;
            self.users = u.users;
        }
        Ok(())
    }

    pub fn statuses(&mut self, client: &RedmineClient) -> Result<&[NamedRef]> {
        self.ensure_ref_data(client)?;
        Ok(&self.statuses)
    }

    pub fn priorities(&mut self, client: &RedmineClient) -> Result<&[NamedRef]> {
        self.ensure_ref_data(client)?;
        Ok(&self.priorities)
    }

    #[allow(dead_code)]
    pub fn trackers(&mut self, client: &RedmineClient) -> Result<&[NamedRef]> {
        self.ensure_trackers(client)?;
        Ok(&self.trackers)
    }

    pub fn projects(&mut self, client: &RedmineClient) -> Result<&[Project]> {
        self.ensure_projects(client)?;
        Ok(&self.projects)
    }

    #[allow(dead_code)]
    pub fn users(&mut self, client: &RedmineClient) -> Result<&[User]> {
        self.ensure_users(client)?;
        Ok(&self.users)
    }

    fn match_named(list: &[NamedRef], name: &str, kind: &str) -> Result<i64> {
        match name.parse::<i64>() {
            Ok(id) => Ok(id),
            Err(_) => resolve_by_name_ci(
                &list.iter().map(|n| (n.id, n.name.clone())).collect::<Vec<_>>(),
                name,
                kind,
            ),
        }
    }

    pub fn resolve_status(&mut self, client: &RedmineClient, name: &str) -> Result<i64> {
        let list = self.statuses(client)?.to_vec();
        Self::match_named(&list, name, "status")
    }

    pub fn resolve_priority(&mut self, client: &RedmineClient, name: &str) -> Result<i64> {
        let list = self.priorities(client)?.to_vec();
        Self::match_named(&list, name, "priority")
    }

    pub fn resolve_tracker(&mut self, client: &RedmineClient, name: &str) -> Result<i64> {
        self.ensure_trackers(client)?;
        let list = self.trackers.clone();
        Self::match_named(&list, name, "tracker")
    }

    pub fn resolve_project(
        &mut self,
        client: &RedmineClient,
        name: &str,
        default: Option<&str>,
    ) -> Result<i64> {
        let raw = if name == "-" || name.is_empty() {
            default
                .ok_or_else(|| Error::Usage("no project specified and no default project set".into()))?
                .to_string()
        } else {
            name.to_string()
        };
        if let Ok(id) = raw.parse::<i64>() {
            return Ok(id);
        }
        self.ensure_projects(client)?;
        let lower = raw.to_ascii_lowercase();
        let mut hits: Vec<&Project> = self
            .projects
            .iter()
            .filter(|p| p.identifier.eq_ignore_ascii_case(&lower) || p.name.eq_ignore_ascii_case(&lower))
            .collect();
        match hits.len() {
            0 => Err(Error::NotFound(format!("project '{raw}'"))),
            1 => Ok(hits.remove(0).id),
            _ => Err(Error::Ambiguous {
                kind: "project".into(),
                name: raw,
                matches: hits.iter().map(|p| format!("{} ({})", p.name, p.identifier)).collect(),
            }),
        }
    }

    pub fn resolve_assignee(&mut self, client: &RedmineClient, name: &str) -> Result<i64> {
        if name.eq_ignore_ascii_case("me") {
            return self.current_user_id(client);
        }
        if let Ok(id) = name.parse::<i64>() {
            return Ok(id);
        }
        self.ensure_users(client)?;
        let lower = name.to_ascii_lowercase();
        let candidates: Vec<&User> = self
            .users
            .iter()
            .filter(|u| {
                u.display_name().to_ascii_lowercase().contains(&lower)
                    || u.login.to_ascii_lowercase().contains(&lower)
            })
            .collect();
        match candidates.len() {
            0 => Err(Error::NotFound(format!("user '{name}'"))),
            1 => Ok(candidates[0].id),
            _ => {
                if let Some(exact) = candidates
                    .iter()
                    .find(|u| {
                        u.display_name().to_ascii_lowercase() == lower
                            || u.login.to_ascii_lowercase() == lower
                    })
                {
                    return Ok(exact.id);
                }
                Err(Error::Ambiguous {
                    kind: "user".into(),
                    name: name.to_string(),
                    matches: candidates
                        .iter()
                        .map(|u| format!("{} ({})", u.display_name(), u.id))
                        .collect(),
                })
            }
        }
    }

    #[allow(dead_code)]
    pub fn set_known_from_issue(&mut self, issue: &Issue) {
        if self.current_user_id.is_none() {
            if let Some(a) = &issue.assigned_to {
                let _ = a;
            }
        }
    }
}

fn resolve_by_name_ci(list: &[(i64, String)], name: &str, kind: &str) -> Result<i64> {
    let lower = name.to_ascii_lowercase();
    let mut exact: Vec<(i64, String)> = list
        .iter()
        .filter(|(_, n)| n.to_ascii_lowercase() == lower)
        .cloned()
        .collect();
    match exact.len() {
        1 => Ok(exact.remove(0).0),
        0 => {
            let partial: Vec<(i64, String)> = list
                .iter()
                .filter(|(_, n)| n.to_ascii_lowercase().contains(&lower))
                .cloned()
                .collect();
            match partial.len() {
                1 => Ok(partial[0].0),
                0 => Err(Error::NotFound(format!("{kind} '{name}'"))),
                _ => Err(Error::Ambiguous {
                    kind: kind.to_string(),
                    name: name.to_string(),
                    matches: partial.iter().map(|(_, n)| n.clone()).collect(),
                }),
            }
        }
        _ => Err(Error::Ambiguous {
            kind: kind.to_string(),
            name: name.to_string(),
            matches: exact.iter().map(|(_, n)| n.clone()).collect(),
        }),
    }
}
