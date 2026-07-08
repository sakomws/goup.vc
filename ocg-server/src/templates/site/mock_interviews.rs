//! Templates for mock interview practice pages.

use askama::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    templates::{PageId, auth::User},
    types::{
        mock_interviews::{
            MockInterviewMatchFilters, MockInterviewProfile, MockInterviewRequest,
            MockInterviewSession,
        },
        site::SiteSettings,
    },
};

/// Landing page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/mock_interviews/page.html")]
pub(crate) struct Page {
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub profile: Option<MockInterviewProfile>,
}

/// Onboarding page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/mock_interviews/onboarding.html")]
pub(crate) struct OnboardingPage {
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub profile: Option<MockInterviewProfile>,
    pub interview_types: &'static [&'static str],
    pub target_company_types: &'static [&'static str],
}

impl OnboardingPage {
    pub(crate) fn profile_role_intent(&self) -> &str {
        self.profile.as_ref().map_or("interviewee", |profile| match profile.role_intent {
            crate::types::mock_interviews::RoleIntent::Interviewee => "interviewee",
            crate::types::mock_interviews::RoleIntent::Interviewer => "interviewer",
            crate::types::mock_interviews::RoleIntent::Both => "both",
        })
    }

    pub(crate) fn profile_timezone_region(&self) -> &str {
        self.profile.as_ref().map_or("usa_canada", |profile| match profile.timezone_region {
            crate::types::mock_interviews::TimezoneRegion::Aze => "aze",
            crate::types::mock_interviews::TimezoneRegion::Eu => "eu",
            crate::types::mock_interviews::TimezoneRegion::UsaCanada => "usa_canada",
            crate::types::mock_interviews::TimezoneRegion::Asia => "asia",
            crate::types::mock_interviews::TimezoneRegion::Other => "other",
        })
    }

    pub(crate) fn profile_seniority(&self) -> &str {
        self.profile.as_ref().map_or("mid", |profile| match profile.seniority {
            crate::types::mock_interviews::Seniority::Junior => "junior",
            crate::types::mock_interviews::Seniority::Mid => "mid",
            crate::types::mock_interviews::Seniority::Senior => "senior",
            crate::types::mock_interviews::Seniority::StaffPlus => "staff_plus",
        })
    }

    pub(crate) fn profile_interview_types_csv(&self) -> String {
        self.profile
            .as_ref()
            .map(|profile| profile.interview_types.join(", "))
            .unwrap_or_default()
    }

    pub(crate) fn profile_target_company_types_csv(&self) -> String {
        self.profile
            .as_ref()
            .map(|profile| profile.target_company_types.join(", "))
            .unwrap_or_default()
    }

    pub(crate) fn profile_availability_slots_json(&self) -> String {
        self.profile
            .as_ref()
            .map(|profile| profile.availability_slots.to_string())
            .unwrap_or_else(|| "[]".to_string())
    }
}

/// Matches page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/mock_interviews/matches.html")]
pub(crate) struct MatchesPage {
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub profile: Option<MockInterviewProfile>,
    pub filters: MockInterviewMatchFilters,
    pub matches: Vec<MockInterviewProfile>,
    pub total: usize,
    pub interview_types: &'static [&'static str],
}

/// Requests page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/mock_interviews/requests.html")]
pub(crate) struct RequestsPage {
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub current_user_id: Uuid,
    pub requests: Vec<MockInterviewRequest>,
}

/// Session page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "site/mock_interviews/session.html")]
pub(crate) struct SessionPage {
    pub page_id: PageId,
    pub path: String,
    pub site_settings: SiteSettings,
    pub user: User,
    pub current_user_id: Uuid,
    pub session: MockInterviewSession,
}
