use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;

pub mod labels;
pub mod milestones;

/// Parsed `owner/repo` reference.
pub struct RepoRef {
    pub owner: String,
    pub repo: String,
}

impl RepoRef {
    pub fn parse(s: &str) -> Result<Self> {
        let (owner, repo) = s
            .split_once('/')
            .filter(|(o, r)| !o.is_empty() && !r.is_empty())
            .ok_or_else(|| anyhow::anyhow!("Invalid repo '{s}': expected 'owner/repo'"))?;
        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

/// Result of a sync operation.
pub struct SyncReport {
    pub resource: &'static str,
    pub source: String,
    pub target: String,
    pub created: Vec<String>,
    pub updated: Vec<String>,
    pub skipped: Vec<String>,
    pub dry_run: bool,
}

impl SyncReport {
    pub fn print(&self) {
        println!(
            "\n{} {} → {}",
            format!("[{}]", self.resource).cyan().bold(),
            self.source.yellow(),
            self.target.yellow()
        );

        if self.dry_run {
            println!("{}", "(dry run – no changes applied)".italic());
        }

        for name in &self.created {
            println!("  {} {}", "+".green().bold(), name);
        }
        for name in &self.updated {
            println!("  {} {}", "~".yellow().bold(), name);
        }
        for name in &self.skipped {
            println!("  {} {} {}", "·".dimmed(), name, "(unchanged)".dimmed());
        }

        println!(
            "\n  {} created  {} updated  {} unchanged",
            self.created.len().to_string().green(),
            self.updated.len().to_string().yellow(),
            self.skipped.len().to_string().dimmed(),
        );
    }
}

/// Implemented by every resource syncer.
#[async_trait]
pub trait Syncer {
    async fn sync(&self, source: &RepoRef, target: &RepoRef) -> Result<SyncReport>;
}
