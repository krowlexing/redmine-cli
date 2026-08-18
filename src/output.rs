use std::io::Write;

use crate::config::Format;
use crate::models::common::NamedRef;
use crate::models::issue::{Issue, Journal};
use crate::models::project::{Project, User};

pub fn print_list<W, F>(
    format: Format,
    out: &mut W,
    columns: &[&str],
    rows: Vec<Row>,
    mut field_fn: F,
) where
    W: Write,
    F: FnMut(&Row, &str) -> String,
{
    match format {
        Format::Json => {
            let mut arr = Vec::with_capacity(rows.len());
            for row in &rows {
                let mut obj = serde_json::Map::new();
                for col in columns {
                    obj.insert(
                        (*col).to_string(),
                        serde_json::Value::String(field_fn(row, col)),
                    );
                }
                arr.push(serde_json::Value::Object(obj));
            }
            let _ = writeln!(out, "{}", serde_json::to_string(&arr).unwrap_or_default());
        }
        Format::Tab => {
            let _ = writeln!(out, "{}", columns.join("\t"));
            for row in &rows {
                let cells: Vec<String> = columns.iter().map(|c| field_fn(row, c)).collect();
                let _ = writeln!(out, "{}", cells.join("\t"));
            }
        }
        Format::Pretty => {
            let widths: Vec<usize> = columns
                .iter()
                .enumerate()
                .map(|(i, _)| {
                    rows.iter()
                        .map(|r| cell_width(&field_fn(r, columns[i])))
                        .max()
                        .unwrap_or(0)
                        .max(columns[i].len())
                })
                .collect();
            let pad = |i: usize, s: &str| {
                let pad = widths[i].saturating_sub(cell_width(s));
                format!("{s}{}", " ".repeat(pad))
            };
            let _ = writeln!(
                out,
                "| {} |",
                columns
                    .iter()
                    .enumerate()
                    .map(|(i, c)| pad(i, c))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
            let _ = writeln!(
                out,
                "|{}|",
                widths
                    .iter()
                    .map(|w| "-".repeat(*w + 2))
                    .collect::<Vec<_>>()
                    .join("|")
            );
            for row in &rows {
                let _ = writeln!(
                    out,
                    "| {} |",
                    columns
                        .iter()
                        .enumerate()
                        .map(|(i, c)| pad(i, &field_fn(row, c)))
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
            }
        }
    }
}

fn cell_width(s: &str) -> usize {
    s.chars().count()
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Row {
    pub kind: &'static str,
    pub id: i64,
    pub fields: Vec<(String, String)>,
}

impl Row {
    pub fn new(kind: &'static str, id: i64) -> Self {
        Self {
            kind,
            id,
            fields: Vec::new(),
        }
    }
    pub fn set(mut self, key: &str, val: impl Into<String>) -> Self {
        self.fields.push((key.to_string(), val.into()));
        self
    }
    pub fn get(&self, key: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

fn opt_str<T: std::fmt::Display>(o: &Option<T>) -> String {
    match o {
        Some(v) => v.to_string(),
        None => String::new(),
    }
}

pub fn named_name(n: &Option<NamedRef>) -> String {
    n.as_ref().map(|n| n.name.clone()).unwrap_or_default()
}

pub fn project_name(p: &Option<crate::models::common::ProjectRef>) -> String {
    p.as_ref().map(|p| p.name.clone()).unwrap_or_default()
}

pub fn user_name(u: &Option<crate::models::common::UserRef>) -> String {
    u.as_ref().map(|u| u.name.clone()).unwrap_or_default()
}

#[allow(dead_code)]
pub fn fmt_date(iso: &str) -> String {
    if iso.is_empty() {
        return String::new();
    }
    iso.split('T').next().unwrap_or(iso).to_string()
}

pub fn fmt_dt(iso: &str) -> String {
    if iso.is_empty() {
        return String::new();
    }
    if let Some((d, t)) = iso.split_once('T') {
        let t = t.trim_end_matches('Z').trim_end_matches("UTC");
        if let Some(hms) = t.split('.').next() {
            return format!("{} {}", d, hms);
        }
        return format!("{} {}", d, t);
    }
    iso.to_string()
}

pub fn fmt_ratio(o: &Option<i64>) -> String {
    match o {
        Some(r) => format!("{}%", r),
        None => String::new(),
    }
}

pub fn render_issue_list<W: Write>(format: Format, out: &mut W, issues: &[Issue]) {
    let cols = ["id", "project", "tracker", "status", "priority", "assignee", "subject", "updated"];
    let rows: Vec<Row> = issues
        .iter()
        .map(|i| {
            Row::new("issue", i.id)
                .set("id", i.id.to_string())
                .set("project", project_name(&i.project))
                .set("tracker", named_name(&i.tracker))
                .set("status", named_name(&i.status))
                .set("priority", named_name(&i.priority))
                .set("assignee", user_name(&i.assigned_to))
                .set("subject", i.subject.clone())
                .set("updated", fmt_dt(&i.updated_on))
        })
        .collect();
    print_list(format, out, &cols, rows, |row, col| {
        row.get(col).unwrap_or("").to_string()
    });
}

pub fn render_issue_detail<W: Write>(
    format: Format,
    out: &mut W,
    issue: &Issue,
    include_journals: bool,
) {
    match format {
        Format::Json => {
            let _ = writeln!(out, "{}", serde_json::to_string(issue).unwrap_or_default());
        }
        Format::Tab | Format::Pretty => {
            let kv = vec![
                ("id", issue.id.to_string()),
                ("project", project_name(&issue.project)),
                ("tracker", named_name(&issue.tracker)),
                ("status", named_name(&issue.status)),
                ("priority", named_name(&issue.priority)),
                ("assignee", user_name(&issue.assigned_to)),
                ("author", user_name(&issue.author)),
                ("subject", issue.subject.clone()),
                ("done_ratio", fmt_ratio(&issue.done_ratio)),
                ("start_date", opt_str(&issue.start_date)),
                ("due_date", opt_str(&issue.due_date)),
                ("created_on", fmt_dt(&issue.created_on)),
                ("updated_on", fmt_dt(&issue.updated_on)),
            ];
            for (k, v) in &kv {
                let _ = writeln!(out, "{k}\t{v}");
            }
            if !issue.description.is_empty() {
                let _ = writeln!(out);
                let _ = writeln!(out, "DESCRIPTION");
                let _ = writeln!(out, "{}", issue.description.trim_end());
            }
            if include_journals {
                if let Some(journals) = &issue.journals {
                    if !journals.is_empty() {
                        let _ = writeln!(out);
                        let _ = writeln!(out, "JOURNALS");
                        for j in journals {
                            render_journal_tab(out.by_ref(), j);
                        }
                    }
                }
            }
        }
    }
}

fn render_journal_tab<W: Write>(out: &mut W, j: &Journal) {
    let who = j.user.as_ref().map(|u| u.name.clone()).unwrap_or_default();
    let _ = writeln!(out, "{}\t{}", fmt_dt(&j.created_on), who);
    for d in &j.details {
        let old = detail_value(&d.old_value);
        let new = detail_value(&d.new_value);
        let label = match d.property.as_str() {
            "attr" => format!("  attr {}", d.name),
            "cf" => format!("  cf {}", d.name),
            "attachment" => "  attachment".to_string(),
            "relation" => "  relation".to_string(),
            _ => format!("  {}", d.property),
        };
        if old.is_empty() && new.is_empty() {
            let _ = writeln!(out, "{label}\t(new)");
        } else {
            let _ = writeln!(out, "{label}\t{old} -> {new}");
        }
    }
    if !j.notes.is_empty() {
        let _ = writeln!(out, "  notes\t{}", j.notes.trim_end());
    }
}

fn detail_value(v: &Option<serde_json::Value>) -> String {
    match v {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => String::new(),
    }
}

pub fn render_projects<W: Write>(format: Format, out: &mut W, projects: &[Project]) {
    let cols = ["id", "identifier", "name", "status"];
    let rows: Vec<Row> = projects
        .iter()
        .map(|p| {
            Row::new("project", p.id)
                .set("id", p.id.to_string())
                .set("identifier", p.identifier.clone())
                .set("name", p.name.clone())
                .set("status", p.status.clone().unwrap_or_default())
        })
        .collect();
    print_list(format, out, &cols, rows, |row, col| {
        row.get(col).unwrap_or("").to_string()
    });
}

pub fn render_users<W: Write>(format: Format, out: &mut W, users: &[User]) {
    let cols = ["id", "login", "name", "mail"];
    let rows: Vec<Row> = users
        .iter()
        .map(|u| {
            Row::new("user", u.id)
                .set("id", u.id.to_string())
                .set("login", u.login.clone())
                .set("name", u.display_name())
                .set("mail", u.mail.clone().unwrap_or_default())
        })
        .collect();
    print_list(format, out, &cols, rows, |row, col| {
        row.get(col).unwrap_or("").to_string()
    });
}

pub fn render_named_list<W: Write>(format: Format, out: &mut W, items: &[NamedRef]) {
    let cols = ["id", "name"];
    let rows: Vec<Row> = items
        .iter()
        .map(|n| {
            Row::new("named", n.id)
                .set("id", n.id.to_string())
                .set("name", n.name.clone())
        })
        .collect();
    print_list(format, out, &cols, rows, |row, col| {
        row.get(col).unwrap_or("").to_string()
    });
}

pub fn render_ack<W: Write>(
    format: Format,
    out: &mut W,
    action: &str,
    id: i64,
    changes: &[(String, String)],
) {
    match format {
        Format::Json => {
            let mut obj = serde_json::Map::new();
            obj.insert("ok".into(), serde_json::Value::Bool(true));
            obj.insert("action".into(), serde_json::Value::String(action.into()));
            obj.insert("id".into(), serde_json::Value::Number(id.into()));
            let changes_arr: Vec<serde_json::Value> = changes
                .iter()
                .map(|(k, v)| serde_json::json!({"field": k, "to": v}))
                .collect();
            obj.insert("changes".into(), serde_json::Value::Array(changes_arr));
            let _ = writeln!(out, "{}", serde_json::to_string(&obj).unwrap_or_default());
        }
        Format::Tab | Format::Pretty => {
            let _ = writeln!(out, "{action}\t{id}");
            for (k, v) in changes {
                let _ = writeln!(out, "  {k}\t{v}");
            }
        }
    }
}

fn format_file_size(size: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;
    
    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

pub fn render_attachment_list<W: Write>(format: Format, out: &mut W, attachments: &[crate::models::issue::Attachment]) {
    let cols = ["id", "filename", "size", "content_type", "author", "created_on"];
    let rows: Vec<Row> = attachments
        .iter()
        .map(|a| {
            Row::new("attachment", a.id)
                .set("id", a.id.to_string())
                .set("filename", a.filename.clone())
                .set("size", format_file_size(a.filesize))
                .set("content_type", a.content_type.clone())
                .set("author", a.author.name.clone())
                .set("created_on", a.created_on.clone())
        })
        .collect();
    
    print_list(format, out, &cols, rows, |row, col| {
        row.get(col).unwrap_or("").to_string()
    });
    
    let _ = writeln!(
        out,
        "Use `redmine issue attachments download <attachment-id>` to download attachment into cwd. Check --help for more flags."
    );
}
