use anyhow::anyhow;
use axum::{
    body::{Body, to_bytes},
    http::{
        HeaderValue, Request, StatusCode,
        header::{CONTENT_TYPE, COOKIE},
    },
};
use axum_login::tower_sessions::session;
use serde_json::from_value;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    db::mock::MockDB,
    handlers::tests::*,
    services::notifications::{MockNotificationsManager, NotificationKind},
    templates::{
        dashboard::DASHBOARD_PAGINATION_LIMIT,
        notifications::{CfsSubmissionUpdated, EventCustom},
    },
    types::permissions::GroupPermission,
};

#[tokio::test]
async fn test_list_page_success() {
    // Setup identifiers and data structures
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );
    let session_proposal_id = Uuid::new_v4();
    let cfs_submission_id = Uuid::new_v4();
    let speaker_id = Uuid::new_v4();
    let event = sample_event_summary(event_id, group_id);
    let submissions_output =
        crate::templates::dashboard::group::submissions::CfsSubmissionsOutput {
            label_counts: vec![],
            submissions: vec![sample_group_cfs_submission(
                cfs_submission_id,
                session_proposal_id,
                speaker_id,
            )],
            total: 1,
            unlabeled_count: 0,
        };
    let statuses = vec![sample_group_cfs_submission_status("submitted", "Submitted")];

    // Setup database mock
    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::Read
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_event_summary()
        .times(1)
        .withf(move |cid, gid, eid| *cid == alliance_id && *gid == group_id && *eid == event_id)
        .returning(move |_, _, _| Ok(event.clone()));
    db.expect_list_event_cfs_labels()
        .times(1)
        .withf(move |eid| *eid == event_id)
        .returning(|_| Ok(vec![]));
    db.expect_list_cfs_submission_statuses_for_review()
        .times(1)
        .returning(move || Ok(statuses.clone()));
    db.expect_list_event_cfs_submissions()
        .times(1)
        .withf(move |eid, filters| {
            *eid == event_id
                && filters.limit == Some(DASHBOARD_PAGINATION_LIMIT)
                && filters.offset == Some(0)
        })
        .returning(move |_, _| Ok(submissions_output.clone()));

    // Setup notifications manager mock
    let nm = MockNotificationsManager::new();

    // Setup router and send request
    let router = TestRouterBuilder::new(db, nm).build().await;
    let request = Request::builder()
        .method("GET")
        .uri(format!("/dashboard/group/events/{event_id}/submissions"))
        .header(COOKIE, format!("id={session_id}"))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();

    // Check response matches expectations
    assert_eq!(parts.status, StatusCode::OK);
    assert_eq!(
        parts.headers.get(CONTENT_TYPE).unwrap(),
        &HeaderValue::from_static("text/html; charset=utf-8"),
    );
    assert!(!bytes.is_empty());
}

#[test]
fn csv_safe_prevents_spreadsheet_formula_execution() {
    for value in ["=HYPERLINK(\"https://evil.test\")", "+1", "-1", "@cmd"] {
        assert!(super::csv_safe(value).starts_with('\''));
    }
    assert_eq!(super::csv_safe("Safe title"), "Safe title");
}

