//! Shared GTM dashboard templates.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::types::{
    gtm::{GTM_AGENTS, GTM_KINDS, GTM_STAGES, GtmLead, GtmLeadFilters, stage_label},
    pagination::NavigationLinks,
};

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

    fn lost_reason_value(&self) -> &str {
        self.lead.lost_reason.as_deref().unwrap_or("")
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
