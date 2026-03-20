use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use crate::github::{
    client::GithubClient,
    model::{CreateMilestoneRequest, UpdateMilestoneRequest},
};

use super::{RepoRef, SyncReport, Syncer};

pub struct MilestoneSyncer {
    client: GithubClient,
    dry_run: bool,
}

impl MilestoneSyncer {
    pub fn new(client: GithubClient, dry_run: bool) -> Self {
        Self { client, dry_run }
    }
}

#[async_trait]
impl Syncer for MilestoneSyncer {
    async fn sync(&self, source: &RepoRef, target: &RepoRef) -> Result<SyncReport> {
        let source_milestones = self
            .client
            .list_milestones(&source.owner, &source.repo)
            .await?;
        let target_milestones = self
            .client
            .list_milestones(&target.owner, &target.repo)
            .await?;

        // Index target milestones by lowercase title for case-insensitive lookup.
        let target_map: HashMap<String, _> = target_milestones
            .into_iter()
            .map(|m| (m.title.to_lowercase(), m))
            .collect();

        let mut report = SyncReport {
            resource: "Milestones",
            source: source.full_name(),
            target: target.full_name(),
            created: Vec::new(),
            updated: Vec::new(),
            skipped: Vec::new(),
            dry_run: self.dry_run,
        };

        for milestone in &source_milestones {
            let key = milestone.title.to_lowercase();

            match target_map.get(&key) {
                Some(existing) => {
                    let needs_update = existing.description != milestone.description
                        || existing.state != milestone.state
                        || existing.due_on != milestone.due_on;

                    if needs_update {
                        if !self.dry_run {
                            self.client
                                .update_milestone(
                                    &target.owner,
                                    &target.repo,
                                    existing.number,
                                    &UpdateMilestoneRequest {
                                        title: None,
                                        description: milestone.description.clone(),
                                        state: Some(milestone.state.clone()),
                                        due_on: milestone.due_on.clone(),
                                    },
                                )
                                .await?;
                        }
                        report.updated.push(milestone.title.clone());
                    } else {
                        report.skipped.push(milestone.title.clone());
                    }
                }
                None => {
                    if !self.dry_run {
                        self.client
                            .create_milestone(
                                &target.owner,
                                &target.repo,
                                &CreateMilestoneRequest {
                                    title: milestone.title.clone(),
                                    description: milestone.description.clone(),
                                    state: Some(milestone.state.clone()),
                                    due_on: milestone.due_on.clone(),
                                },
                            )
                            .await?;
                    }
                    report.created.push(milestone.title.clone());
                }
            }
        }

        Ok(report)
    }
}
