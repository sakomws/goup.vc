//! Database operations for mock interviews.

use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::types::Json;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::PgExecutor,
    types::mock_interviews::{
        MockInterviewIntervieweeFeedbackInput, MockInterviewInterviewerFeedbackInput,
        MockInterviewMatchFilters, MockInterviewMatchesOutput, MockInterviewProfile,
        MockInterviewProfileInput, MockInterviewRequest, MockInterviewRequestInput,
        MockInterviewSession,
    },
};

/// Database operations for mock interview practice.
#[async_trait]
pub(crate) trait DBMockInterviews {
    /// Get a user's mock interview profile.
    async fn get_mock_interview_profile(
        &self,
        user_id: Uuid,
    ) -> Result<Option<MockInterviewProfile>>;

    /// Create or update a mock interview profile.
    async fn upsert_mock_interview_profile(
        &self,
        user_id: Uuid,
        input: &MockInterviewProfileInput,
    ) -> Result<MockInterviewProfile>;

    /// Search suggested interviewer matches.
    async fn search_mock_interview_matches(
        &self,
        user_id: Uuid,
        filters: &MockInterviewMatchFilters,
    ) -> Result<MockInterviewMatchesOutput>;

    /// List requests involving the user.
    async fn list_mock_interview_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<MockInterviewRequest>>;

    /// Create a mock interview request.
    async fn add_mock_interview_request(
        &self,
        user_id: Uuid,
        input: &MockInterviewRequestInput,
    ) -> Result<MockInterviewRequest>;

    /// Accept, decline, or cancel a request.
    async fn respond_mock_interview_request(
        &self,
        user_id: Uuid,
        request_id: Uuid,
        action: &str,
        meeting_url: Option<&str>,
    ) -> Result<Option<Uuid>>;

    /// Get a session the user participates in.
    async fn get_mock_interview_session(
        &self,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<Option<MockInterviewSession>>;

    /// Submit interviewer feedback.
    async fn add_mock_interview_interviewer_feedback(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        input: &MockInterviewInterviewerFeedbackInput,
    ) -> Result<()>;

    /// Submit interviewee feedback.
    async fn add_mock_interview_interviewee_feedback(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        input: &MockInterviewIntervieweeFeedbackInput,
    ) -> Result<()>;
}

#[async_trait]
impl<T> DBMockInterviews for T
where
    T: PgExecutor + Send + Sync,
{
    #[instrument(skip(self), err)]
    async fn get_mock_interview_profile(
        &self,
        user_id: Uuid,
    ) -> Result<Option<MockInterviewProfile>> {
        self.fetch_json_opt("select get_mock_interview_profile($1::uuid)", &[&user_id])
            .await
    }

    #[instrument(skip(self, input), err)]
    async fn upsert_mock_interview_profile(
        &self,
        user_id: Uuid,
        input: &MockInterviewProfileInput,
    ) -> Result<MockInterviewProfile> {
        let payload = crate::types::mock_interviews::profile_input_to_json(input);
        self.fetch_json_one(
            "select upsert_mock_interview_profile($1::uuid, $2::jsonb)",
            &[&user_id, &Json(payload)],
        )
        .await
    }

    #[instrument(skip(self, filters), err)]
    async fn search_mock_interview_matches(
        &self,
        user_id: Uuid,
        filters: &MockInterviewMatchFilters,
    ) -> Result<MockInterviewMatchesOutput> {
        self.fetch_json_one(
            "select search_mock_interview_matches($1::uuid, $2::jsonb)",
            &[&user_id, &Json(filters)],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn list_mock_interview_requests(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<MockInterviewRequest>> {
        self.fetch_json_one("select list_mock_interview_requests($1::uuid)", &[&user_id])
            .await
    }

    #[instrument(skip(self, input), err)]
    async fn add_mock_interview_request(
        &self,
        user_id: Uuid,
        input: &MockInterviewRequestInput,
    ) -> Result<MockInterviewRequest> {
        self.fetch_json_one(
            "select add_mock_interview_request($1::uuid, $2::jsonb)",
            &[&user_id, &Json(input)],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn respond_mock_interview_request(
        &self,
        user_id: Uuid,
        request_id: Uuid,
        action: &str,
        meeting_url: Option<&str>,
    ) -> Result<Option<Uuid>> {
        let output: serde_json::Value = self.fetch_json_one(
            "select respond_mock_interview_request($1::uuid, $2::uuid, $3::text, $4::text)",
            &[&user_id, &request_id, &action, &meeting_url],
        )
        .await?;
        Ok(output
            .get("session_id")
            .and_then(|value| value.as_str())
            .and_then(|value| Uuid::parse_str(value).ok()))
    }

    #[instrument(skip(self), err)]
    async fn get_mock_interview_session(
        &self,
        user_id: Uuid,
        session_id: Uuid,
    ) -> Result<Option<MockInterviewSession>> {
        self.fetch_json_opt(
            "select get_mock_interview_session($1::uuid, $2::uuid)",
            &[&user_id, &session_id],
        )
        .await
    }

    #[instrument(skip(self, input), err)]
    async fn add_mock_interview_interviewer_feedback(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        input: &MockInterviewInterviewerFeedbackInput,
    ) -> Result<()> {
        self.execute(
            "select add_mock_interview_feedback($1::uuid, $2::uuid, $3::jsonb)",
            &[&user_id, &session_id, &Json(input)],
        )
        .await?;
        Ok(())
    }

    #[instrument(skip(self, input), err)]
    async fn add_mock_interview_interviewee_feedback(
        &self,
        user_id: Uuid,
        session_id: Uuid,
        input: &MockInterviewIntervieweeFeedbackInput,
    ) -> Result<()> {
        self.execute(
            "select add_mock_interview_feedback($1::uuid, $2::uuid, $3::jsonb)",
            &[&user_id, &session_id, &Json(input)],
        )
        .await?;
        Ok(())
    }
}
