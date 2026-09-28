//! Database operations for authenticated event surveys.

use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::types::Json;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::PgExecutor,
    types::{
        questionnaire::QuestionnaireAnswers,
        survey::{EventSurvey, EventSurveyDashboard},
    },
};

#[async_trait]
pub(crate) trait DBSurvey {
    async fn get_event_survey_for_user(
        &self,
        alliance_id: Uuid,
        event_id: Uuid,
        audience: &str,
        user_id: Uuid,
    ) -> Result<Option<EventSurvey>>;

    async fn submit_event_survey_response(
        &self,
        alliance_id: Uuid,
        event_id: Uuid,
        audience: &str,
        user_id: Uuid,
        answers: &QuestionnaireAnswers,
    ) -> Result<()>;

    async fn get_event_survey_dashboard(
        &self,
        group_id: Uuid,
        event_id: Uuid,
        audience: Option<String>,
    ) -> Result<EventSurveyDashboard>;
}

#[async_trait]
impl<T> DBSurvey for T
where
    T: PgExecutor + Send + Sync,
{
    #[instrument(skip(self), err)]
    async fn get_event_survey_for_user(
        &self,
        alliance_id: Uuid,
        event_id: Uuid,
        audience: &str,
        user_id: Uuid,
    ) -> Result<Option<EventSurvey>> {
        self.fetch_json_opt(
            "select get_event_survey_for_user($1::uuid, $2::uuid, $3::text, $4::uuid)",
            &[&alliance_id, &event_id, &audience, &user_id],
        )
        .await
    }

    #[instrument(skip(self, answers), err)]
    async fn submit_event_survey_response(
        &self,
        alliance_id: Uuid,
        event_id: Uuid,
        audience: &str,
        user_id: Uuid,
        answers: &QuestionnaireAnswers,
    ) -> Result<()> {
        self.execute(
            "select submit_event_survey_response($1::uuid, $2::uuid, $3::text, $4::uuid, $5::jsonb)",
            &[&alliance_id, &event_id, &audience, &user_id, &Json(answers)],
        )
        .await
    }

    #[instrument(skip(self), err)]
    async fn get_event_survey_dashboard(
        &self,
        group_id: Uuid,
        event_id: Uuid,
        audience: Option<String>,
    ) -> Result<EventSurveyDashboard> {
        self.fetch_json_one(
            "select get_event_survey_dashboard($1::uuid, $2::uuid, $3::text)",
            &[&group_id, &event_id, &audience.as_deref()],
        )
        .await
    }
}
