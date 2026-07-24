use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::COOKIE},
};
use axum_login::tower_sessions::session;
use tower::ServiceExt;
use uuid::Uuid;

use crate::{
    db::mock::MockDB, handlers::tests::*, services::notifications::MockNotificationsManager,
};

#[tokio::test]
async fn deleting_a_job_refreshes_the_dashboard_body() {
    let user_id = Uuid::new_v4();
    let job_id = Uuid::new_v4();
    let session_id = session::Id::default();
    let auth_hash = "hash".to_owned();
    let session_record = sample_session_record(session_id, user_id, &auth_hash, None, None);

    let mut db = MockDB::new();
    db.expect_get_session()
        .times(1)
        .withf(move |id| *id == session_id)
        .returning(move |_| Ok(Some(session_record.clone())));
    db.expect_get_user_by_id()
        .times(1)
        .withf(move |id| *id == user_id)
        .returning(move |_| Ok(Some(sample_auth_user(user_id, &auth_hash))));
    db.expect_delete_job()
        .times(1)
        .withf(move |owner_id, deleted_job_id| *owner_id == user_id && *deleted_job_id == job_id)
        .returning(|_, _| Ok(()));

    let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
        .build()
        .await;
    let request = Request::builder()
        .method("DELETE")
        .uri(format!("/dashboard/jobs/{job_id}"))
        .header(COOKIE, format!("id={session_id}"))
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    let body = to_bytes(body, usize::MAX).await.unwrap();

    assert_empty_hx_trigger_response(&parts, &body, StatusCode::NO_CONTENT, "refresh-body");
}
