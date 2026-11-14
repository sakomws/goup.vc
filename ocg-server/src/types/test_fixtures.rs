//! Shared sample data builders for domain-type tests.

use std::collections::BTreeMap;

use chrono::{TimeZone, Utc};
use serde_json::json;
use uuid::Uuid;

use crate::types::{
    dashboard::group::event_analytics::EventAnalytics,
    inbox::{
        InboxConversation, InboxConversationStatus, InboxConversationSummary, InboxEvent,
        InboxMessage, InboxMessageKind,
    },
    site::{SiteSettings, Theme},
    user::User,
};

pub(crate) fn sample_event_analytics() -> EventAnalytics {
    serde_json::from_value(json!({
        "audience": {
            "questions": [{
                "kind": "single-select",
                "options": [{ "count": 3, "label": "Developer", "option_id": Uuid::from_u128(11) }],
                "prompt": "Role",
                "question_id": Uuid::from_u128(10),
                "respondents": 3,
                "selections": 3
            }],
            "window_end_capped": false,
            "window_end_inferred": true,
            "window_start_inferred": true,
            "members_joined_in_window": 4,
            "members_joined_in_window_registered": 2,
            "new_attendees": 3,
            "returning_attendees": 1,
            "window_ends_at": 1710097200,
            "window_starts_at": 1709892000
        },
        "check_ins": {
            "bin_minutes": 15,
            "bins": [[1710087300000_i64, 1], [1710088200000_i64, 2]],
            "outside_range": 0
        },
        "enrollment": {
            "admission_offers": [{ "count": 1, "outcome": "deadline-passed", "source": "waitlist" }],
            "cancellations": 1,
            "checked_in": 3,
            "invitation_requests": { "accepted": 1, "pending": 0, "rejected": 0 },
            "registrations": 4,
            "registrations_by_path": {
                "approval": 0,
                "direct": 3,
                "organizer_invitation": 0,
                "unknown": 0,
                "waitlist": 1
            },
            "waitlist": 2
        },
        "event": {
            "canceled": false,
            "is_paid": true,
            "published": true,
            "timezone": "Europe/Madrid",
            "capacity": 10,
            "ends_at": 1710097200,
            "published_at": 1709892000,
            "registration_ends_at": null,
            "registration_starts_at": null,
            "starts_at": 1710090000
        },
        "markers": [{ "kind": "published", "ts": 1709892000 }],
        "markers_announcements_omitted": 0,
        "traffic": {
            "attribution_consistent": true,
            "registrations_baseline": 0,
            "registrations_by_ticket_type_per_day": [{
                "kind": "ticket-type",
                "series": [["2024-03-08", 1], ["2024-03-09", 2], ["2024-03-10", 1]],
                "event_ticket_type_id": Uuid::from_u128(20),
                "order": 1,
                "title": "General"
            }],
            "registrations_running_total": [["2024-03-08", 1], ["2024-03-09", 3], ["2024-03-10", 4]],
            "views_after_end": 3,
            "views_before_start": 2,
            "views_by_source_per_day": [
                {
                    "kind": "source",
                    "series": [["2024-03-08", 4], ["2024-03-09", 0], ["2024-03-10", 0]],
                    "source": "newsletter"
                },
                {
                    "kind": "untagged",
                    "series": [["2024-03-08", 6], ["2024-03-09", 5], ["2024-03-10", 0]],
                    "source": null
                }
            ],
            "views_total": 20
        },
        "utm": {
            "by_campaign": { "not_specified": 4, "remainder_count": 0, "remainder_views": 0, "values": [] },
            "by_medium": {
                "not_specified": 0,
                "remainder_count": 0,
                "remainder_views": 0,
                "values": [{ "value": "email", "views": 4 }]
            },
            "by_source": {
                "not_specified": 0,
                "remainder_count": 0,
                "remainder_views": 0,
                "values": [{ "value": "newsletter", "views": 4 }]
            },
            "combinations": [{
                "first_day": "2024-03-08",
                "last_day": "2024-03-08",
                "views": 4,
                "utm_campaign": null,
                "utm_medium": "email",
                "utm_source": "newsletter"
            }],
            "combinations_remainder_views": 0,
            "combinations_total_count": 1,
            "tagged_views": 4
        },
        "range": { "end_day": "2024-03-10", "start_day": "2024-03-08" },
        "tickets": {
            "discount_codes": [{
                "active": true,
                "code": "SAVE10",
                "event_discount_code_id": Uuid::from_u128(30),
                "redemptions": 2,
                "title": "Save ten",
                "available": 8,
                "total_available": 10
            }],
            "ticket_types": [{
                "active": true,
                "allocated": 5,
                "event_ticket_type_id": Uuid::from_u128(20),
                "order": 1,
                "remaining": 5,
                "seats_total": 10,
                "sold": 4,
                "title": "General"
            }]
        }
    }))
    .expect("sample event analytics should deserialize")
}

pub(crate) fn sample_inbox_conversation(inbox_conversation_id: Uuid) -> InboxConversation {
    let created_at = Utc.with_ymd_and_hms(2030, 1, 10, 10, 0, 0).unwrap();
    let user = User {
        user_id: Uuid::new_v4(),
        username: "inbox-user".to_string(),
        name: Some("Inbox User".to_string()),
        ..Default::default()
    };

    InboxConversation {
        community_display_name: "Test Alliance".to_string(),
        community_name: "test".to_string(),
        created_at,
        group_name: "Test Group".to_string(),
        group_slug: "test-group".to_string(),
        inbox_conversation_id,
        last_message_at: created_at,
        messages: vec![InboxMessage {
            body: "When do doors open?".to_string(),
            created_at,
            inbox_message_id: Uuid::new_v4(),
            kind: InboxMessageKind::Initial,
            author: Some(user.clone()),
        }],
        status: InboxConversationStatus::Open,
        event: Some(InboxEvent {
            is_public: true,
            name: "Test Event".to_string(),
            slug: Some("test-event".to_string()),
        }),
        group_slug_pretty: None,
        user: Some(user),
    }
}

pub(crate) fn sample_inbox_conversation_summary(
    inbox_conversation_id: Uuid,
) -> InboxConversationSummary {
    let last_message_at = Utc.with_ymd_and_hms(2030, 1, 10, 10, 0, 0).unwrap();

    InboxConversationSummary {
        community_display_name: "Test Alliance".to_string(),
        group_name: "Test Group".to_string(),
        inbox_conversation_id,
        last_message_at,
        last_message_excerpt: "When do doors open?".to_string(),
        last_message_kind: InboxMessageKind::Initial,
        status: InboxConversationStatus::Open,
        event: Some(InboxEvent {
            is_public: true,
            name: "Test Event".to_string(),
            slug: None,
        }),
        user: Some(User {
            user_id: Uuid::new_v4(),
            username: "inbox-user".to_string(),
            name: Some("Inbox User".to_string()),
            ..Default::default()
        }),
    }
}

pub(crate) fn sample_site_settings() -> SiteSettings {
    SiteSettings {
        description: "Test site".to_string(),
        site_id: Uuid::new_v4(),
        theme: Theme {
            palette: BTreeMap::new(),
            primary_color: "#000000".to_string(),
        },
        title: "Test Site".to_string(),
        ..Default::default()
    }
}

pub(crate) fn sample_template_user() -> User {
    User {
        user_id: Uuid::new_v4(),
        username: "organizer".to_string(),
        name: Some("Organizer".to_string()),
        ..Default::default()
    }
}
