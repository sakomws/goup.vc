use axum::{
    body::Body,
    http::{
        Request, StatusCode,
        header::{CONTENT_TYPE, COOKIE},
    },
};
use axum_login::tower_sessions::session;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    db::mock::MockDB,
    handlers::tests::*,
    services::notifications::MockNotificationsManager,
    types::{gtm::GtmReviewDraftResult, permissions::AlliancePermission},
};

#[tokio::test]
async fn test_add_success() {
    let alliance_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let lead_id = Uuid::new_v4();

    let mut db = MockDB::new();
    expect_authenticated_alliance_session(&mut db, session_id, user_id, alliance_id);
    expect_alliance_permission(&mut db, alliance_id, user_id, AlliancePermission::GtmWrite);
    db.expect_add_gtm_lead()
        .times(1)
        .withf(move |actor_user_id, id, input| {
            *actor_user_id == user_id
                && *id == alliance_id
                && input.name == "Ada Example"
                && input.kind == "sponsor"
        })
        .returning(move |_, _, _| Ok(lead_id));

    let router =
        Box::pin(TestRouterBuilder::new(db, MockNotificationsManager::new()).build()).await;
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/dashboard/alliance/gtm/add")
                .header(COOKIE, format!("id={session_id}"))
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("name=Ada+Example&kind=sponsor"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_delete_success() {
    let alliance_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let lead_id = Uuid::new_v4();

    let mut db = MockDB::new();
    expect_authenticated_alliance_session(&mut db, session_id, user_id, alliance_id);
    expect_alliance_permission(&mut db, alliance_id, user_id, AlliancePermission::GtmWrite);
    db.expect_delete_gtm_lead()
        .times(1)
        .withf(move |actor_user_id, id, id_lead, group_id| {
            *actor_user_id == user_id
                && *id == alliance_id
                && *id_lead == lead_id
                && group_id.is_none()
        })
        .returning(|_, _, _, _| Ok(()));

    let router =
        Box::pin(TestRouterBuilder::new(db, MockNotificationsManager::new()).build()).await;
    let response = router
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/dashboard/alliance/gtm/{lead_id}"))
                .header(COOKIE, format!("id={session_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_review_draft_approves_without_sending_when_not_outreach() {
    let alliance_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let draft_id = Uuid::new_v4();
    let lead_id = Uuid::new_v4();

    let mut db = MockDB::new();
    expect_authenticated_alliance_session(&mut db, session_id, user_id, alliance_id);
    expect_alliance_permission(&mut db, alliance_id, user_id, AlliancePermission::GtmWrite);
    db.expect_review_gtm_agent_draft()
        .times(1)
        .withf(move |actor_user_id, id, id_draft, input| {
            *actor_user_id == user_id
                && *id == alliance_id
                && *id_draft == draft_id
                && input["status"] == "approved"
        })
        .returning(move |_, _, _, _| {
            Ok(GtmReviewDraftResult {
                gtm_agent_draft_id: draft_id,
                status: "approved".into(),
                created_lead_ids: vec![],
                side_effects: serde_json::json!({}),
                gtm_lead_id: Some(lead_id),
                agent_id: Some("qualification".into()),
                body: Some("Qualified".into()),
                suggested_stage: Some("qualification".into()),
            })
        });

    let router =
        Box::pin(TestRouterBuilder::new(db, MockNotificationsManager::new()).build()).await;
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/dashboard/alliance/gtm/drafts/{draft_id}/review"))
                .header(COOKIE, format!("id={session_id}"))
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("status=approved"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_transition_rejects_invalid_stage() {
    let alliance_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let lead_id = Uuid::new_v4();

    let mut db = MockDB::new();
    expect_authenticated_alliance_session(&mut db, session_id, user_id, alliance_id);
    expect_alliance_permission(&mut db, alliance_id, user_id, AlliancePermission::GtmWrite);

    let router =
        Box::pin(TestRouterBuilder::new(db, MockNotificationsManager::new()).build()).await;
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/dashboard/alliance/gtm/{lead_id}/transition"))
                .header(COOKIE, format!("id={session_id}"))
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from("stage=not-a-stage"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
