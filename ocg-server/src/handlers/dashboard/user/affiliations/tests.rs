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
    db::mock::MockDB, handlers::tests::*, services::notifications::MockNotificationsManager,
};

#[tokio::test]
async fn test_add_affiliation_success() {
    // Setup identifiers and data structures.
    let landscape_entry_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(session_id, user_id, &auth_hash, None, None);

    // Setup database mock.
    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_add_user_affiliation()
        .times(1)
        .withf(move |actor_user_id, affiliation| {
            *actor_user_id == user_id
                && affiliation.landscape_entry_id == landscape_entry_id
                && affiliation.role == "maintainer"
        })
        .returning(|_, _| Ok(()));

    // Setup router and send request.
    let nm = MockNotificationsManager::new();
    let router = TestRouterBuilder::new(db, nm).build().await;
    let form_data = format!("landscape_entry_id={landscape_entry_id}&role=maintainer");
    let request = Request::builder()
        .method("POST")
        .uri("/dashboard/user/affiliations")
        .header(COOKIE, format!("id={session_id}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    // Execute request and verify response.
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response.headers().get("HX-Trigger").unwrap(),
        "refresh-user-dashboard-content"
    );
}

#[tokio::test]
async fn test_add_affiliation_invalid_role() {
    // Setup identifiers and data structures.
    let landscape_entry_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(session_id, user_id, &auth_hash, None, None);

    // Setup database mock (add_user_affiliation must not be called).
    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));

    // Setup router and send request.
    let nm = MockNotificationsManager::new();
    let router = TestRouterBuilder::new(db, nm).build().await;
    let form_data = format!("landscape_entry_id={landscape_entry_id}&role=ceo");
    let request = Request::builder()
        .method("POST")
        .uri("/dashboard/user/affiliations")
        .header(COOKIE, format!("id={session_id}"))
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form_data))
        .unwrap();

    // Execute request and verify response.
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_delete_affiliation_success() {
    // Setup identifiers and data structures.
    let user_affiliation_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let user_id = Uuid::new_v4();
    let auth_hash = "hash".to_string();
    let session_record = sample_session_record(session_id, user_id, &auth_hash, None, None);

    // Setup database mock.
    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_delete_user_affiliation()
        .times(1)
        .withf(move |actor_user_id, requested_affiliation_id| {
            *actor_user_id == user_id && *requested_affiliation_id == user_affiliation_id
        })
        .returning(|_, _| Ok(()));

    // Setup router and send request.
    let nm = MockNotificationsManager::new();
    let router = TestRouterBuilder::new(db, nm).build().await;
    let request = Request::builder()
        .method("DELETE")
        .uri(format!(
            "/dashboard/user/affiliations/{user_affiliation_id}"
        ))
        .header(COOKIE, format!("id={session_id}"))
        .body(Body::empty())
        .unwrap();

    // Execute request and verify response.
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response.headers().get("HX-Trigger").unwrap(),
        "refresh-user-dashboard-content"
    );
}