#[tokio::test]
async fn test_list_page_with_pagination_params() {
    // Setup identifiers and data structures
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );
    let event = sample_event_summary(event_id, group_id);
    let submissions_output =
        crate::templates::dashboard::group::submissions::CfsSubmissionsOutput {
            label_counts: vec![],
            submissions: vec![],
            total: 0,
            unlabeled_count: 0,
        };
    let statuses = vec![sample_group_cfs_submission_status("submitted", "Submitted")];

    // Setup database mock
    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::Read
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_event_summary()
        .times(1)
        .withf(move |cid, gid, eid| *cid == alliance_id && *gid == group_id && *eid == event_id)
        .returning(move |_, _, _| Ok(event.clone()));
    db.expect_list_event_cfs_labels()
        .times(1)
        .withf(move |eid| *eid == event_id)
        .returning(|_| Ok(vec![]));
    db.expect_list_cfs_submission_statuses_for_review()
        .times(1)
        .returning(move || Ok(statuses.clone()));
    db.expect_list_event_cfs_submissions()
        .times(1)
        .withf(move |eid, filters| {
            *eid == event_id && filters.limit == Some(5) && filters.offset == Some(10)
        })
        .returning(move |_, _| Ok(submissions_output.clone()));

    // Setup notifications manager mock
    let nm = MockNotificationsManager::new();

    // Setup router and send request
    let router = TestRouterBuilder::new(db, nm).build().await;
    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/dashboard/group/events/{event_id}/submissions?limit=5&offset=10"
        ))
        .header(COOKIE, format!("id={session_id}"))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();

    // Check response matches expectations
    assert_eq!(parts.status, StatusCode::OK);
    assert_eq!(
        parts.headers.get(CONTENT_TYPE).unwrap(),
        &HeaderValue::from_static("text/html; charset=utf-8"),
    );
    assert!(!bytes.is_empty());
}

#[tokio::test]
async fn test_list_page_with_no_labels_filter() {
    // Setup identifiers and data structures
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let label_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );
    let event = sample_event_summary(event_id, group_id);
    let labels = vec![crate::types::event::EventCfsLabel {
        color: "#DBEAFE".to_string(),
        event_cfs_label_id: Some(label_id),
        name: "track / backend".to_string(),
    }];
    let submissions_output =
        crate::templates::dashboard::group::submissions::CfsSubmissionsOutput {
            label_counts: vec![],
            submissions: vec![],
            total: 0,
            unlabeled_count: 3,
        };
    let statuses = vec![sample_group_cfs_submission_status("submitted", "Submitted")];

    // Setup database mock
    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::Read
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_event_summary()
        .times(1)
        .withf(move |cid, gid, eid| *cid == alliance_id && *gid == group_id && *eid == event_id)
        .returning(move |_, _, _| Ok(event.clone()));
    db.expect_list_event_cfs_labels()
        .times(1)
        .withf(move |eid| *eid == event_id)
        .returning(move |_| Ok(labels.clone()));
    db.expect_list_cfs_submission_statuses_for_review()
        .times(1)
        .returning(move || Ok(statuses.clone()));
    db.expect_list_event_cfs_submissions()
        .times(1)
        .withf(move |eid, filters| {
            *eid == event_id && filters.no_labels == Some(true) && filters.label_ids.is_none()
        })
        .returning(move |_, _| Ok(submissions_output.clone()));

    // Setup notifications manager mock
    let nm = MockNotificationsManager::new();

    // Setup router and send request
    let router = TestRouterBuilder::new(db, nm).build().await;
    let request = Request::builder()
        .method("GET")
        .uri(format!(
            "/dashboard/group/events/{event_id}/submissions?no_labels=true"
        ))
        .header(COOKIE, format!("id={session_id}"))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();

    // Check response matches expectations
    assert_eq!(parts.status, StatusCode::OK);
    assert!(html.contains("include-no-label"));
    assert!(html.contains("no-label-count=\"3\""));
    assert!(html.contains("no-label-selected"));
}

