//! Draft-only GTM agents. None of these functions mutate lead stage.

use anyhow::{Result, anyhow};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    db::DynDB,
    types::gtm::{GtmLead, suggested_stage_for_agent},
};

/// Runs one GTM agent and stores a pending draft for human review.
pub(crate) async fn run_agent(
    db: &DynDB,
    actor_user_id: Uuid,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
    gtm_lead_id: Option<Uuid>,
    agent_id: &str,
    reply: Option<&str>,
) -> Result<Uuid> {
    let lead = if let Some(lead_id) = gtm_lead_id {
        Some(
            db.get_gtm_lead(alliance_id, lead_id)
                .await?
                .ok_or_else(|| anyhow!("gtm lead not found"))?,
        )
    } else {
        None
    };

    if agent_id != "lead_generation" && lead.is_none() {
        return Err(anyhow!("gtm lead is required for this agent"));
    }

    let draft = match agent_id {
        "lead_generation" => lead_generation_draft(db, alliance_id, group_id).await?,
        "reachout" => reachout_draft(lead.as_ref().expect("lead")),
        "get_response" => get_response_draft(lead.as_ref().expect("lead"), reply),
        "qualification" => qualification_draft(lead.as_ref().expect("lead")),
        "proposal" => proposal_draft(lead.as_ref().expect("lead")),
        "negotiation" => negotiation_draft(lead.as_ref().expect("lead")),
        "won_lost" => won_lost_draft(lead.as_ref().expect("lead")),
        "delivered" => delivered_draft(lead.as_ref().expect("lead")),
        "renewal" => renewal_draft(lead.as_ref().expect("lead")),
        other => return Err(anyhow!("unknown gtm agent: {other}")),
    };

    db.add_gtm_agent_draft(actor_user_id, alliance_id, &draft).await
}

async fn lead_generation_draft(
    db: &DynDB,
    alliance_id: Uuid,
    group_id: Option<Uuid>,
) -> Result<Value> {
    let suggestions = db.suggest_gtm_lead_candidates(alliance_id, group_id, 12).await?;
    let count = suggestions.candidates.len();
    Ok(json!({
        "group_id": group_id,
        "agent_id": "lead_generation",
        "title": format!("Review {count} suggested GTM leads"),
        "body": "Approve to create these candidate leads at Lead generation. Duplicates are skipped.",
        "suggested_stage": "lead_generation",
        "payload": {
            "candidates": suggestions.candidates
        }
    }))
}

fn reachout_draft(lead: &GtmLead) -> Value {
    let org = lead.org_name.as_deref().unwrap_or(&lead.name);
    let body = format!(
        "Hi {name},\n\nI am reaching out from our {kind} program because {org} looks like a strong fit.\n\nWould you have 20 minutes this or next week to talk about a partnership?\n\nThanks,\nGOUP",
        name = lead.name,
        kind = lead.kind,
        org = org
    );
    json!({
        "gtm_lead_id": lead.gtm_lead_id,
        "group_id": lead.group_id,
        "agent_id": "reachout",
        "title": format!("Outreach draft for {}", lead.name),
        "body": body,
        "suggested_stage": suggested_stage_for_agent("reachout", &lead.stage),
        "payload": { "channel": "email" }
    })
}

fn get_response_draft(lead: &GtmLead, reply: Option<&str>) -> Value {
    let reply = reply.unwrap_or("").trim();
    let (summary, sentiment) = if reply.is_empty() {
        (
            "No reply pasted. Mark this if the lead responded offline.".to_string(),
            "unknown",
        )
    } else {
        let lower = reply.to_ascii_lowercase();
        let sentiment = if lower.contains("not interested")
            || lower.contains("no thanks")
            || lower.contains("unsubscribe")
        {
            "negative"
        } else if lower.contains("interested")
            || lower.contains("sounds good")
            || lower.contains("let's")
            || lower.contains("lets")
        {
            "positive"
        } else {
            "neutral"
        };
        (format!("Response summary: {sentiment}. {reply}"), sentiment)
    };
    json!({
        "gtm_lead_id": lead.gtm_lead_id,
        "group_id": lead.group_id,
        "agent_id": "get_response",
        "title": format!("Response logged for {}", lead.name),
        "body": summary,
        "suggested_stage": suggested_stage_for_agent("get_response", &lead.stage),
        "payload": { "sentiment": sentiment, "reply": reply }
    })
}

fn qualification_draft(lead: &GtmLead) -> Value {
    let mut score = 40;
    let mut reasons = vec![format!("Kind is {}", lead.kind)];
    if lead.email.is_some() {
        score += 15;
        reasons.push("Has an email".into());
    }
    if lead.website_url.is_some() {
        score += 10;
        reasons.push("Has a website".into());
    }
    if lead.org_name.is_some() {
        score += 10;
        reasons.push("Has an organization".into());
    }
    if matches!(lead.kind.as_str(), "sponsor" | "investor") {
        score += 10;
        reasons.push("Revenue-adjacent kind".into());
    }
    if lead.notes.as_deref().is_some_and(|notes| !notes.is_empty()) {
        score += 5;
        reasons.push("Has notes from prior research".into());
    }
    score = score.min(100);
    let suggested = if score < 35 { "lost" } else { "qualification" };
    json!({
        "gtm_lead_id": lead.gtm_lead_id,
        "group_id": lead.group_id,
        "agent_id": "qualification",
        "title": format!("Qualification score {score} for {}", lead.name),
        "body": reasons.join(". "),
        "suggested_stage": suggested,
        "payload": { "score": score, "reasons": reasons }
    })
}

