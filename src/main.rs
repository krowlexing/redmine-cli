use clap::Parser;

use redmine_cli::cli::{Cli, Command, IssueCommand};
use redmine_cli::client::RedmineClient;
use redmine_cli::commands;
use redmine_cli::config::Config;
use redmine_cli::error::{set_verbose, Error, Result};
use redmine_cli::output;
use redmine_cli::resolve;

fn main() {
    let args = Cli::parse();
    set_verbose(args.verbose);
    match run(args) {
        Ok(()) => {}
        Err(e) => {
            print_error(&e);
            std::process::exit(exit_code(&e));
        }
    }
}

fn run(args: Cli) -> Result<()> {
    let Cli { url, key, project, format, verbose, command } = args;
    match command {
        Command::Config { action } => commands::config_cmd::run(action),
        Command::Completion { shell } => {
            commands::completion::generate(shell);
            Ok(())
        }
        Command::Status => {
            let config = Config::resolve(url.as_deref(), key.as_deref(), project.as_deref(), format)?;
            let client = RedmineClient::with_verbose(&config, verbose)?;
            let mut resolver = resolve::Resolver::empty();
            let list = resolver.statuses(&client)?;
            output::render_named_list(config.format, &mut std::io::stdout(), list);
            Ok(())
        }
        Command::Issue { action } => {
            let config = Config::resolve(url.as_deref(), key.as_deref(), project.as_deref(), format)?;
            let client = RedmineClient::with_verbose(&config, verbose)?;
            run_issue(&client, &config, action)
        }
        Command::Project { action } => {
            let config = Config::resolve(url.as_deref(), key.as_deref(), project.as_deref(), format)?;
            let client = RedmineClient::with_verbose(&config, verbose)?;
            commands::project::run(&client, &config, action, &mut std::io::stdout())
        }
        Command::User { action } => {
            let config = Config::resolve(url.as_deref(), key.as_deref(), project.as_deref(), format)?;
            let client = RedmineClient::with_verbose(&config, verbose)?;
            commands::user::run(&client, &config, action, &mut std::io::stdout())
        }
    }
}

fn run_issue(client: &RedmineClient, config: &Config, action: IssueCommand) -> Result<()> {
    let mut out = std::io::stdout();
    match action {
        IssueCommand::Mine(args) => commands::issue::mine(client, config, args, &mut out),
        IssueCommand::List(args) => commands::issue::list(client, config, args, &mut out),
        IssueCommand::Of(args) => commands::issue::of(client, config, args, &mut out),
        IssueCommand::Show { id, notes } => commands::issue::show(client, config, id, notes, &mut out),
        IssueCommand::Update(args) => {
            config.ensure_mutable()?;
            commands::issue::update(client, config, args, &mut out)
        }
        IssueCommand::Create(args) => {
            config.ensure_mutable()?;
            commands::issue::create(client, config, args, &mut out)
        }
        IssueCommand::SetStatus { id, status } => {
            config.ensure_mutable()?;
            commands::issue::update(client, config, issue_args_for_status(id, status), &mut out)
        }
        IssueCommand::Assign { id, assignee } => {
            config.ensure_mutable()?;
            commands::issue::update(client, config, issue_args_for_assignee(id, assignee), &mut out)
        }
        IssueCommand::Close { id } => {
            config.ensure_mutable()?;
            commands::issue::update(client, config, issue_args_for_status(id, "Closed".into()), &mut out)
        }
        IssueCommand::Attachments { action } => {
            commands::issue::attachments(client, config, action, &mut out)
        }
    }
}

fn issue_args_for_status(id: i64, status: String) -> redmine_cli::cli::IssueUpdateArgs {
    redmine_cli::cli::IssueUpdateArgs {
        id,
        status: Some(status),
        assignee: None,
        subject: None,
        description: None,
        priority: None,
        done: None,
        note: None,
    }
}

fn issue_args_for_assignee(id: i64, assignee: String) -> redmine_cli::cli::IssueUpdateArgs {
    redmine_cli::cli::IssueUpdateArgs {
        id,
        status: None,
        assignee: Some(assignee),
        subject: None,
        description: None,
        priority: None,
        done: None,
        note: None,
    }
}

fn print_error(e: &Error) {
    eprintln!("error: {e}");
}

fn exit_code(e: &Error) -> i32 {
    match e {
        Error::Usage(_) => 2,
        Error::NotFound(_) => 3,
        Error::Ambiguous { .. } => 4,
        Error::Config(_) => 5,
        Error::Blocked => 6,
        Error::Http { status, .. } => match *status {
            401 | 403 => 11,
            404 => 12,
            422 => 13,
            s if (500..600).contains(&s) => 14,
            _ => 10,
        },
        Error::Network(_) => 15,
        Error::Decode(_) => 16,
    }
}
