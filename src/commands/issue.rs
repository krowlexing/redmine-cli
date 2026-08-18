use std::io::Write;

use crate::cli::{AttachmentsArgs, IssueCreateArgs, IssueListArgs, IssueUpdateArgs, OfArgs};
use crate::client::RedmineClient;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::models::issue::{
    Issue, IssueCreate, IssueCreateResponse, IssueList, IssueUpdate,
};
use crate::output;
use crate::resolve::Resolver;

pub fn mine<W: Write>(client: &RedmineClient, config: &Config, mut args: IssueListArgs, out: &mut W) -> Result<()> {
    args.assigned_to = Some(args.assigned_to.unwrap_or_else(|| "me".to_string()));
    list(client, config, args, out)
}

pub fn list<W: Write>(client: &RedmineClient, config: &Config, args: IssueListArgs, out: &mut W) -> Result<()> {
    let mut resolver = Resolver::empty();
    let mut query: Vec<(&str, String)> = Vec::new();

    let limit = if args.all {
        100u32
    } else {
        args.limit.clamp(1, 100)
    };
    query.push(("limit", limit.to_string()));
    query.push(("sort", args.sort.clone()));

    if let Some(s) = &args.status {
        match s.to_ascii_lowercase().as_str() {
            "open" | "closed" => query.push(("status_id", s.to_ascii_lowercase())),
            _ => {
                let id = resolver.resolve_status(client, s)?;
                query.push(("status_id", id.to_string()));
            }
        }
    } else {
        query.push(("status_id", "open".to_string()));
    }

    if let Some(a) = &args.assigned_to {
        let id = if a.eq_ignore_ascii_case("me") {
            resolver.current_user_id(client)?
        } else {
            resolver.resolve_assignee(client, a)?
        };
        query.push(("assigned_to_id", id.to_string()));
    }

    if let Some(p) = &args.project {
        let id = resolver.resolve_project(client, p, config.default_project.as_deref())?;
        query.push(("project_id", id.to_string()));
    }

    let qref: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

    if args.all {
        let mut all: Vec<Issue> = Vec::new();
        let mut offset = 0u32;
        loop {
            let mut q = qref.clone();
            q.push(("offset", &""));
            let off_str = offset.to_string();
            q.last_mut().unwrap().1 = off_str.as_str();
            let page: IssueList = client.get("/issues.json", &q)?;
            let n = page.issues.len() as u32;
            all.extend(page.issues);
            if n < limit || all.len() as i64 >= page.total_count.unwrap_or(0) {
                break;
            }
            offset += limit;
        }
        output::render_issue_list(config.format, out, &all);
    } else {
        let page: IssueList = client.get("/issues.json", &qref)?;
        output::render_issue_list(config.format, out, &page.issues);
    }
    Ok(())
}

pub fn of<W: Write>(client: &RedmineClient, config: &Config, args: OfArgs, out: &mut W) -> Result<()> {
    let merged = IssueListArgs {
        assigned_to: args.assigned_to,
        status: args.status,
        project: args.project,
        limit: args.limit,
        sort: args.sort,
        all: args.all,
    };
    list(client, config, merged, out)
}

pub fn show<W: Write>(client: &RedmineClient, config: &Config, id: i64, include_journals: bool, out: &mut W) -> Result<()> {
    let path = format!("/issues/{id}.json");
    let query: &[(&str, &str)] = if include_journals {
        &[("include", "journals")]
    } else {
        &[]
    };
    let wrapper: IssueWrapper = client.get(&path, query)?;
    output::render_issue_detail(config.format, out, &wrapper.issue, include_journals);
    Ok(())
}

#[derive(serde::Deserialize)]
struct IssueWrapper {
    pub issue: Issue,
}

