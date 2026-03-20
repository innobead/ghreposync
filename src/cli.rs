use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(
    name = "ghreposync",
    about = "Sync GitHub repository resources between repos",
    version
)]
pub struct Cli {
    /// GitHub API token (or set GITHUB_TOKEN env var)
    #[arg(long, env = "GITHUB_TOKEN", global = true, hide_env_values = true)]
    pub token: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Sync resources from a source repo to a target repo
    Sync {
        /// Source repository in `owner/repo` format
        #[arg(short, long)]
        source: String,

        /// Target repository in `owner/repo` format
        #[arg(short, long)]
        target: String,

        /// Preview changes without applying them
        #[arg(long)]
        dry_run: bool,

        /// Resources to sync. Accepts one or more comma-separated values, or `all`.
        #[arg(
            short,
            long = "resource",
            value_enum,
            num_args = 1..,
            value_delimiter = ',',
            default_value = "all"
        )]
        resources: Vec<Resource>,
    },
}

#[derive(Clone, PartialEq, Eq, ValueEnum)]
pub enum Resource {
    /// All supported resources
    All,
    /// Repository labels
    Labels,
    /// Repository milestones
    Milestones,
}
