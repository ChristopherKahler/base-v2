use clap::{Parser, Subcommand};

use base::domain;
use base::hook;

#[derive(Parser)]
#[command(name = "base", version, about = "Proactive context-injection engine for Claude Code")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Handle Claude Code hook events (session-start, post-tool-use, user-prompt-submit)
    Hook {
        /// Hook event type
        event: String,
    },
    /// Manage projects
    Project {
        #[command(subcommand)]
        action: CrudAction,
    },
    /// Manage tasks
    Task {
        #[command(subcommand)]
        action: CrudAction,
    },
    /// Log and search decisions
    Decision {
        #[command(subcommand)]
        action: CrudAction,
    },
    /// Manage entities (people, organizations)
    Entity {
        #[command(subcommand)]
        action: CrudAction,
    },
    /// Sync file-owned data into the graph
    Sync,
    /// Manage domain matching rules
    Domain {
        #[command(subcommand)]
        action: DomainAction,
    },
}

#[derive(Subcommand)]
pub enum CrudAction {
    /// Add a new item
    Add,
    /// List items
    List,
    /// Get a specific item
    Get {
        /// Item identifier
        id: String,
    },
}

#[derive(Subcommand)]
pub enum DomainAction {
    /// Add a keyword or path trigger to a domain
    AddTrigger {
        /// Domain name
        #[arg(long)]
        domain: String,
        /// Keyword trigger to add
        #[arg(long)]
        keyword: Option<String>,
        /// Path trigger to add
        #[arg(long)]
        path: Option<String>,
    },
    /// List all configured domains
    List,
    /// Show a specific domain's full configuration
    Get {
        /// Domain name
        name: String,
    },
}

pub fn run() {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap_or_default();

    match cli.command {
        Some(Commands::Hook { event }) => {
            hook::dispatch(&event);
        }
        Some(Commands::Project { action }) => {
            stub("project", &action);
        }
        Some(Commands::Task { action }) => {
            stub("task", &action);
        }
        Some(Commands::Decision { action }) => {
            stub("decision", &action);
        }
        Some(Commands::Entity { action }) => {
            stub("entity", &action);
        }
        Some(Commands::Sync) => {
            eprintln!("sync — not yet implemented");
        }
        Some(Commands::Domain { action }) => match action {
            DomainAction::AddTrigger {
                domain: name,
                keyword,
                path,
            } => {
                if keyword.is_none() && path.is_none() {
                    eprintln!("Provide --keyword and/or --path");
                    return;
                }
                match domain::add_trigger(&cwd, &name, keyword.as_deref(), path.as_deref()) {
                    Ok(()) => println!("Trigger added to domain '{name}'"),
                    Err(e) => eprintln!("Failed to add trigger: {e}"),
                }
            }
            DomainAction::List => {
                domain::list_domains(&cwd);
            }
            DomainAction::Get { name } => {
                domain::get_domain(&cwd, &name);
            }
        },
        None => {
            eprintln!("No command provided. Run `base --help` for usage.");
        }
    }
}

fn stub(noun: &str, action: &CrudAction) {
    let verb = match action {
        CrudAction::Add => "add",
        CrudAction::List => "list",
        CrudAction::Get { id } => {
            eprintln!("{noun}:get {id} — not yet implemented");
            return;
        }
    };
    eprintln!("{noun}:{verb} — not yet implemented");
}
