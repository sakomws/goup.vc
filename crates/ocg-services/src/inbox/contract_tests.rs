//! Real-database contracts for the inbox manager: transactional compositions
//! that mocks cannot prove, run by `just db-contract-tests`.

use std::{env, sync::Arc};

use anyhow::{Context, Result};
use deadpool_postgres::{Config as DbConfig, Pool, Runtime};
use tokio_postgres::NoTls;
use uuid::Uuid;

use crate::inbox::{InboxManager, PgInboxManager, StartConversationInput};
use ocg_db::PgDB;

/// Inactive community holding every inbox contract fixture.
const INBOX_COMMUNITY_ID: Uuid = Uuid::from_u128(0xe001);
/// Group team member with the admin role in the write group.
const INBOX_ADMIN_ID: Uuid = Uuid::from_u128(0xe021);
/// Group team member with the events-manager role in the write group.
const INBOX_EVENTS_MANAGER_ID: Uuid = Uuid::from_u128(0xe022);
/// User starting a conversation through the manager.
const INBOX_MANAGER_USER_ID: Uuid = Uuid::from_u128(0xe02a);
/// Published event owned by the write group.
const INBOX_WRITE_EVENT_ID: Uuid = Uuid::from_u128(0xe032);

#[tokio::test]
#[ignore = "requires the contract test database"]
async fn db_contracts_inbox_manager_commits_message_with_notification() -> Result<()> {
    // Setup the manager over the contract database
    seed_inbox_manager_fixture().await?;
    let manager = PgInboxManager::new(Arc::new(contract_tests_db()?));

    // Start a conversation through the manager
    let inbox_conversation_id = manager
        .start_conversation(&StartConversationInput {
            body: "Can I bring a friend?".to_string(),
            community_id: INBOX_COMMUNITY_ID,
            event_id: INBOX_WRITE_EVENT_ID,
            user_id: INBOX_MANAGER_USER_ID,
        })
        .await?;

    // Load the committed message and notification rows
    let client = contract_tests_pool()?.get().await?;
    let body: String = client
        .query_one(
            "select body from inbox_message where inbox_conversation_id = $1::uuid",
            &[&inbox_conversation_id],
        )
        .await?
        .get(0);
    let recipients: Vec<Uuid> = client
        .query(
            "select n.user_id
            from notification n
            join notification_template_data d using (notification_template_data_id)
            where n.kind = 'inbox-message-received'
            and d.data->>'sender_name' = 'Inbox Manager'
            order by n.user_id",
            &[],
        )
        .await?
        .iter()
        .map(|row| row.get(0))
        .collect();

    // Check the message and its emails were committed together
    assert_eq!(body, "Can I bring a friend?");
    assert_eq!(recipients, vec![INBOX_ADMIN_ID, INBOX_EVENTS_MANAGER_ID]);

    Ok(())
}

fn contract_tests_pool() -> Result<Pool> {
    let port = env::var("OCG_DB_PORT")
        .unwrap_or_else(|_| "5432".to_string())
        .parse()
        .context("OCG_DB_PORT must be a valid port number")?;
    let mut cfg = DbConfig::new();
    cfg.dbname = Some(
        env::var("OCG_DB_NAME_TESTS_CONTRACT").unwrap_or_else(|_| "ocg_tests_contract".to_string()),
    );
    cfg.host = Some(env::var("OCG_DB_HOST").unwrap_or_else(|_| "localhost".to_string()));
    cfg.port = Some(port);
    cfg.user = Some(env::var("OCG_DB_USER").unwrap_or_else(|_| "postgres".to_string()));
    if let Ok(password) = env::var("OCG_DB_PASSWORD")
        && !password.is_empty()
    {
        cfg.password = Some(password);
    }
    Ok(cfg.create_pool(Some(Runtime::Tokio1), NoTls)?)
}

fn contract_tests_db() -> Result<PgDB> {
    Ok(PgDB::new(contract_tests_pool()?))
}

async fn seed_inbox_manager_fixture() -> Result<()> {
    let client = contract_tests_pool()?.get().await?;
    client
        .batch_execute(
            "
            insert into alliance (
                active,
                alliance_id,
                banner_mobile_url,
                banner_url,
                description,
                display_name,
                logo_url,
                name
            ) values (
                false,
                '00000000-0000-0000-0000-00000000e001',
                'https://example.com/inbox-alliance-banner-mobile.png',
                'https://example.com/inbox-alliance-banner.png',
                'An inbox alliance used by Rust database contract tests',
                'Contract Inbox Alliance',
                'https://example.com/inbox-alliance-logo.png',
                'contract-inbox-alliance'
            ) on conflict do nothing;

            insert into group_category (alliance_id, group_category_id, name)
            values (
                '00000000-0000-0000-0000-00000000e001',
                '00000000-0000-0000-0000-00000000e002',
                'Inbox groups'
            ) on conflict do nothing;

            insert into event_category (alliance_id, event_category_id, name)
            values (
                '00000000-0000-0000-0000-00000000e001',
                '00000000-0000-0000-0000-00000000e003',
                'Inbox events'
            ) on conflict do nothing;

            insert into \"group\" (
                alliance_id,
                group_category_id,
                group_id,
                name,
                slug
            ) values (
                '00000000-0000-0000-0000-00000000e001',
                '00000000-0000-0000-0000-00000000e002',
                '00000000-0000-0000-0000-00000000e012',
                'Contract Inbox Write Group',
                'contract-inbox-write-group'
            ) on conflict do nothing;

            insert into \"user\" (
                auth_hash,
                email,
                email_verified,
                name,
                user_id,
                username
            ) values
                (
                    'contract_hash_inbox_admin',
                    'inbox.admin.contract@example.com',
                    true,
                    'Inbox Admin',
                    '00000000-0000-0000-0000-00000000e021',
                    'contract-inbox-admin'
                ),
                (
                    'contract_hash_inbox_events_manager',
                    'inbox.events-manager.contract@example.com',
                    true,
                    'Inbox Events Manager',
                    '00000000-0000-0000-0000-00000000e022',
                    'contract-inbox-events-manager'
                ),
                (
                    'contract_hash_inbox_manager',
                    'inbox.manager.contract@example.com',
                    true,
                    'Inbox Manager',
                    '00000000-0000-0000-0000-00000000e02a',
                    'contract-inbox-manager'
                )
            on conflict do nothing;

            insert into group_team (accepted, group_id, role, user_id)
            values
                (
                    true,
                    '00000000-0000-0000-0000-00000000e012',
                    'admin',
                    '00000000-0000-0000-0000-00000000e021'
                ),
                (
                    true,
                    '00000000-0000-0000-0000-00000000e012',
                    'events-manager',
                    '00000000-0000-0000-0000-00000000e022'
                )
            on conflict do nothing;

            insert into event (
                description,
                event_category_id,
                event_id,
                event_kind_id,
                group_id,
                name,
                published,
                published_at,
                slug,
                starts_at,
                timezone
            ) values (
                'Event contacted by the inbox manager contract',
                '00000000-0000-0000-0000-00000000e003',
                '00000000-0000-0000-0000-00000000e032',
                'virtual',
                '00000000-0000-0000-0000-00000000e012',
                'Contract Inbox Write Event',
                true,
                '2024-01-01 00:00:00+00',
                'contract-inbox-write-event',
                '2099-01-10 10:00:00+00',
                'UTC'
            ) on conflict do nothing;
            ",
        )
        .await?;
    Ok(())
}
