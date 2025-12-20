//! Thin CLI for repo-lens that maps subcommands to API requests.
//!
//! This binary provides a command-line interface to repo-lens functionality.
//! By default, it outputs JSON for machine consumption. Use --pretty for human-readable output.

use clap::{Parser, Subcommand};
use rl_api::{request::*, ApiVersion, Request};
use rl_core::RepoEngine;
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "repo-lens")]
#[command(about = "High-performance Git UI backend")]
#[command(version)]
struct Cli {
    /// Repository path
    #[arg(short, long, global = true)]
    repo: Option<String>,

    /// Output pretty-printed JSON instead of compact JSON
    #[arg(long, global = true)]
    pretty: bool,

    /// Page size for paginated commands
    #[arg(long, global = true, default_value = "50")]
    page_size: u32,

    /// Cursor for pagination
    #[arg(long, global = true, default_value = "")]
    cursor: String,

    /// Timeout in milliseconds
    #[arg(long, global = true)]
    timeout_ms: Option<u64>,

    /// Log filter (e.g., debug, rl_core=trace, rl_git=debug)
    #[arg(long, global = true)]
    log: Option<String>,

    /// Output logs as JSON
    #[arg(long, global = true)]
    log_json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get repository status
    Status,
    /// Get commit log
    Log {
        /// Revision range (optional)
        revision_range: Option<String>,
    },
    /// Get commit graph window
    Graph {
        /// Revision range (optional)
        revision_range: Option<String>,
    },
    /// Show commit details
    Show {
        /// Commit ID
        commit_id: String,
    },
    /// Get diff summary
    DiffSummary {
        /// From revision
        #[arg(long)]
        from: Option<String>,
        /// To revision
        #[arg(long)]
        to: Option<String>,
    },
    /// Get diff content
    Diff {
        /// From revision
        #[arg(long)]
        from: Option<String>,
        /// To revision
        #[arg(long)]
        to: Option<String>,
        /// Path filter
        #[arg(long)]
        path: Option<String>,
    },
    /// Get blame information
    Blame {
        /// File path
        path: String,
        /// Revision
        #[arg(long)]
        revision: Option<String>,
    },
    /// List branches
    Branches,
    /// List tags
    Tags,
    /// List remotes
    Remotes,
    /// Checkout operation
    Checkout {
        /// Target to checkout
        target: String,
        /// Create new branch
        #[arg(long)]
        create_branch: bool,
    },
    /// Commit operation
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,
        /// Author name
        #[arg(long)]
        author_name: Option<String>,
        /// Author email
        #[arg(long)]
        author_email: Option<String>,
    },
    /// Fetch operation
    Fetch {
        /// Remote name
        #[arg(long)]
        remote: Option<String>,
        /// Refspecs to fetch
        #[arg(long)]
        refspecs: Option<Vec<String>>,
    },
    /// Push operation
    Push {
        /// Remote name
        #[arg(long)]
        remote: Option<String>,
        /// Refspecs to push
        #[arg(long)]
        refspecs: Option<Vec<String>>,
        /// Force push
        #[arg(long)]
        force: bool,
    },
    /// Merge operation
    Merge {
        /// Source branch/commit
        source: String,
        /// Commit message
        #[arg(long)]
        message: Option<String>,
    },
    /// Rebase operation
    Rebase {
        /// Branch/commit to rebase onto
        onto: String,
        /// Upstream branch
        #[arg(long)]
        upstream: Option<String>,
    },
    /// Stash operation
    Stash {
        /// Stash message
        #[arg(long)]
        message: Option<String>,
    },
    /// Watch for repository changes
    Watch,
    /// Run benchmarks
    Bench,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    rl_core::telemetry::init_telemetry(cli.log.as_deref(), cli.log_json);

    // Get repository path
    let repo_path = cli.repo.unwrap_or_else(|| ".".to_string());

    // Create request based on command
    let request_payload = match cli.command {
        Commands::Status => RequestPayload::Status(StatusRequest {
            repo_path: repo_path.clone(),
        }),
        Commands::Log { revision_range } => RequestPayload::Log(LogRequest {
            repo_path: repo_path.clone(),
            paging: rl_api::Paging {
                page_size: rl_api::PageSize::try_from(cli.page_size).unwrap(),
                cursor: rl_api::Cursor::from(cli.cursor.clone()),
            },
            revision_range,
        }),
        Commands::Graph { revision_range } => RequestPayload::Graph(GraphRequest {
            repo_path: repo_path.clone(),
            window_size: rl_api::WindowSize::try_from(cli.page_size).unwrap(),
            cursor: rl_api::Cursor::from(cli.cursor.clone()),
            revision_range,
        }),
        Commands::Show { commit_id } => RequestPayload::ShowCommit(ShowCommitRequest {
            repo_path: repo_path.clone(),
            commit_id,
        }),
        Commands::DiffSummary { from, to } => RequestPayload::DiffSummary(DiffSummaryRequest {
            repo_path: repo_path.clone(),
            from,
            to,
            max_bytes: rl_api::MaxBytes::try_from(1024 * 1024).unwrap(), // 1MB default
            max_hunks: rl_api::MaxHunks::try_from(1000).unwrap(),
        }),
        Commands::Diff { from, to, path } => RequestPayload::DiffContent(DiffContentRequest {
            repo_path: repo_path.clone(),
            from,
            to,
            path,
            max_bytes: rl_api::MaxBytes::try_from(1024 * 1024).unwrap(), // 1MB default
        }),
        Commands::Blame { path, revision } => RequestPayload::Blame(BlameRequest {
            repo_path: repo_path.clone(),
            path,
            revision,
        }),
        Commands::Branches => RequestPayload::Branches(BranchesRequest {
            repo_path: repo_path.clone(),
        }),
        Commands::Tags => RequestPayload::Tags(TagsRequest {
            repo_path: repo_path.clone(),
        }),
        Commands::Remotes => RequestPayload::Remotes(RemotesRequest {
            repo_path: repo_path.clone(),
        }),
        Commands::Checkout {
            target,
            create_branch,
        } => RequestPayload::Checkout(CheckoutRequest {
            repo_path: repo_path.clone(),
            target,
            create_branch,
        }),
        Commands::Commit {
            message,
            author_name,
            author_email,
        } => RequestPayload::Commit(CommitRequest {
            repo_path: repo_path.clone(),
            message,
            author_name,
            author_email,
        }),
        Commands::Fetch { remote, refspecs } => RequestPayload::Fetch(FetchRequest {
            repo_path: repo_path.clone(),
            remote,
            refspecs,
        }),
        Commands::Push {
            remote,
            refspecs,
            force,
        } => RequestPayload::Push(PushRequest {
            repo_path: repo_path.clone(),
            remote,
            refspecs,
            force,
        }),
        Commands::Merge { source, message } => RequestPayload::Merge(MergeRequest {
            repo_path: repo_path.clone(),
            source,
            message,
        }),
        Commands::Rebase { onto, upstream } => RequestPayload::Rebase(RebaseRequest {
            repo_path: repo_path.clone(),
            onto,
            upstream,
        }),
        Commands::Stash { message } => RequestPayload::Stash(StashRequest {
            repo_path: repo_path.clone(),
            message,
        }),
        Commands::Watch => RequestPayload::Watch(WatchRequest {
            repo_path: repo_path.clone(),
        }),
        Commands::Bench => {
            // For bench command, delegate to the bench binary
            eprintln!("Use 'repo-lens-bench' for benchmarking");
            std::process::exit(1);
        }
    };

    let request = Request {
        version: ApiVersion::V0,
        id: "cli-request".to_string(),
        payload: request_payload,
    };

    // Create engine and handle request
    let engine = RepoEngine::new();
    let response = engine.handle(request).await;

    // Output response
    let json = if cli.pretty {
        serde_json::to_string_pretty(&response)?
    } else {
        serde_json::to_string(&response)?
    };

    writeln!(io::stdout(), "{}", json)?;

    Ok(())
}