fn proposal_draft(lead: &GtmLead) -> Value {
    let package = match lead.kind.as_str() {
        "sponsor" => "a community sponsorship package (logo, talk slot, and newsletter mention)",
        "startup" => "a landscape listing plus intro to the relevant city chapter",
        "investor" => "an investor landscape profile and curated founder introductions",
        "speaker" => "a featured speaking slot at the next chapter event",
        "organizer" => "an organizer role on the group team (pending their acceptance)",
        _ => "a partnership package",
    };
    let body = format!(
        "Hi {name},\n\nFollowing our conversation, here is a proposed next step for {org}: {package}.\n\nIf this looks right, we can lock dates and terms this week.\n\nThanks,\nGOUP",
        name = lead.name,
        org = lead.org_name.as_deref().unwrap_or(&lead.name),
        package = package
    );
    json!({
        "gtm_lead_id": lead.gtm_lead_id,
        "group_id": lead.group_id,
        "agent_id": "proposal",
        "title": format!("Proposal for {}", lead.name),
        "body": body,
        "suggested_stage": suggested_stage_for_agent("proposal", &lead.stage),
        "payload": { "package": package }
    })
}

fn negotiation_draft(lead: &GtmLead) -> Value {
    let body = format!(
        "Hi {name},\n\nThanks for the feedback. We can flex on timing and the included deliverables while keeping the core partnership intact.\n\nSuggested close: confirm terms this week and start delivery on the next event cycle.\n\nThanks,\nGOUP",
        name = lead.name
    );
    json!({
        "gtm_lead_id": lead.gtm_lead_id,
        "group_id": lead.group_id,
        "agent_id": "negotiation",
        "title": format!("Negotiation reply for {}", lead.name),
        "body": body,
        "suggested_stage": suggested_stage_for_agent("negotiation", &lead.stage),
        "payload": { "recommended_terms": "confirm this week" }
    })
}

fn won_lost_draft(lead: &GtmLead) -> Value {
    let positive =
        lead.score.unwrap_or(50) >= 50 && lead.lost_reason.as_deref().is_none_or(str::is_empty);
    let (stage, title, body) = if positive {
        (
            "won",
            format!("Recommend marking {} as won", lead.name),
            "The thread and score look strong enough to close. Approve to mark Won. Optional side effects stay unchecked until you choose them.".to_string(),
        )
    } else {
        (
            "lost",
            format!("Recommend marking {} as lost", lead.name),
            "There is not enough signal to close. Approve to mark Lost, or edit the suggested stage to Won.".to_string(),
        )
    };
    json!({
        "gtm_lead_id": lead.gtm_lead_id,
        "group_id": lead.group_id,
        "agent_id": "won_lost",
        "title": title,
        "body": body,
        "suggested_stage": stage,
        "payload": {
            "side_effects": {
                "create_sponsor": false,
                "create_landscape_entry": false,
                "invite_organizer": false
            }
        }
    })
}

fn delivered_draft(lead: &GtmLead) -> Value {
    let checklist = match lead.kind.as_str() {
        "sponsor" => [
            "Add logo to the group page",
            "Confirm event mention",
            "Send recap",
        ],
        "startup" | "investor" => [
            "Publish landscape entry",
            "Introduce to the chapter",
            "Confirm listing copy",
        ],
        "speaker" => ["Confirm talk title", "Add to event page", "Collect slides"],
        "organizer" => [
            "Send team invite",
            "Share the group-leads runbook",
            "Pair with a co-admin",
        ],
        _ => [
            "Confirm deliverable",
            "Share recap",
            "Ask for a testimonial",
        ],
    };
    json!({
        "gtm_lead_id": lead.gtm_lead_id,
        "group_id": lead.group_id,
        "agent_id": "delivered",
        "title": format!("Delivery checklist for {}", lead.name),
        "body": checklist.join("\n"),
        "suggested_stage": suggested_stage_for_agent("delivered", &lead.stage),
        "payload": { "checklist": checklist }
    })
}

fn renewal_draft(lead: &GtmLead) -> Value {
    let body = format!(
        "Hi {name},\n\nIt has been a cycle since we closed the last partnership with {org}. We would like to renew and keep the same (or expanded) deliverables.\n\nAre you open to a short renewal conversation this month?\n\nThanks,\nGOUP",
        name = lead.name,
        org = lead.org_name.as_deref().unwrap_or(&lead.name)
    );
    json!({
        "gtm_lead_id": lead.gtm_lead_id,
        "group_id": lead.group_id,
        "agent_id": "renewal",
        "title": format!("Renewal outreach for {}", lead.name),
        "body": body,
        "suggested_stage": suggested_stage_for_agent("renewal", &lead.stage),
        "payload": { "cycle": "renewal" }
    })
}
