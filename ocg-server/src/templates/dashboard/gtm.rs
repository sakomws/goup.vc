//! Shared GTM dashboard templates.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::types::{
    gtm::{
        GTM_AGENTS, GTM_KINDS, GTM_LOST_REASONS, GTM_STAGES, GtmAgentDraft, GtmLead,
        GtmLeadFilters, GtmSponsorPackage, GtmTask, stage_label,
    },
    pagination::NavigationLinks,
};

/// Candidate displayed while a lead-generation draft awaits review.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(crate) struct LeadGenerationCandidate {
    pub name: String,
    pub kind: String,
    pub org_name: Option<String>,
    pub email: Option<String>,
    pub website_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub source: Option<String>,
    pub notes: Option<String>,
}

/// Latest scope-specific lead-generation draft and its candidates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct LeadGenerationDraft {
    pub draft: GtmAgentDraft,
    pub candidates: Vec<LeadGenerationCandidate>,
    pub older_pending_count: usize,
}

/// GTM list page used by alliance and group dashboards.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/gtm/list.html")]
pub(crate) struct ListPage {
    /// Whether the user can mutate GTM records.
    pub can_manage_gtm: bool,
    /// Alliance or group dashboard base path.
    pub dashboard_base: String,
    /// Dashboard home tab URL.
    pub dashboard_tab_url: String,
    /// Current filters.
    pub filters: GtmLeadFilters,
    /// Matching leads.
    pub leads: Vec<GtmLead>,
    /// Total matching leads.
    pub total: usize,
    /// Counts keyed by stage.
    pub stage_counts: serde_json::Map<String, serde_json::Value>,
    /// Pagination links.
    pub navigation_links: NavigationLinks,
    /// Sponsor packages available in this scope.
    pub packages: Vec<GtmSponsorPackage>,
    /// Open tasks currently due.
    pub due_tasks: Vec<GtmTask>,
    /// Latest pending lead-generation draft in this exact scope.
    pub lead_generation_draft: Option<LeadGenerationDraft>,
}

#[allow(clippy::unused_self)]
impl ListPage {
    fn kinds(&self) -> &'static [&'static str] {
        &GTM_KINDS
    }

    fn stages(&self) -> &'static [&'static str] {
        &GTM_STAGES
    }

    fn stage_label(&self, stage: &str) -> &'static str {
        stage_label(stage)
    }

    fn stage_count(&self, stage: &str) -> i64 {
        self.stage_counts
            .get(stage)
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0)
    }

    fn is_kind(&self, kind: &str) -> bool {
        self.filters.kind.as_deref() == Some(kind)
    }

    fn is_stage(&self, stage: &str) -> bool {
        self.filters.stage.as_deref() == Some(stage)
    }

    fn query_value(&self) -> &str {
        self.filters.query.as_deref().unwrap_or("")
    }

    fn has_pending_lead_generation_draft(&self) -> bool {
        self.lead_generation_draft.is_some()
    }
}

/// Lead detail page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/gtm/detail.html")]
pub(crate) struct DetailPage {
    /// Whether the user can mutate GTM records.
    pub can_manage_gtm: bool,
    /// Alliance or group dashboard base path.
    pub dashboard_base: String,
    /// Dashboard home tab URL.
    pub dashboard_tab_url: String,
    /// Lead plus activity and drafts.
    pub lead: GtmLead,
    /// Sponsor packages available for proposal creation.
    pub packages: Vec<GtmSponsorPackage>,
}

#[allow(clippy::unused_self)]
impl DetailPage {
    fn kinds(&self) -> &'static [&'static str] {
        &GTM_KINDS
    }

    fn stages(&self) -> &'static [&'static str] {
        &GTM_STAGES
    }

    fn agents(&self) -> &'static [&'static str] {
        &GTM_AGENTS
    }

    fn stage_label(&self, stage: &str) -> &'static str {
        stage_label(stage)
    }

    fn pending_draft(&self) -> Option<&crate::types::gtm::GtmAgentDraft> {
        self.lead.drafts.iter().find(|draft| draft.status == "pending")
    }

    fn is_lead_kind(&self, kind: &str) -> bool {
        self.lead.kind == kind
    }

    fn is_lead_stage(&self, stage: &str) -> bool {
        self.lead.stage == stage
    }

    fn notes_value(&self) -> &str {
        self.lead.notes.as_deref().unwrap_or("")
    }

    fn next_action_value(&self) -> String {
        self.lead
            .next_action_at
            .map(|value| value.format("%Y-%m-%dT%H:%M").to_string())
            .unwrap_or_default()
    }

    fn renewal_value(&self) -> String {
        self.lead
            .renewal_at
            .map(|value| value.format("%Y-%m-%dT%H:%M").to_string())
            .unwrap_or_default()
    }

    fn lost_reasons(&self) -> &'static [(&'static str, &'static str)] {
        &GTM_LOST_REASONS
    }

    fn is_lost_reason(&self, code: &str) -> bool {
        self.lead.lost_reason.as_deref() == Some(code)
    }

    fn is_suggested_stage(&self, stage: &str) -> bool {
        self.pending_draft()
            .and_then(|draft| draft.suggested_stage.as_deref())
            == Some(stage)
    }

    fn is_won_lost_draft(&self) -> bool {
        self.pending_draft().is_some_and(|draft| draft.agent_id == "won_lost")
    }

    fn is_lead_generation_agent(&self, agent: &str) -> bool {
        agent == "lead_generation"
    }
}
