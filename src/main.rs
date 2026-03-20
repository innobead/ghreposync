mod cli;
mod github;
mod sync;

use anyhow::{Context, Result};
use clap::Parser;

use cli::{Cli, Commands, Resource};
use github::client::GithubClient;
use sync::{RepoRef, Syncer, labels::LabelSyncer, milestones::MilestoneSyncer};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync {
            source,
            target,
            dry_run,
            resources,
        } => {
            let source_ref = RepoRef::parse(&source).context("Invalid --source")?;
            let target_ref = RepoRef::parse(&target).context("Invalid --target")?;
            let client = GithubClient::new(cli.token)?;

            let all = resources.contains(&Resource::All);

            if all || resources.contains(&Resource::Labels) {
                let syncer = LabelSyncer::new(client.clone(), dry_run);
                let report = syncer.sync(&source_ref, &target_ref).await?;
                report.print();
            }

            if all || resources.contains(&Resource::Milestones) {
                let syncer = MilestoneSyncer::new(client, dry_run);
                let report = syncer.sync(&source_ref, &target_ref).await?;
                report.print();
            }
        }
    }

    Ok(())
}
