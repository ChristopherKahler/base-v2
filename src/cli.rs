use clap::{Parser, Subcommand};

use base::config::BaseConfig;
use base::crud;
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
        action: ProjectAction,
    },
    /// Manage tasks
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
    /// Log and search decisions
    Decision {
        #[command(subcommand)]
        action: DecisionAction,
    },
    /// Manage entities (people, organizations)
    Entity {
        #[command(subcommand)]
        action: EntityAction,
    },
    /// Manage goals
    Goal {
        #[command(subcommand)]
        action: GoalAction,
    },
    /// Manage reminders
    Reminder {
        #[command(subcommand)]
        action: ReminderAction,
    },
    /// Sync file-owned data into the graph
    Sync {
        /// Only re-extract files changed since last sync
        #[arg(long)]
        incremental: bool,
    },
    /// Manage domain matching rules
    Domain {
        #[command(subcommand)]
        action: DomainAction,
    },
}

#[derive(Subcommand)]
pub enum ProjectAction {
    /// Add a new project
    Add {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "active")]
        status: String,
        #[arg(long)]
        path: Option<String>,
    },
    /// List all projects
    List,
    /// Show a specific project
    Get { slug: String },
    /// Update a project
    Update {
        slug: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        blocked_by: Option<String>,
        #[arg(long)]
        next_action: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum TaskAction {
    /// Add a task to a project
    Add {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        priority: Option<String>,
    },
    /// List tasks
    List {
        #[arg(long)]
        project: Option<String>,
    },
    /// Mark a task as completed
    Done { slug: String },
}

#[derive(Subcommand)]
pub enum DecisionAction {
    /// Log a new decision
    Log {
        #[arg(long)]
        domain: String,
        #[arg(long)]
        decision: String,
        #[arg(long)]
        rationale: String,
        #[arg(long)]
        recall: Option<String>,
    },
    /// Search decisions by keyword
    Search {
        #[arg(long)]
        keyword: String,
    },
}

#[derive(Subcommand)]
pub enum EntityAction {
    /// Add an entity (person or organization)
    Add {
        #[arg(long)]
        name: String,
        /// Type: person, organization
        #[arg(long, name = "type", default_value = "person")]
        entity_type: String,
    },
    /// List all entities
    List,
    /// Show a specific entity
    Get { slug: String },
    /// Update an entity
    Update {
        slug: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum GoalAction {
    /// Add a goal
    Add {
        #[arg(long)]
        name: String,
        #[arg(long)]
        target: String,
    },
    /// List all goals
    List,
    /// Update a goal
    Update {
        slug: String,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        target: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ReminderAction {
    /// Add a reminder
    Add {
        #[arg(long)]
        name: String,
        /// Due date (YYYY-MM-DD)
        #[arg(long)]
        due: String,
    },
    /// List all reminders
    List,
    /// Remove a reminder (hard delete)
    Remove { slug: String },
}

#[derive(Subcommand)]
pub enum DomainAction {
    /// Add a keyword or path trigger to a domain
    AddTrigger {
        #[arg(long)]
        domain: String,
        #[arg(long)]
        keyword: Option<String>,
        #[arg(long)]
        path: Option<String>,
    },
    /// List all configured domains
    List,
    /// Show a specific domain's full configuration
    Get { name: String },
}

pub fn run() {
    let cli = Cli::parse();
    let cwd = std::env::current_dir().unwrap_or_default();
    let config = BaseConfig::load(&cwd);

    match cli.command {
        Some(Commands::Hook { event }) => hook::dispatch(&event),

        // ─── Project ─────────────────────────────────────
        Some(Commands::Project { action }) => match action {
            ProjectAction::Add { name, status, path } => {
                match crud::project::add(&cwd, &config.namespace, &name, &status, path.as_deref()) {
                    Ok(slug) => println!("Project '{name}' created (slug: {slug})"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
            ProjectAction::List => { let _ = crud::project::list(&cwd, &config.namespace); }
            ProjectAction::Get { slug } => { let _ = crud::project::get(&cwd, &config.namespace, &slug); }
            ProjectAction::Update { slug, status, blocked_by, next_action } => {
                match crud::project::update(&cwd, &config.namespace, &slug, status.as_deref(), blocked_by.as_deref(), next_action.as_deref()) {
                    Ok(()) => println!("Project '{slug}' updated"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
        },

        // ─── Task ────────────────────────────────────────
        Some(Commands::Task { action }) => match action {
            TaskAction::Add { project, name, priority } => {
                match crud::task::add(&cwd, &config.namespace, &project, &name, priority.as_deref()) {
                    Ok(slug) => println!("Task '{name}' created (slug: {slug})"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
            TaskAction::List { project } => { let _ = crud::task::list(&cwd, &config.namespace, project.as_deref()); }
            TaskAction::Done { slug } => {
                match crud::task::done(&cwd, &config.namespace, &slug) {
                    Ok(()) => println!("Task '{slug}' completed"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
        },

        // ─── Decision ────────────────────────────────────
        Some(Commands::Decision { action }) => match action {
            DecisionAction::Log { domain, decision, rationale, recall } => {
                match crud::decision::log(&cwd, &config.namespace, &domain, &decision, &rationale, recall.as_deref()) {
                    Ok(slug) => println!("Decision logged (slug: {slug})"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
            DecisionAction::Search { keyword } => { let _ = crud::decision::search(&cwd, &config.namespace, &keyword); }
        },

        // ─── Entity ──────────────────────────────────────
        Some(Commands::Entity { action }) => match action {
            EntityAction::Add { name, entity_type } => {
                match crud::entity::add(&cwd, &config.namespace, &name, &entity_type) {
                    Ok(slug) => println!("Entity '{name}' created (slug: {slug})"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
            EntityAction::List => { let _ = crud::entity::list(&cwd, &config.namespace); }
            EntityAction::Get { slug } => { let _ = crud::entity::get(&cwd, &config.namespace, &slug); }
            EntityAction::Update { slug, status, description } => {
                match crud::entity::update(&cwd, &config.namespace, &slug, status.as_deref(), description.as_deref()) {
                    Ok(()) => println!("Entity '{slug}' updated"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
        },

        // ─── Goal ────────────────────────────────────────
        Some(Commands::Goal { action }) => match action {
            GoalAction::Add { name, target } => {
                match crud::goal::add(&cwd, &config.namespace, &name, &target) {
                    Ok(slug) => println!("Goal '{name}' created (slug: {slug})"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
            GoalAction::List => { let _ = crud::goal::list(&cwd, &config.namespace); }
            GoalAction::Update { slug, status, target } => {
                match crud::goal::update(&cwd, &config.namespace, &slug, status.as_deref(), target.as_deref()) {
                    Ok(()) => println!("Goal '{slug}' updated"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
        },

        // ─── Reminder ────────────────────────────────────
        Some(Commands::Reminder { action }) => match action {
            ReminderAction::Add { name, due } => {
                match crud::reminder::add(&cwd, &config.namespace, &name, &due) {
                    Ok(slug) => println!("Reminder '{name}' created (slug: {slug})"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
            ReminderAction::List => { let _ = crud::reminder::list(&cwd, &config.namespace); }
            ReminderAction::Remove { slug } => {
                match crud::reminder::remove(&cwd, &config.namespace, &slug) {
                    Ok(()) => println!("Reminder '{slug}' removed"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
        },

        // ─── Sync ────────────────────────────────────────
        Some(Commands::Sync { incremental }) => {
            match base::extract::sync(&cwd, &config, incremental) {
                Ok(report) => {
                    println!(
                        "Sync complete: {} scanned, {} extracted, {} skipped",
                        report.scanned, report.extracted, report.skipped
                    );
                }
                Err(e) => eprintln!("Sync failed: {e}"),
            }
        }

        // ─── Domain ──────────────────────────────────────
        Some(Commands::Domain { action }) => match action {
            DomainAction::AddTrigger { domain: name, keyword, path } => {
                if keyword.is_none() && path.is_none() {
                    eprintln!("Provide --keyword and/or --path");
                    return;
                }
                match domain::add_trigger(&cwd, &name, keyword.as_deref(), path.as_deref()) {
                    Ok(()) => println!("Trigger added to domain '{name}'"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
            DomainAction::List => domain::list_domains(&cwd),
            DomainAction::Get { name } => domain::get_domain(&cwd, &name),
        },

        None => eprintln!("No command provided. Run `base --help` for usage."),
    }
}
