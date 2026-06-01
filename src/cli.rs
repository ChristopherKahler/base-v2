use clap::{Parser, Subcommand};

use base::config::BaseConfig;
use base::crud;
use base::domain;
use base::hook;

#[derive(Parser)]
#[command(
    name = "base",
    version,
    about = "BASE — Proactive context-injection engine for Claude Code",
    after_help = "Built by Chris Kahler · Chris AI Systems\n\
                  Community & support: https://chrisai.cv/skool\n\
                  Tutorials: https://www.youtube.com/@chris-ai-systems"
)]
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
        /// Run AST codebase extraction (tree-sitter, 35+ languages)
        #[arg(long)]
        ast: bool,
        /// Target directory for AST extraction (defaults to cwd)
        #[arg(long)]
        target: Option<String>,
    },
    /// Manage domain matching rules
    Domain {
        #[command(subcommand)]
        action: DomainAction,
    },
    /// Graph-backed structured memory
    Learn {
        /// The memory text to store
        #[arg(long)]
        text: String,
        /// Note type: insight, correction, decision, commitment, shift
        #[arg(long, default_value = "insight")]
        r#type: String,
        /// Link to a domain (REQUIRED — notes without domain edges are orphans)
        #[arg(long)]
        domain: String,
        /// Link to a project (optional additional edge)
        #[arg(long)]
        project: Option<String>,
        /// Link to an entity (optional additional edge)
        #[arg(long)]
        entity: Option<String>,
    },
    /// Search notes by keyword and/or domain
    Recall {
        /// Search text in note content
        #[arg(long)]
        keyword: Option<String>,
        /// Filter by linked domain
        #[arg(long)]
        domain: Option<String>,
    },
    /// Manage rules in the graph (add, list, remove)
    Rule {
        #[command(subcommand)]
        action: RuleAction,
    },
    /// Install base globally: build, symlink, create ~/.base-gbl, wire hooks
    Install {
        /// Path to carl.json for decision migration
        #[arg(long)]
        carl: Option<String>,
        /// Skip hook wiring in settings.json
        #[arg(long)]
        skip_hooks: bool,
    },
    /// Scaffold a new workspace: create .base/, write configs, register globally
    Scaffold {
        /// Target directory (defaults to cwd)
        path: Option<String>,
    },
    /// Operator identity profile (init, show)
    Operator {
        #[command(subcommand)]
        action: OperatorAction,
    },
}

#[derive(Subcommand)]
pub enum OperatorAction {
    /// Create operator profile at ~/.base-gbl/operator.toml
    Init {
        #[arg(long)]
        name: String,
    },
    /// Show current operator profile
    Show,
}