#[tokio::test]
async fn test_list_page_db_error() {
    // Setup identifiers and data structures
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );

    // Setup database mock
    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::Read
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_event_summary()
        .times(1)
        .withf(move |cid, gid, eid| *cid == alliance_id && *gid == group_id && *eid == event_id)
        .returning(|_, _, _| Err(anyhow!("db error")));

    // Setup notifications manager mock
    let nm = MockNotificationsManager::new();

    // Setup router and send request
    let router = TestRouterBuilder::new(db, nm).build().await;
    let request = Request::builder()
        .method("GET")
        .uri(format!("/dashboard/group/events/{event_id}/submissions"))
        .header(COOKIE, format!("id={session_id}"))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();

    // Check response matches expectations
    assert_eq!(parts.status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(bytes.is_empty());
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_update_success() {
    // Setup identifiers and data structures
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let cfs_submission_id = Uuid::new_v4();
    let notification_user_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );
    let event = sample_event_summary(event_id, group_id);
    let update = crate::templates::dashboard::group::submissions::CfsSubmissionUpdate {
        label_ids: vec![],
        status_id: "approved".to_string(),
        action_required_message: Some("Please update your slides.".to_string()),
        rating_comment: None,
        rating_stars: None,
    };
    let form_data = serde_qs::to_string(&update).unwrap();
    let notification_data =
        crate::templates::dashboard::group::submissions::CfsSubmissionNotificationData {
            status_id: update.status_id.clone(),
            status_name: "Approved".to_string(),
            user_id: notification_user_id,

            action_required_message: update.action_required_message.clone(),
        };
    let site_settings = sample_site_settings();
    let expected_link = "/dashboard/user?tab=submissions".to_string();
    let theme_primary_color = site_settings.theme.primary_color.clone();

    // Setup database mock
    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_event_summary()
        .times(1)
        .withf(move |cid, gid, eid| *cid == alliance_id && *gid == group_id && *eid == event_id)
        .returning(move |_, _, _| Ok(event.clone()));
    db.expect_update_cfs_submission()
        .times(1)
        .withf(move |uid, eid, sid, submission| {
            *uid == user_id
                && *eid == event_id
                && *sid == cfs_submission_id
                && submission.status_id == "approved"
                && submission.label_ids.is_empty()
                && submission.action_required_message.as_deref()
                    == Some("Please update your slides.")
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_cfs_submission_notification_data()
        .times(1)
        .withf(move |eid, sid| *eid == event_id && *sid == cfs_submission_id)
        .returning(move |_, _| Ok(notification_data.clone()));
    db.expect_get_site_settings()
        .times(1)
        .returning(move || Ok(site_settings.clone()));

    // Setup notifications manager mock
    let mut nm = MockNotificationsManager::new();
    nm.expect_enqueue()
        .times(1)
        .withf(move |notification| {
            matches!(notification.kind, NotificationKind::CfsSubmissionUpdated)
                && notification.recipients == vec![notification_user_id]
                && notification.template_data.as_ref().is_some_and(|value| {
                    from_value::<CfsSubmissionUpdated>(value.clone()).is_ok_and(|template| {
                        template.action_required_message.as_deref()
                            == Some("Please update your slides.")
                            && template.event.event_id == event_id
                            && template.link == expected_link
                            && template.status_name == "Approved"
                            && template.theme.primary_color == theme_primary_color
                    })
                })
        })
        .returning(|_| Box::pin(async { Ok(()) }));

    // Setup router and send request
    let router = TestRouterBuilder::new(db, nm).build().await;
    let request = Request::builder()
        .method("PUT")
        .uri(format!(
            "/dashboard/group/events/{event_id}/submissions/{cfs_submission_id}"
        ))
        .header(COOKIE, format!("id={session_id}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();

    // Check response matches expectations
    assert_eq!(parts.status, StatusCode::NO_CONTENT);
    assert_eq!(
        parts.headers.get("HX-Trigger").unwrap(),
        &HeaderValue::from_static("refresh-event-submissions"),
    );
    assert!(bytes.is_empty());
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_send_email_to_approved_speakers() {
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let speaker_id = Uuid::new_v4();
    let co_speaker_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );
    let event = sample_event_summary(event_id, group_id);
    let event_for_db = event.clone();
    let event_for_notifications = event.clone();
    let site_settings = sample_site_settings();
    let site_settings_for_notifications = site_settings.clone();
    let notification_body = "Your proposal was approved.";
    let notification_subject = "Proposal approved";
    let expected_link = format!(
        "/{}/group/{}/event/{}",
        event.alliance_name, event.group_slug, event.slug
    );
    let form_data = serde_qs::to_string(&super::CfsSubmissionEmail {
        audience: super::CfsEmailAudience::Approved,
        body: notification_body.to_string(),
        subject: notification_subject.to_string(),
    })
    .unwrap();
    let track_user_id = user_id;
    let track_event_id = event_id;
    let track_group_id = group_id;
    let track_subject = notification_subject.to_string();
    let track_body = notification_body.to_string();

    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_site_settings()
        .times(1)
        .returning(move || Ok(site_settings.clone()));
    db.expect_get_event_summary()
        .times(1)
        .withf(move |cid, gid, eid| *cid == alliance_id && *gid == group_id && *eid == event_id)
        .returning(move |_, _, _| Ok(event_for_db.clone()));
    db.expect_resolve_event_cfs_email_recipient_ids()
        .times(1)
        .withf(move |gid, eid, audience| {
            *gid == group_id && *eid == event_id && audience == "approved"
        })
        .returning(move |_, _, _| Ok(vec![speaker_id, co_speaker_id]));
    db.expect_enqueue_tracked_custom_notification()
        .times(1)
        .withf(move |notification, tracking| {
            matches!(notification.kind, NotificationKind::EventCustom)
                && notification.recipients == vec![speaker_id, co_speaker_id]
                && notification.template_data.as_ref().is_some_and(|value| {
                    from_value::<EventCustom>(value.clone()).is_ok_and(|template| {
                        template.subject == notification_subject
                            && template.body == notification_body
                            && template.event.name == event_for_notifications.name
                            && template.link == expected_link
                            && template.theme.primary_color
                                == site_settings_for_notifications.theme.primary_color
                    })
                })
                && tracking.created_by == track_user_id
                && tracking.alliance_id == Some(alliance_id)
                && tracking.event_id == Some(track_event_id)
                && tracking.group_id == Some(track_group_id)
                && tracking.recipient_count == 2
                && tracking.subject == track_subject
                && tracking.body == track_body
        })
        .returning(|_, _| Ok(()));

    let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
        .build()
        .await;
    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/dashboard/group/events/{event_id}/submissions/email"
        ))
        .header(COOKIE, format!("id={session_id}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();

    assert_empty_response(&parts, &bytes, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_send_email_rejects_empty_recipient_list() {
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );
    let event = sample_event_summary(event_id, group_id);
    let form_data = serde_qs::to_string(&super::CfsSubmissionEmail {
        audience: super::CfsEmailAudience::Rejected,
        body: "Body".to_string(),
        subject: "Subject".to_string(),
    })
    .unwrap();

    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_site_settings()
        .times(1)
        .returning(|| Ok(sample_site_settings()));
    db.expect_get_event_summary()
        .times(1)
        .returning(move |_, _, _| Ok(event.clone()));
    db.expect_resolve_event_cfs_email_recipient_ids()
        .times(1)
        .withf(|_, _, audience| audience == "rejected")
        .returning(|_, _, _| Ok(Vec::new()));

    let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
        .build()
        .await;
    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/dashboard/group/events/{event_id}/submissions/email"
        ))
        .header(COOKIE, format!("id={session_id}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();

    assert_eq!(parts.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        String::from_utf8(bytes.to_vec()).unwrap(),
        "No rejected speakers with verified email addresses who accept messages from event organizers."
    );
}

#[tokio::test]
async fn test_send_prep_schedule_email() {
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let submission_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );
    let event = sample_event_summary(event_id, group_id);
    let starts_at = chrono::TimeZone::with_ymd_and_hms(&chrono::Utc, 2024, 2, 1, 10, 0, 0).unwrap();
    let context = crate::templates::dashboard::group::submissions::CfsSubmissionPrepEmailContext {
        speaker_user_id: Uuid::new_v4(),
        status_id: "approved".to_string(),
        title: "Opening keynote".to_string(),
        co_speaker_user_id: None,
        linked_session_id: Some(Uuid::new_v4()),
        prep_emails: vec![],
        session_starts_at: Some(starts_at),
        tracks: vec!["AI".to_string()],
        venue_name: Some("Sample Venue".to_string()),
    };
    let expected_subject = "Your time slot for Sample Event".to_string();
    let expected_body = "Your talk \"Opening keynote\" is on the AI track at Feb 01, 2024 10:00 UTC. The venue is Sample Venue. The event page has the schedule.".to_string();

    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_site_settings()
        .times(1)
        .returning(|| Ok(sample_site_settings()));
    db.expect_get_event_summary()
        .times(1)
        .returning(move |_, _, _| Ok(event.clone()));
    db.expect_get_cfs_submission_prep_email_context()
        .times(1)
        .withf(move |gid, eid, sid| *gid == group_id && *eid == event_id && *sid == submission_id)
        .returning(move |_, _, _| Ok(Some(context.clone())));
    db.expect_send_cfs_submission_prep_email()
        .times(1)
        .withf(
            move |created_by, gid, eid, sid, step, _template, subject, body| {
                *created_by == user_id
                    && *gid == group_id
                    && *eid == event_id
                    && *sid == submission_id
                    && step == "schedule"
                    && subject == expected_subject
                    && body == expected_body
            },
        )
        .returning(|_, _, _, _, _, _, _, _| Ok(()));

    let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
        .build()
        .await;
    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/dashboard/group/events/{event_id}/submissions/{submission_id}/prep-email"
        ))
        .header(COOKIE, format!("id={session_id}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("step=schedule"))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();

    assert_empty_response(&parts, &bytes, StatusCode::NO_CONTENT);
    assert_eq!(
        parts.headers.get("hx-trigger").and_then(|value| value.to_str().ok()),
        Some("refresh-event-submissions")
    );
}

#[tokio::test]
async fn test_send_prep_listed_email_requires_a_session() {
    let alliance_id = Uuid::new_v4();
    let group_id = Uuid::new_v4();
    let event_id = Uuid::new_v4();
    let submission_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(
        session_id,
        user_id,
        &auth_hash,
        Some(alliance_id),
        Some(group_id),
    );
    let event = sample_event_summary(event_id, group_id);
    let context = crate::templates::dashboard::group::submissions::CfsSubmissionPrepEmailContext {
        speaker_user_id: Uuid::new_v4(),
        status_id: "approved".to_string(),
        title: "Opening keynote".to_string(),
        co_speaker_user_id: None,
        linked_session_id: None,
        prep_emails: vec![],
        session_starts_at: None,
        tracks: vec![],
        venue_name: None,
    };

    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_user_has_group_permission()
        .times(1)
        .withf(move |cid, gid, uid, permission| {
            *cid == alliance_id
                && *gid == group_id
                && *uid == user_id
                && permission == GroupPermission::EventsWrite
        })
        .returning(|_, _, _, _| Ok(true));
    db.expect_get_site_settings()
        .times(1)
        .returning(|| Ok(sample_site_settings()));
    db.expect_get_event_summary()
        .times(1)
        .returning(move |_, _, _| Ok(event.clone()));
    db.expect_get_cfs_submission_prep_email_context()
        .times(1)
        .returning(move |_, _, _| Ok(Some(context.clone())));

    let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
        .build()
        .await;
    let request = Request::builder()
        .method("POST")
        .uri(format!(
            "/dashboard/group/events/{event_id}/submissions/{submission_id}/prep-email"
        ))
        .header(COOKIE, format!("id={session_id}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("step=listed"))
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();

    assert_eq!(parts.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        String::from_utf8(bytes.to_vec()).unwrap(),
        "Add this talk to the agenda before emailing that it is listed."
    );
}
