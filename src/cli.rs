use clap::{Parser, Subcommand};

use crate::config::Format;

#[derive(Parser, Debug)]
#[command(
    name = "redmine-cli",
    version,
    about = "Command-line client for the Redmine REST API (agent-friendly output)",
    long_about = None
)]
pub struct Cli {
    #[arg(long, env = "REDMINE_URL", global = true, help = "Redmine base URL")]
    pub url: Option<String>,

    #[arg(long, env = "REDMINE_API_KEY", global = true, help = "Redmine API key")]
    pub key: Option<String>,

    #[arg(
        long,
        env = "REDMINE_FORMAT",
        global = true,
        value_enum,
        help = "Output format: tab (default), pretty, json"
    )]
    pub format: Option<Format>,

    #[arg(long, global = true, help = "Override the default project identifier")]
    pub project: Option<String>,

    #[arg(long, global = true, help = "Print full HTTP details on errors")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Issue operations
    Issue {
        #[command(subcommand)]
        action: IssueCommand,
    },
    /// List projects
    Project {
        #[command(subcommand)]
        action: ProjectCommand,
    },
    /// List users
    User {
        #[command(subcommand)]
        action: UserCommand,
    },
    /// List issue statuses
    Status,
    /// Generate shell completions
    Completion {
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
    /// Manage local configuration
    Config {
        #[command(subcommand)]
        action: ConfigCommand,
    },
}

#[derive(Subcommand, Debug)]
pub enum IssueCommand {
    /// List issues assigned to me
    Mine(IssueListArgs),
    /// List issues with filters
    List(IssueListArgs),
    /// Show a single issue in detail
    Show {
        id: i64,
        #[arg(long, help = "Include journal history")]
        notes: bool,
    },
    /// Update an issue (any subset of fields)
    Update(IssueUpdateArgs),
    /// Create a new issue
    Create(IssueCreateArgs),
    /// Convenience: set status only
    SetStatus { id: i64, status: String },
    /// Convenience: assign only
    Assign { id: i64, assignee: String },
    /// Convenience: close (set status to Closed)
    Close { id: i64 },
    /// Convenience: list issues by status/project
    Of(OfArgs),
}

#[derive(Parser, Debug, Clone)]
pub struct IssueListArgs {
    #[arg(long, help = "Filter assignee: id, 'me', or name substring")]
    pub assigned_to: Option<String>,
    #[arg(long, help = "Filter status: 'open', 'closed', name, or id")]
    pub status: Option<String>,
    #[arg(long, help = "Filter project: identifier, id, or '-' for default")]
    pub project: Option<String>,
    #[arg(long, default_value = "25", help = "Page size")]
    pub limit: u32,
    #[arg(long, default_value = "updated_on:desc", help = "Sort spec e.g. updated_on:desc")]
    pub sort: String,
    #[arg(long, help = "Fetch all pages (overrides --limit)")]
    pub all: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct OfArgs {
    #[arg(long, help = "Filter status: 'open', 'closed', name, or id")]
    pub status: Option<String>,
    #[arg(long, help = "Filter project: identifier, id, or '-' for default")]
    pub project: Option<String>,
    #[arg(long, help = "Filter assignee")]
    pub assigned_to: Option<String>,
    #[arg(long, default_value = "25")]
    pub limit: u32,
    #[arg(long, default_value = "updated_on:desc")]
    pub sort: String,
    #[arg(long)]
    pub all: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct IssueUpdateArgs {
    pub id: i64,
    #[arg(long, help = "New status (name or id)")]
    pub status: Option<String>,
    #[arg(long, help = "New assignee (id, 'me', or name substring)")]
    pub assignee: Option<String>,
    #[arg(long)]
    pub subject: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, help = "New priority (name or id)")]
    pub priority: Option<String>,
    #[arg(long, help = "Done ratio 0-100")]
    pub done: Option<i64>,
    #[arg(long, help = "Append a comment / journal note")]
    pub note: Option<String>,
}

#[derive(Parser, Debug, Clone)]
pub struct IssueCreateArgs {
    #[arg(long, help = "Project: identifier, id, or '-' for default")]
    pub project: Option<String>,
    #[arg(long)]
    pub subject: String,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long, help = "Assignee (id, 'me', or name substring)")]
    pub assignee: Option<String>,
    #[arg(long, help = "Tracker (name or id)")]
    pub tracker: Option<String>,
    #[arg(long, help = "Priority (name or id)")]
    pub priority: Option<String>,
    #[arg(long, help = "Initial status (name or id)")]
    pub status: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommand {
    /// List all projects
    List,
    /// Show details of one project
    Show { id: Identifier },
}

#[derive(Subcommand, Debug)]
pub enum UserCommand {
    /// List all users
    List {
        #[arg(long, help = "Filter by name substring")]
        name: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Set configuration values
    Set {
        #[arg(long)]
        url: Option<String>,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        default_project: Option<String>,
    },
    /// Show effective configuration and source
    Show,
    /// Print the config file path
    Path,
}

#[derive(Clone, Debug)]
pub enum Identifier {
    Id(i64),
    Name(String),
}

impl std::str::FromStr for Identifier {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(id) = s.parse::<i64>() {
            Ok(Identifier::Id(id))
        } else {
            Ok(Identifier::Name(s.to_string()))
        }
    }
}