#[derive(Subcommand)]
pub enum ProjectAction {
    /// Add a new project
    Add {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = "active")]
        status: String,
        /// Project path (REQUIRED — auto-creates domain trigger for file matching)
        #[arg(long)]
        path: String,
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
    /// Add an entity (person or organization) — must link to at least one domain or project
    Add {
        #[arg(long)]
        name: String,
        /// Type: person, organization
        #[arg(long, name = "type", default_value = "person")]
        entity_type: String,
        /// Domain this entity relates to (REQUIRED — prevents orphan entities)
        #[arg(long)]
        domain: String,
        /// Project this entity relates to (optional additional edge)
        #[arg(long)]
        project: Option<String>,
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
pub enum RuleAction {
    /// Add a rule to a domain in the graph
    Add {
        #[arg(long)]
        domain: String,
        #[arg(long)]
        text: String,
    },
    /// List rules for a domain from the graph
    List {
        #[arg(long)]
        domain: String,
    },
    /// Remove a rule by index from a domain
    Remove {
        #[arg(long)]
        domain: String,
        #[arg(long)]
        index: u32,
    },
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
    /// Sync domains/rules from domains.toml into the graph. Optionally migrate decisions from carl.json.
    Sync {
        /// Path to carl.json for one-time decision migration
        #[arg(long)]
        carl: Option<String>,
    },
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
                match crud::project::add(&cwd, &config.namespace, &name, &status, Some(&path)) {
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
            EntityAction::Add { name, entity_type, domain, project } => {
                match crud::entity::add(&cwd, &config.namespace, &name, &entity_type, &domain, project.as_deref()) {
                    Ok(slug) => println!("Entity '{name}' created (slug: {slug}, domain: {domain})"),
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
        Some(Commands::Sync { incremental, ast, target }) => {
            if ast {
                // AST extraction via bundled Python scripts
                let target_dir = target.as_deref().unwrap_or(".");
                let binary_path = std::env::current_exe().unwrap_or_default();
                let scripts_dir = binary_path
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("scripts").join("ast"))
                    .unwrap_or_else(|| std::path::PathBuf::from("scripts/ast"));

                // Also check relative to cwd for dev builds
                let ast_script = if scripts_dir.join("onto_ast.py").exists() {
                    scripts_dir.join("onto_ast.py")
                } else {
                    // Fallback: look relative to the base-v2 source
                    cwd.join("scripts/ast/onto_ast.py")
                };

                if !ast_script.exists() {
                    eprintln!("AST extractor not found at {}", ast_script.display());
                    eprintln!("Expected: scripts/ast/onto_ast.py bundled with base");
                    return;
                }

                let graph_path = base::config::find_workspace_base(&cwd)
                    .map(|b| b.join("graph.trig"))
                    .unwrap_or_else(|| cwd.join(".base/graph.trig"));

                println!("AST extraction: {} → {}", target_dir, graph_path.display());
                let status = std::process::Command::new("python3")
                    .arg(&ast_script)
                    .arg(target_dir)
                    .arg("--output")
                    .arg(&graph_path)
                    .arg("--format")
                    .arg("trig")
                    .status();

                match status {
                    Ok(s) if s.success() => println!("AST extraction complete"),
                    Ok(s) => eprintln!("AST extraction exited with code {:?}", s.code()),
                    Err(e) => eprintln!("Failed to run AST extractor: {e}"),
                }
            } else {
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
            DomainAction::Sync { carl } => {
                let carl_path = carl.as_ref().map(std::path::Path::new);
                match domain::sync::sync_domains_to_graph(&config, &cwd, carl_path) {
                    Ok(stats) => println!(
                        "Domain sync complete: {} domains, {} rules, {} decisions",
                        stats.domains, stats.rules, stats.decisions
                    ),
                    Err(e) => eprintln!("Domain sync failed: {e}"),
                }
            }
        },

        // ─── Rule ─────────────────────────────────────────
        Some(Commands::Rule { action }) => match action {
            RuleAction::Add { domain: name, text } => {
                match crud::rule::add(&cwd, &config.namespace, &name, &text) {
                    Ok(index) => println!("Rule {index} added to domain '{name}'"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
            RuleAction::List { domain: name } => {
                let _ = crud::rule::list(&cwd, &config.namespace, &name);
            }
            RuleAction::Remove { domain: name, index } => {
                match crud::rule::remove(&cwd, &config.namespace, &name, index) {
                    Ok(()) => println!("Rule {index} removed from domain '{name}'"),
                    Err(e) => eprintln!("Failed: {e}"),
                }
            }
        },

        // ─── Learn ────────────────────────────────────────
        Some(Commands::Learn { text, r#type, domain, project, entity }) => {
            match crud::note::learn(
                &cwd,
                &config.namespace,
                &text,
                &r#type,
                Some(&domain),
                project.as_deref(),
                entity.as_deref(),
            ) {
                Ok(slug) => println!("Learned: '{text}' (slug: {slug}, type: {}, domain: {domain})", r#type),
                Err(e) => eprintln!("Failed: {e}"),
            }
        }

        // ─── Recall ─────────────────────────────────────────
        Some(Commands::Recall { keyword, domain }) => {
            if keyword.is_none() && domain.is_none() {
                eprintln!("Provide --keyword and/or --domain");
                return;
            }
            let _ = crud::note::recall(&cwd, &config.namespace, keyword.as_deref(), domain.as_deref());
        }

        // ─── Install ─────────────────────────────────────────
        Some(Commands::Install { carl, skip_hooks }) => {
            let carl_path = carl.as_ref().map(std::path::Path::new);
            if let Err(e) = base::install::run(carl_path, skip_hooks) {
                eprintln!("Install failed: {e}");
            }
        }

        // ─── Scaffold ─────────────────────────────────────────
        Some(Commands::Scaffold { path }) => {
            let target = path
                .as_ref()
                .map(std::path::PathBuf::from)
                .unwrap_or(cwd.clone());
            if let Err(e) = base::scaffold::run(&target) {
                eprintln!("Scaffold failed: {e}");
            }
        }

        // ─── Operator ─────────────────────────────────────────
        Some(Commands::Operator { action }) => match action {
            OperatorAction::Init { name } => {
                if let Err(e) = base::operator::init(&name) {
                    eprintln!("Failed: {e}");
                }
            }
            OperatorAction::Show => base::operator::show(),
        },

        None => eprintln!("No command provided. Run `base --help` for usage."),
    }
}
