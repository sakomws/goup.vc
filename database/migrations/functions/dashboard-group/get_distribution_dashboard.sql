create or replace function get_distribution_dashboard(p_group_id uuid)
returns jsonb as $$
declare
    v_registrations jsonb := '{}'::jsonb;
    v_link_registrations jsonb := '{}'::jsonb;
    v_partner_registrations jsonb := '{}'::jsonb;
    v_channel_registrations jsonb := '{}'::jsonb;
begin
    -- event_registration_attribution is supplied by an independent migration.
    -- Dynamic SQL keeps 0125 valid when that migration is not installed.
    if to_regclass('public.event_registration_attribution') is not null then
        execute $query$
            select coalesce(jsonb_object_agg(distribution_campaign_id, registrations), '{}'::jsonb)
            from (
                select c.distribution_campaign_id, count(distinct a.user_id) registrations
                from distribution_campaign c
                join event_registration_attribution a on a.event_id = c.event_id
                left join distribution_link l on l.distribution_campaign_id = c.distribution_campaign_id
                left join distribution_partner p on p.distribution_partner_id = l.distribution_partner_id
                where c.group_id = $1
                and (
                    lower(a.utm_campaign) = lower(l.utm_campaign)
                    or lower(a.referral_code) = lower(p.referral_code)
                )
                group by c.distribution_campaign_id
            ) totals
        $query$ into v_registrations using p_group_id;
        execute $query$
            select coalesce(jsonb_object_agg(distribution_link_id, registrations), '{}'::jsonb)
            from (
                select l.distribution_link_id, count(distinct a.user_id) registrations
                from distribution_link l
                join distribution_campaign c using (distribution_campaign_id)
                join event_registration_attribution a on a.event_id = c.event_id
                left join distribution_partner p using (distribution_partner_id)
                where c.group_id = $1 and (
                    (
                        lower(a.utm_campaign) = lower(l.utm_campaign)
                        and lower(a.utm_source) = lower(l.utm_source)
                        and lower(a.utm_medium) = lower(l.utm_medium)
                        and (l.utm_content is null or lower(a.utm_content) = lower(l.utm_content))
                    )
                    or lower(a.referral_code) = lower(p.referral_code)
                )
                group by l.distribution_link_id
            ) totals
        $query$ into v_link_registrations using p_group_id;
        execute $query$
            select coalesce(jsonb_object_agg(distribution_partner_id, registrations), '{}'::jsonb)
            from (
                select p.distribution_partner_id, count(distinct a.user_id) registrations
                from distribution_partner p
                join distribution_campaign c on c.group_id = p.group_id
                join event_registration_attribution a on a.event_id = c.event_id
                    and lower(a.referral_code) = lower(p.referral_code)
                where p.group_id = $1
                group by p.distribution_partner_id
            ) totals
        $query$ into v_partner_registrations using p_group_id;
        execute $query$
            select coalesce(jsonb_object_agg(channel, registrations), '{}'::jsonb)
            from (
                select l.channel, count(distinct a.user_id) registrations
                from distribution_link l
                join distribution_campaign c using (distribution_campaign_id)
                join event_registration_attribution a on a.event_id = c.event_id
                    and lower(a.utm_campaign) = lower(l.utm_campaign)
                    and lower(a.utm_source) = lower(l.utm_source)
                where c.group_id = $1
                group by l.channel
            ) totals
        $query$ into v_channel_registrations using p_group_id;
    end if;

    return jsonb_build_object(
        'events', (
            select coalesce(
                jsonb_agg(jsonb_build_object('event_id', event_id, 'name', name) order by starts_at desc nulls last),
                '[]'::jsonb
            )
            from event
            where group_id = p_group_id and deleted = false
        ),
        'campaigns', (
            select coalesce(jsonb_agg(jsonb_build_object(
                'distribution_campaign_id', c.distribution_campaign_id,
                'event_id', c.event_id,
                'name', c.name,
                'status', c.status,
                'starts_at', extract(epoch from c.starts_at)::bigint,
                'ends_at', extract(epoch from c.ends_at)::bigint,
                'clicks', coalesce(m.clicks, 0),
                'registrations', coalesce((v_registrations->>c.distribution_campaign_id::text)::bigint, 0)
            ) order by c.created_at desc), '[]'::jsonb)
            from distribution_campaign c
            left join lateral (
                select count(*) clicks
                from distribution_link l
                join distribution_link_click_daily d using (distribution_link_id)
                where l.distribution_campaign_id = c.distribution_campaign_id
            ) m on true
            where c.group_id = p_group_id
        ),
        'partners', (
            select coalesce(jsonb_agg(jsonb_build_object(
                'distribution_partner_id', p.distribution_partner_id,
                'name', p.name, 'referral_code', p.referral_code, 'active', p.active,
                'clicks', coalesce(m.clicks, 0),
                'registrations', coalesce((v_partner_registrations->>p.distribution_partner_id::text)::bigint, 0)
            ) order by p.name), '[]'::jsonb)
            from distribution_partner p
            left join lateral (
                select count(*) clicks from distribution_link l
                join distribution_link_click_daily d using (distribution_link_id)
                where l.distribution_partner_id = p.distribution_partner_id
            ) m on true
            where p.group_id = p_group_id
        ),
        'links', (
            select coalesce(jsonb_agg(jsonb_build_object(
                'distribution_link_id', l.distribution_link_id,
                'distribution_campaign_id', l.distribution_campaign_id,
                'distribution_partner_id', l.distribution_partner_id,
                'code', l.code, 'channel', l.channel, 'target_url', l.target_url,
                'utm_source', l.utm_source, 'utm_medium', l.utm_medium,
                'utm_campaign', l.utm_campaign, 'utm_content', l.utm_content,
                'active', l.active, 'clicks', coalesce(m.clicks, 0),
                'registrations', coalesce((v_link_registrations->>l.distribution_link_id::text)::bigint, 0)
            ) order by l.created_at desc), '[]'::jsonb)
            from distribution_link l
            join distribution_campaign c using (distribution_campaign_id)
            left join lateral (
                select count(*) clicks from distribution_link_click_daily d
                where d.distribution_link_id = l.distribution_link_id
            ) m on true
            where c.group_id = p_group_id
        ),
        'content', (
            select coalesce(jsonb_agg(jsonb_build_object(
                'distribution_content_id', d.distribution_content_id,
                'distribution_campaign_id', d.distribution_campaign_id,
                'channel', d.channel, 'state', d.state, 'title', d.title,
                'caption', d.caption, 'cta', d.cta, 'hashtags', d.hashtags,
                'event_image_reference', d.event_image_reference,
                'scheduled_for', extract(epoch from d.scheduled_for)::bigint,
                'remind_when_due', d.remind_when_due,
                'posted_at', extract(epoch from d.posted_at)::bigint
            ) order by d.scheduled_for nulls last, d.created_at desc), '[]'::jsonb)
            from distribution_content d
            join distribution_campaign c using (distribution_campaign_id)
            where c.group_id = p_group_id
        ),
        'library', (
            select coalesce(jsonb_agg(jsonb_build_object(
                'distribution_library_item_id', distribution_library_item_id,
                'kind', kind, 'name', name, 'value', value
            ) order by kind, name), '[]'::jsonb)
            from distribution_library_item where group_id = p_group_id
        ),
        'channel_metrics', (
            select coalesce(
                jsonb_agg(jsonb_build_object(
                    'channel', channel,
                    'clicks', clicks,
                    'registrations', coalesce((v_channel_registrations->>channel)::bigint, 0)
                ) order by channel),
                '[]'::jsonb
            )
            from (
                select l.channel, count(d.*) clicks
                from distribution_link l
                join distribution_campaign c using (distribution_campaign_id)
                left join distribution_link_click_daily d using (distribution_link_id)
                where c.group_id = p_group_id group by l.channel
            ) channels
        )
    );
end;
$$ language plpgsql stable;
