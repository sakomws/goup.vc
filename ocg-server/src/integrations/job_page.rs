//! Structured JobPosting extraction from safely fetched candidate pages.

use anyhow::Result;
use garde::Validate;
use serde_json::Value;

use crate::{
    integrations::event_page::{EventPageClient, json_ld_scripts},
    types::jobs::JobInput,
};

#[derive(Clone)]
pub(crate) struct JobPageClient {
    pages: EventPageClient,
}

impl JobPageClient {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            pages: EventPageClient::new()?,
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
}