pub fn update<W: Write>(client: &RedmineClient, config: &Config, args: IssueUpdateArgs, out: &mut W) -> Result<()> {
    let mut resolver = Resolver::empty();
    let mut payload = IssueUpdate::default();
    let mut changes: Vec<(String, String)> = Vec::new();

    if let Some(s) = &args.status {
        let id = resolver.resolve_status(client, s)?;
        payload.status_id = Some(id);
        changes.push(("status".into(), s.clone()));
    }
    if let Some(a) = &args.assignee {
        let id = resolver.resolve_assignee(client, a)?;
        payload.assigned_to_id = Some(id);
        changes.push(("assignee".into(), a.clone()));
    }
    if let Some(p) = &args.priority {
        let id = resolver.resolve_priority(client, p)?;
        payload.priority_id = Some(id);
        changes.push(("priority".into(), p.clone()));
    }
    if let Some(s) = args.subject {
        changes.push(("subject".into(), s.clone()));
        payload.subject = Some(s);
    }
    if let Some(d) = args.description {
        changes.push(("description".into(), "(set)".into()));
        payload.description = Some(d);
    }
    if let Some(d) = args.done {
        if !(0..=100).contains(&d) {
            return Err(Error::Usage("done must be 0..=100".into()));
        }
        payload.done_ratio = Some(d);
        changes.push(("done_ratio".into(), format!("{d}%")));
    }
    if let Some(n) = args.note {
        payload.notes = Some(n.clone());
        changes.push(("note".into(), "(added)".into()));
    }

    if changes.is_empty() {
        return Err(Error::Usage("no update fields supplied".into()));
    }

    client.put(&format!("/issues/{}.json", args.id), &IssueBody {
        issue: payload,
    })?;
    output::render_ack(config.format, out, "updated", args.id, &changes);
    Ok(())
}

#[derive(serde::Serialize)]
struct IssueBody {
    issue: IssueUpdate,
}

pub fn create<W: Write>(client: &RedmineClient, config: &Config, args: IssueCreateArgs, out: &mut W) -> Result<()> {
    let mut resolver = Resolver::empty();
    let project_name = args.project.as_deref().unwrap_or("-");
    let project_id = resolver.resolve_project(client, project_name, config.default_project.as_deref())?;

    let mut payload = IssueCreate {
        project_id,
        subject: args.subject.clone(),
        description: args.description.clone(),
        assigned_to_id: None,
        tracker_id: None,
        priority_id: None,
        status_id: None,
    };
    let mut changes: Vec<(String, String)> = vec![("project".into(), project_name.to_string())];
    if let Some(a) = &args.assignee {
        let id = resolver.resolve_assignee(client, a)?;
        payload.assigned_to_id = Some(id);
        changes.push(("assignee".into(), a.clone()));
    }
    if let Some(t) = &args.tracker {
        let id = resolver.resolve_tracker(client, t)?;
        payload.tracker_id = Some(id);
        changes.push(("tracker".into(), t.clone()));
    }
    if let Some(p) = &args.priority {
        let id = resolver.resolve_priority(client, p)?;
        payload.priority_id = Some(id);
        changes.push(("priority".into(), p.clone()));
    }
    if let Some(s) = &args.status {
        let id = resolver.resolve_status(client, s)?;
        payload.status_id = Some(id);
        changes.push(("status".into(), s.clone()));
    }

    let resp: IssueCreateResponse = client.post("/issues.json", &IssueCreateBody { issue: payload })?;
    output::render_ack(config.format, out, "created", resp.issue.id, &changes);
    Ok(())
}

#[derive(serde::Serialize)]
struct IssueCreateBody {
    issue: IssueCreate,
}

pub fn attachments<W: Write>(client: &RedmineClient, config: &Config, args: AttachmentsArgs, out: &mut W) -> Result<()> {
    match args {
        AttachmentsArgs::List { id } => {
            Err(Error::Usage("attachments list not yet implemented".into()))
        }
        AttachmentsArgs::Download { id, output } => {
            Err(Error::Usage("attachments download not yet implemented".into()))
        }
    }
}
