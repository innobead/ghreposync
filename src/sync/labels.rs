use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use crate::github::{
    client::GithubClient,
    model::{CreateLabelRequest, UpdateLabelRequest},
};

use super::{RepoRef, SyncReport, Syncer};

pub struct LabelSyncer {
    client: GithubClient,
    dry_run: bool,
}

impl LabelSyncer {
    pub fn new(client: GithubClient, dry_run: bool) -> Self {
        Self { client, dry_run }
    }
}

#[async_trait]
impl Syncer for LabelSyncer {
    async fn sync(&self, source: &RepoRef, target: &RepoRef) -> Result<SyncReport> {
        let source_labels = self.client.list_labels(&source.owner, &source.repo).await?;
        let target_labels = self.client.list_labels(&target.owner, &target.repo).await?;

        // Index target labels by lowercase name for case-insensitive lookup.
        let target_map: HashMap<String, _> = target_labels
            .into_iter()
            .map(|l| (l.name.to_lowercase(), l))
            .collect();

        let mut report = SyncReport {
            resource: "Labels",
            source: source.full_name(),
            target: target.full_name(),
            created: Vec::new(),
            updated: Vec::new(),
            skipped: Vec::new(),
            dry_run: self.dry_run,
        };

        for label in &source_labels {
            let key = label.name.to_lowercase();

            match target_map.get(&key) {
                Some(existing) => {
                    let needs_update =
                        existing.color != label.color || existing.description != label.description;

                    if needs_update {
                        if !self.dry_run {
                            self.client
                                .update_label(
                                    &target.owner,
                                    &target.repo,
                                    &existing.name,
                                    &UpdateLabelRequest {
                                        new_name: None,
                                        color: Some(label.color.clone()),
                                        description: label.description.clone(),
                                    },
                                )
                                .await?;
                        }
                        report.updated.push(label.name.clone());
                    } else {
                        report.skipped.push(label.name.clone());
                    }
                }
                None => {
                    if !self.dry_run {
                        self.client
                            .create_label(
                                &target.owner,
                                &target.repo,
                                &CreateLabelRequest {
                                    name: label.name.clone(),
                                    color: label.color.clone(),
                                    description: label.description.clone(),
                                },
                            )
                            .await?;
                    }
                    report.created.push(label.name.clone());
                }
            }
        }

        Ok(report)
    }
}
