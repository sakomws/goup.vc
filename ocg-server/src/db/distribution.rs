//! Database operations for group distribution planning.

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    db::PgExecutor,
    types::distribution::{
        CampaignInput, ContentInput, DistributionDashboard, LibraryInput, LinkInput, PartnerInput,
        ResolvedDistributionLink,
    },
};

#[async_trait]
pub(crate) trait DBDistribution {
    async fn get_distribution_dashboard(&self, group_id: Uuid) -> Result<DistributionDashboard>;
    async fn add_distribution_campaign(
        &self,
        actor: Uuid,
        group_id: Uuid,
        input: &CampaignInput,
    ) -> Result<Uuid>;
    async fn add_distribution_partner(&self, group_id: Uuid, input: &PartnerInput) -> Result<Uuid>;
    async fn add_distribution_link(&self, group_id: Uuid, input: &LinkInput) -> Result<Uuid>;
    async fn add_distribution_content(&self, group_id: Uuid, input: &ContentInput) -> Result<Uuid>;
    async fn add_distribution_library_item(
        &self,
        group_id: Uuid,
        input: &LibraryInput,
    ) -> Result<Uuid>;
    async fn update_distribution_content_state(
        &self,
        group_id: Uuid,
        content_id: Uuid,
        state: &str,
    ) -> Result<()>;
    async fn resolve_distribution_link(
        &self,
        code: &str,
        fingerprint_hash: &str,
    ) -> Result<Option<ResolvedDistributionLink>>;
}

#[async_trait]
impl<T> DBDistribution for T
where
    T: PgExecutor + Send + Sync,
{
    async fn get_distribution_dashboard(&self, group_id: Uuid) -> Result<DistributionDashboard> {
        self.fetch_json_one("select get_distribution_dashboard($1::uuid)", &[&group_id])
            .await
    }

    async fn add_distribution_campaign(
        &self,
        actor: Uuid,
        group_id: Uuid,
        input: &CampaignInput,
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "insert into distribution_campaign (group_id,event_id,created_by,name,status)
             values ($1,$2,$3,$4,$5) returning distribution_campaign_id",
            &[
                &group_id,
                &input.event_id,
                &actor,
                &input.name,
                &input.status,
            ],
        )
        .await
    }

    async fn add_distribution_partner(&self, group_id: Uuid, input: &PartnerInput) -> Result<Uuid> {
        self.fetch_scalar_one(
            "insert into distribution_partner (group_id,name,referral_code)
             values ($1,$2,$3) returning distribution_partner_id",
            &[&group_id, &input.name, &input.referral_code],
        )
        .await
    }

    async fn add_distribution_link(&self, group_id: Uuid, input: &LinkInput) -> Result<Uuid> {
        self.fetch_scalar_one(
            "insert into distribution_link (
                distribution_campaign_id,distribution_partner_id,code,channel,target_url,
                utm_source,utm_medium,utm_campaign,utm_content
             ) select $1,$2,$3,$4,$5,$6,$7,$8,$9
             where exists (
                select 1 from distribution_campaign
                where distribution_campaign_id=$1 and group_id=$10
             )
             and ($2::uuid is null or exists (
                select 1 from distribution_partner
                where distribution_partner_id=$2 and group_id=$10
             ))
             returning distribution_link_id",
            &[
                &input.distribution_campaign_id,
                &input.distribution_partner_id,
                &input.code,
                &input.channel,
                &input.target_url,
                &input.utm_source,
                &input.utm_medium,
                &input.utm_campaign,
                &input.utm_content,
                &group_id,
            ],
        )
        .await
    }

    async fn add_distribution_content(&self, group_id: Uuid, input: &ContentInput) -> Result<Uuid> {
        self.fetch_scalar_one(
            "insert into distribution_content (
                distribution_campaign_id,channel,state,title,caption,cta,hashtags,
                event_image_reference,scheduled_for,remind_when_due
             ) select $1,$2,$3,$4,$5,$6,$7,$8,
                $9::timestamp without time zone at time zone 'UTC',$10
             where exists (
                select 1 from distribution_campaign
                where distribution_campaign_id=$1 and group_id=$11
             ) returning distribution_content_id",
            &[
                &input.distribution_campaign_id,
                &input.channel,
                &input.state,
                &input.title,
                &input.caption,
                &input.cta,
                &input.hashtags,
                &input.event_image_reference,
                &input.scheduled_for,
                &input.remind_when_due,
                &group_id,
            ],
        )
        .await
    }

    async fn add_distribution_library_item(
        &self,
        group_id: Uuid,
        input: &LibraryInput,
    ) -> Result<Uuid> {
        self.fetch_scalar_one(
            "insert into distribution_library_item (group_id,kind,name,value)
             values ($1,$2,$3,$4) returning distribution_library_item_id",
            &[&group_id, &input.kind, &input.name, &input.value],
        )
        .await
    }

    async fn update_distribution_content_state(
        &self,
        group_id: Uuid,
        content_id: Uuid,
        state: &str,
    ) -> Result<()> {
        self.execute(
            "update distribution_content d set state=$3,
                posted_at=case when $3='posted_manual' then current_timestamp else null end,
                updated_at=current_timestamp
             from distribution_campaign c
             where d.distribution_campaign_id=c.distribution_campaign_id
             and c.group_id=$1 and d.distribution_content_id=$2",
            &[&group_id, &content_id, &state],
        )
        .await
    }

    async fn resolve_distribution_link(
        &self,
        code: &str,
        fingerprint_hash: &str,
    ) -> Result<Option<ResolvedDistributionLink>> {
        self.fetch_json_opt(
            "select resolve_distribution_link($1::text,$2::text)",
            &[&code, &fingerprint_hash],
        )
        .await
    }
}
