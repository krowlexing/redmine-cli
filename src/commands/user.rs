use std::io::Write;

use crate::cli::UserCommand;
use crate::client::RedmineClient;
use crate::config::Config;
use crate::error::Result;
use crate::models::project::UserList;
use crate::output;

pub fn run<W: Write>(client: &RedmineClient, config: &Config, cmd: UserCommand, out: &mut W) -> Result<()> {
    match cmd {
        UserCommand::List { name } => {
            let list: UserList = client.get("/users.json", &[("limit", "200")])?;
            let users = match name {
                Some(sub) => {
                    let lower = sub.to_ascii_lowercase();
                    list.users
                        .into_iter()
                        .filter(|u| {
                            u.display_name().to_ascii_lowercase().contains(&lower)
                                || u.login.to_ascii_lowercase().contains(&lower)
                        })
                        .collect::<Vec<_>>()
                }
                None => list.users,
            };
            output::render_users(config.format, out, &users);
            Ok(())
        }
    }
}
