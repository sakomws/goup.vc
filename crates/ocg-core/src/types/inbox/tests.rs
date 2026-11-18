use garde::Validate;
use uuid::Uuid;

use crate::types::tests::sample_inbox_conversation;

use super::*;

#[test]
fn test_event_url_links_public_events() {
    let conversation = sample_inbox_conversation(Uuid::new_v4());

    assert_eq!(
        conversation.event_url().as_deref(),
        Some("/test/group/test-group/event/test-event")
    );
}

#[test]
fn test_event_url_skips_events_that_are_not_public() {
    let mut conversation = sample_inbox_conversation(Uuid::new_v4());
    conversation.event.as_mut().unwrap().is_public = false;

    assert_eq!(conversation.event_url(), None);
}

#[test]
fn test_event_url_skips_removed_events() {
    let mut conversation = sample_inbox_conversation(Uuid::new_v4());
    conversation.event = None;

    assert_eq!(conversation.event_url(), None);
}

#[test]
fn test_event_url_uses_pretty_group_slug() {
    let mut conversation = sample_inbox_conversation(Uuid::new_v4());
    conversation.group_slug_pretty = Some("pretty".to_string());

    assert_eq!(
        conversation.event_url().as_deref(),
        Some("/test/group/pretty/event/test-event")
    );
}

#[test]
fn test_inbox_summary_accepts_alliance_display_name_from_database() {
    let summary: InboxConversationSummary = serde_json::from_str(
        r#"{
            "alliance_display_name": "Open Alliance",
            "group_name": "Baku",
            "inbox_conversation_id": "00000000-0000-0000-0000-000000000002",
            "last_message_at": 1894276800,
            "last_message_excerpt": "When do doors open?",
            "last_message_kind": "user-reply",
            "status": "open"
        }"#,
    )
    .unwrap();

    assert_eq!(summary.community_display_name, "Open Alliance");
    assert_eq!(summary.last_message_excerpt, "When do doors open?");
}

#[test]
fn test_inbox_conversation_accepts_alliance_fields_from_database() {
    let conversation: InboxConversation = serde_json::from_str(
        r#"{
            "alliance_display_name": "Open Alliance",
            "alliance_name": "goup",
            "created_at": 1894269600,
            "group_name": "Baku",
            "group_slug": "baku",
            "inbox_conversation_id": "00000000-0000-0000-0000-000000000002",
            "last_message_at": 1894276800,
            "messages": [],
            "status": "open"
        }"#,
    )
    .unwrap();

    assert_eq!(conversation.community_display_name, "Open Alliance");
    assert_eq!(conversation.community_name, "goup");
}

#[test]
fn test_inbox_contact_context_accepts_alliance_name_from_database() {
    let context: InboxContactContext = serde_json::from_str(
        r#"{
            "alliance_name": "goup",
            "event_id": "00000000-0000-0000-0000-000000000001",
            "event_name": "Demo",
            "event_slug": "demo",
            "group_name": "Baku",
            "group_slug": "baku"
        }"#,
    )
    .unwrap();

    assert_eq!(context.community_name, "goup");
    assert_eq!(context.event_path(), "/goup/group/baku/event/demo");
}

#[test]
fn test_inbox_contact_context_event_path_uses_pretty_group_slug() {
    let context = InboxContactContext {
        community_name: "test".to_string(),
        event_id: Uuid::new_v4(),
        event_name: "Test Event".to_string(),
        event_slug: "test-event".to_string(),
        group_name: "Test Group".to_string(),
        group_slug: "test-group".to_string(),

        group_slug_pretty: Some("pretty".to_string()),
        viewer: None,
    };

    assert_eq!(context.event_path(), "/test/group/pretty/event/test-event");
}

#[test]
fn test_inbox_conversation_status_label_and_encoding() {
    assert_eq!(InboxConversationStatus::Answered.label(), "Answered");
    assert_eq!(InboxConversationStatus::Closed.label(), "Closed");
    assert_eq!(InboxConversationStatus::Open.label(), "Open");
    assert_eq!(InboxConversationStatus::Spam.label(), "Spam");
    assert_eq!(InboxConversationStatus::Answered.to_string(), "answered");
    assert_eq!(
        "closed".parse::<InboxConversationStatus>().unwrap(),
        InboxConversationStatus::Closed
    );
    assert_eq!(
        "spam".parse::<InboxConversationStatus>().unwrap(),
        InboxConversationStatus::Spam
    );
}

#[test]
fn test_inbox_message_input_accepts_5000_multibyte_characters() {
    let input = InboxMessageInput {
        body: "é".repeat(5000),
    };

    assert!(input.validate().is_ok());
}

#[test]
fn test_inbox_message_input_rejects_5001_characters() {
    let input = InboxMessageInput {
        body: "a".repeat(5001),
    };

    assert!(input.validate().is_err());
}

#[test]
fn test_inbox_message_input_rejects_whitespace_only_body() {
    let input = InboxMessageInput {
        body: " \n\t ".to_string(),
    };

    assert!(input.validate().is_err());
}

#[test]
fn test_inbox_message_kind_is_from_group() {
    assert!(InboxMessageKind::GroupReply.is_from_group());
    assert!(!InboxMessageKind::Initial.is_from_group());
    assert!(!InboxMessageKind::UserReply.is_from_group());
}

#[test]
fn test_is_read_only_when_user_was_deleted() {
    let mut conversation = sample_inbox_conversation(Uuid::new_v4());
    assert!(!conversation.is_read_only());

    conversation.user = None;

    assert!(conversation.is_read_only());
}
