//! DNS ownership verification for custom hostnames.

use anyhow::{Context, Result};
use serde::Deserialize;

const DNS_QUERY_ENDPOINT: &str = "https://cloudflare-dns.com/dns-query";

#[derive(Debug, Deserialize)]
struct DnsAnswer {
    data: String,
    #[serde(rename = "type")]
    record_type: u16,
}

#[derive(Debug, Deserialize)]
struct DnsResponse {
    #[serde(rename = "Answer", default)]
    answers: Vec<DnsAnswer>,
}

fn txt_answer_matches(answer: &DnsAnswer, expected_token: &str) -> bool {
    answer.record_type == 16
        && answer
            .data
            .chars()
            .filter(|character| !matches!(character, '"' | '\\') && !character.is_whitespace())
            .collect::<String>()
            == expected_token
}

/// Checks for the expected GOUP verification token in a DNS TXT record.
pub(crate) async fn verify_txt_record(hostname: &str, expected_token: &str) -> Result<bool> {
    let record_name = format!("_goup-verification.{hostname}");
    let mut url = reqwest::Url::parse(DNS_QUERY_ENDPOINT)?;
    url.query_pairs_mut()
        .append_pair("name", &record_name)
        .append_pair("type", "TXT");
    let response = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::ACCEPT, "application/dns-json")
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .context("DNS verification request failed")?
        .error_for_status()
        .context("DNS verification service returned an error")?
        .json::<DnsResponse>()
        .await
        .context("DNS verification response was invalid")?;

    Ok(response
        .answers
        .iter()
        .any(|answer| txt_answer_matches(answer, expected_token)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_response_deserializes_txt_answers() {
        let response: DnsResponse = serde_json::from_str(
            r#"{"Status":0,"Answer":[{"name":"_goup-verification.events.example.com","type":16,"TTL":300,"data":"\"goup-verification=test-token\""}]}"#,
        )
        .unwrap();

        assert_eq!(response.answers.len(), 1);
        assert_eq!(response.answers[0].record_type, 16);
        assert_eq!(
            response.answers[0].data.trim_matches('"'),
            "goup-verification=test-token"
        );
        assert!(txt_answer_matches(
            &response.answers[0],
            "goup-verification=test-token"
        ));
    }

    #[test]
    fn test_split_txt_chunks_are_joined() {
        let answer = DnsAnswer {
            data: "\"goup-verification=\" \"test-token\"".to_string(),
            record_type: 16,
        };

        assert!(txt_answer_matches(&answer, "goup-verification=test-token"));
    }
}
