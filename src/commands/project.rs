use std::io::Write;

use crate::cli::{Identifier, ProjectCommand};
use crate::client::RedmineClient;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::models::project::ProjectList;
use crate::output;
use crate::resolve::Resolver;

pub fn run<W: Write>(client: &RedmineClient, config: &Config, cmd: ProjectCommand, out: &mut W) -> Result<()> {
    match cmd {
        ProjectCommand::List => {
            let list: ProjectList = client.get("/projects.json", &[("limit", "200")])?;
            output::render_projects(config.format, out, &list.projects);
            Ok(())
        }
        ProjectCommand::Show { id } => {
            let list: ProjectList = client.get("/projects.json", &[("limit", "200")])?;
            let found = match id {
                Identifier::Id(n) => list.projects.into_iter().find(|p| p.id == n),
                Identifier::Name(s) => {
                    let lower = s.to_ascii_lowercase();
                    list.projects
                        .into_iter()
                        .find(|p| p.identifier.eq_ignore_ascii_case(&lower) || p.name.eq_ignore_ascii_case(&lower))
                }
            };
            match found {
                Some(p) => {
                    let mut resolver = Resolver::empty();
                    let _ = resolver.projects(client)?;
                    output::render_projects(config.format, out, std::slice::from_ref(&p));
                    Ok(())
                }
                None => Err(Error::NotFound("project".into())),
            }
        }
    }
}
