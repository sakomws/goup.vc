//! Structured JobPosting extraction from safely fetched candidate pages.

use std::time::Duration;

use anyhow::{Context, Result};
use garde::Validate;
use reqwest::{Client, redirect};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    integrations::event_page::{EventPageClient, json_ld_scripts},
    types::jobs::JobInput,
};

#[derive(Clone)]
pub(crate) struct JobPageClient {
    pages: EventPageClient,
    http: Client,
}

impl JobPageClient {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            pages: EventPageClient::new()?,
            http: Client::builder()
                .timeout(Duration::from_secs(20))
                .redirect(redirect::Policy::none())
                .user_agent("GOUP-job-discovery/1.0")
                .build()?,
        })
    }

    pub(crate) async fn fetch(
        &self,
        candidate_url: &str,
        approved_source_url: &str,
    ) -> Result<Option<JobInput>> {
        let Some(html) = self.pages.fetch_html(candidate_url, approved_source_url).await? else {
            return Ok(None);
        };
        Ok(extract_job_json_ld(&html, candidate_url))
    }

    /// Resolves a Greenhouse board explicitly embedded by an approved careers source.
    pub(crate) async fn discover_greenhouse_jobs(&self, source_url: &str) -> Result<Vec<JobInput>> {
        let Some(html) = self.pages.fetch_html(source_url, source_url).await? else {
            return Ok(vec![]);
        };
        let Some(board) = greenhouse_board_from_html(&html) else {
            return Ok(vec![]);
        };
        let url = format!("https://boards-api.greenhouse.io/v1/boards/{board}/jobs?content=true");
        let response: GreenhouseResponse = self
            .http
            .get(&url)
            .send()
            .await
            .context("requesting embedded Greenhouse job board")?
            .error_for_status()
            .context("embedded Greenhouse job board returned an error")?
            .json()
            .await
            .context("decoding embedded Greenhouse job board")?;
        Ok(response
            .jobs
            .into_iter()
            .filter_map(|job| greenhouse_job_to_input(job, source_url))
            .collect())
    }
}

#[derive(Deserialize)]
struct GreenhouseResponse {
    #[serde(default)]
    jobs: Vec<GreenhouseJob>,
}

#[derive(Deserialize)]
struct GreenhouseJob {
    title: String,
    absolute_url: String,
    content: String,
    #[serde(default)]
    location: Option<GreenhouseLocation>,
}

#[derive(Deserialize)]
struct GreenhouseLocation {
    name: String,
}

fn greenhouse_board_from_html(html: &str) -> Option<&str> {
    const PREFIX: &str = "boards.greenhouse.io/embed/job_board/js?for=";
    let board = html.split(PREFIX).nth(1)?.split(['&', '"', '\'', '<']).next()?;
    (!board.is_empty()
        && board
            .bytes()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, b'_' | b'-')))
    .then_some(board)
}

fn greenhouse_job_to_input(job: GreenhouseJob, source_url: &str) -> Option<JobInput> {
    let title = job.title.trim();
    let description = strip_html(&job.content);
    let company_name = company_name_from_source(source_url)?;
    let input = JobInput {
        title: title.to_owned(),
        company_name,
        summary: description.chars().take(280).collect(),
        description,
        apply_url: job.absolute_url,
        location: job.location.map(|location| location.name),
        remote: None,
        members_only: Some(false),
        tags: None,
    };
    input.validate().ok().map(|_| input)
}

fn company_name_from_source(source_url: &str) -> Option<String> {
    let source = reqwest::Url::parse(source_url).ok()?;
    let host = source.host_str()?.trim_start_matches("www.");
    let mut company = host.to_owned();
    if let Some(first) = company.get_mut(..1) {
        first.make_ascii_uppercase();
    }
    Some(company)
}

fn strip_html(input: &str) -> String {
    let mut text = String::with_capacity(input.len());
    let mut inside_tag = false;
    for character in input.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                text.push(' ');
            }
            _ if !inside_tag => text.push(character),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn extract_job_json_ld(html: &str, candidate_url: &str) -> Option<JobInput> {
    json_ld_scripts(html)
        .filter_map(|json| serde_json::from_str::<Value>(json).ok())
        .find_map(|value| job_from_json_ld(&value, candidate_url))
}

fn job_from_json_ld(value: &Value, candidate_url: &str) -> Option<JobInput> {
    match value {
        Value::Array(values) => {
            values.iter().find_map(|value| job_from_json_ld(value, candidate_url))
        }
        Value::Object(object) => {
            if is_job_posting(object.get("@type")) {
                return job_from_object(object, candidate_url);
            }
            object
                .get("@graph")
                .and_then(|graph| job_from_json_ld(graph, candidate_url))
        }
        _ => None,
    }
}

fn is_job_posting(value: Option<&Value>) -> bool {
    match value {
        Some(Value::String(kind)) => kind.rsplit('/').next() == Some("JobPosting"),
        Some(Value::Array(kinds)) => kinds.iter().any(|kind| is_job_posting(Some(kind))),
        _ => false,
    }
}

fn job_from_object(
    object: &serde_json::Map<String, Value>,
    candidate_url: &str,
) -> Option<JobInput> {
    let title = object.get("title").or_else(|| object.get("name"))?.as_str()?.trim();
    let company_name = object.get("hiringOrganization")?.get("name")?.as_str()?.trim();
    let description = object.get("description")?.as_str()?.trim();
    let location = object
        .get("jobLocation")
        .and_then(|value| value.get("address"))
        .and_then(|value| value.get("addressLocality"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let remote = object
        .get("jobLocationType")
        .and_then(Value::as_str)
        .is_some_and(|value| value.eq_ignore_ascii_case("TELECOMMUTE"));
    let input = JobInput {
        title: title.to_owned(),
        company_name: company_name.to_owned(),
        summary: description.chars().take(280).collect(),
        description: description.to_owned(),
        apply_url: candidate_url.to_owned(),
        location,
        remote: Some(remote),
        members_only: Some(false),
        tags: None,
    };
    input.validate().ok().map(|_| input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_job_posting_from_json_ld() {
        let job = extract_job_json_ld(
            r#"<script type="application/ld+json">{
              "@type":"JobPosting","title":"Rust Engineer","description":"Build reliable systems.",
              "hiringOrganization":{"name":"Acme"},"jobLocationType":"TELECOMMUTE"
            }</script>"#,
            "https://jobs.example.com/roles/rust-engineer",
        )
        .unwrap();
        assert_eq!(job.title, "Rust Engineer");
        assert_eq!(job.company_name, "Acme");
        assert_eq!(job.remote, Some(true));
    }

    #[test]
    fn extracts_embedded_greenhouse_board() {
        assert_eq!(
            greenhouse_board_from_html(
                r#"<script src="https://boards.greenhouse.io/embed/job_board/js?for=youcom"></script>"#
            ),
            Some("youcom")
        );
    }

    #[test]
    fn maps_greenhouse_job_to_input() {
        let job = greenhouse_job_to_input(
            GreenhouseJob {
                title: "Senior Rust Engineer".into(),
                absolute_url: "https://job-boards.greenhouse.io/acme/jobs/1".into(),
                content: "<p>Build reliable systems.</p>".into(),
                location: Some(GreenhouseLocation {
                    name: "Remote".into(),
                }),
            },
            "https://www.acme.com/careers",
        )
        .unwrap();
        assert_eq!(job.company_name, "Acme.com");
        assert_eq!(job.description, "Build reliable systems.");
        assert_eq!(job.location.as_deref(), Some("Remote"));
    }
}
