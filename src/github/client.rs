use anyhow::{Context, Result};
use reqwest::{Client, header};

use super::model::{
    CreateLabelRequest, CreateMilestoneRequest, Label, Milestone, UpdateLabelRequest,
    UpdateMilestoneRequest,
};

const BASE_URL: &str = "https://api.github.com";
const USER_AGENT: &str = concat!("ghreposync/", env!("CARGO_PKG_VERSION"));

#[derive(Clone)]
pub struct GithubClient {
    client: Client,
}

impl GithubClient {
    pub fn new(token: Option<String>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            header::HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static(USER_AGENT),
        );

        if let Some(token) = token {
            let value = format!("Bearer {token}");
            headers.insert(
                header::AUTHORIZATION,
                header::HeaderValue::from_str(&value).context("Invalid token format")?,
            );
        }

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { client })
    }

    pub async fn list_labels(&self, owner: &str, repo: &str) -> Result<Vec<Label>> {
        let mut labels = Vec::new();
        let mut page = 1u32;

        loop {
            let url = format!("{BASE_URL}/repos/{owner}/{repo}/labels?per_page=100&page={page}");
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .context("Failed to list labels")?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("GitHub API error {status}: {body}");
            }

            let page_labels: Vec<Label> = resp.json().await.context("Failed to parse labels")?;
            if page_labels.is_empty() {
                break;
            }
            labels.extend(page_labels);
            page += 1;
        }

        Ok(labels)
    }

    pub async fn create_label(
        &self,
        owner: &str,
        repo: &str,
        req: &CreateLabelRequest,
    ) -> Result<Label> {
        let url = format!("{BASE_URL}/repos/{owner}/{repo}/labels");
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .context("Failed to create label")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to create label '{}': {status} {body}", req.name);
        }

        resp.json().await.context("Failed to parse created label")
    }

    pub async fn update_label(
        &self,
        owner: &str,
        repo: &str,
        name: &str,
        req: &UpdateLabelRequest,
    ) -> Result<Label> {
        let encoded = urlencoding::encode(name);
        let url = format!("{BASE_URL}/repos/{owner}/{repo}/labels/{encoded}");
        let resp = self
            .client
            .patch(&url)
            .json(req)
            .send()
            .await
            .context("Failed to update label")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to update label '{name}': {status} {body}");
        }

        resp.json().await.context("Failed to parse updated label")
    }

    pub async fn list_milestones(&self, owner: &str, repo: &str) -> Result<Vec<Milestone>> {
        let mut milestones = Vec::new();
        let mut page = 1u32;

        loop {
            let url = format!(
                "{BASE_URL}/repos/{owner}/{repo}/milestones?state=all&per_page=100&page={page}"
            );
            let resp = self
                .client
                .get(&url)
                .send()
                .await
                .context("Failed to list milestones")?;

            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                anyhow::bail!("GitHub API error {status}: {body}");
            }

            let page_items: Vec<Milestone> =
                resp.json().await.context("Failed to parse milestones")?;
            if page_items.is_empty() {
                break;
            }
            milestones.extend(page_items);
            page += 1;
        }

        Ok(milestones)
    }

    pub async fn create_milestone(
        &self,
        owner: &str,
        repo: &str,
        req: &CreateMilestoneRequest,
    ) -> Result<Milestone> {
        let url = format!("{BASE_URL}/repos/{owner}/{repo}/milestones");
        let resp = self
            .client
            .post(&url)
            .json(req)
            .send()
            .await
            .context("Failed to create milestone")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "Failed to create milestone '{}': {status} {body}",
                req.title
            );
        }

        resp.json()
            .await
            .context("Failed to parse created milestone")
    }

    pub async fn update_milestone(
        &self,
        owner: &str,
        repo: &str,
        number: u64,
        req: &UpdateMilestoneRequest,
    ) -> Result<Milestone> {
        let url = format!("{BASE_URL}/repos/{owner}/{repo}/milestones/{number}");
        let resp = self
            .client
            .patch(&url)
            .json(req)
            .send()
            .await
            .context("Failed to update milestone")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Failed to update milestone #{number}: {status} {body}");
        }

        resp.json()
            .await
            .context("Failed to parse updated milestone")
    }
}
