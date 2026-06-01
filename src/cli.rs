use clap::{Parser, Subcommand};

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
        action: CrudAction,
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

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Hook { event }) => {
            eprintln!("hook:{event} — not yet implemented");
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
        Some(Commands::Domain { action }) => {
            stub("domain", &action);
        }
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
