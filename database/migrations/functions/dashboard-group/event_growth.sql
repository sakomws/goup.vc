-- Capture the first successful registration touch without changing existing registration APIs.
create or replace function capture_event_registration_attribution(
    p_event_id uuid,
    p_user_id uuid,
    p_attribution jsonb default '{}'::jsonb
) returns void as $$
begin
    insert into event_registration_attribution (
        event_id,
        user_id,
        source,
        referral_code,
        referrer,
        utm_source,
        utm_medium,
        utm_campaign,
        utm_content,
        utm_term
    )
    select
        p_event_id,
        p_user_id,
        left(nullif(btrim(p_attribution->>'source'), ''), 255),
        left(nullif(btrim(p_attribution->>'referral_code'), ''), 255),
        left(nullif(btrim(p_attribution->>'referrer'), ''), 2048),
        left(nullif(btrim(p_attribution->>'utm_source'), ''), 255),
        left(nullif(btrim(p_attribution->>'utm_medium'), ''), 255),
        left(nullif(btrim(p_attribution->>'utm_campaign'), ''), 255),
        left(nullif(btrim(p_attribution->>'utm_content'), ''), 255),
        left(nullif(btrim(p_attribution->>'utm_term'), ''), 255)
    where exists (
        select 1 from event where event_id = p_event_id and deleted = false
    )
    on conflict (event_id, user_id) do nothing;
end;
$$ language plpgsql;

-- Organizer-facing event funnel, attribution, collaboration, and finance rollup.
create or replace function get_event_growth(
    p_group_id uuid,
    p_event_id uuid
) returns jsonb as $$
declare
    v_result jsonb;
begin
    if not exists (
        select 1 from event
        where event_id = p_event_id
        and group_id = p_group_id
        and deleted = false
    ) then
        raise exception 'event not found';
    end if;

    with target as (
        select event_id, group_id, starts_at, created_at, payment_currency_code
        from event
        where event_id = p_event_id and group_id = p_group_id
    ),
    registrations as (
        select era.*
        from event_registration_attribution era
        where era.event_id = p_event_id
    ),
    registration_users as (
        select user_id from registrations
        union
        select user_id from event_attendee where event_id = p_event_id
        union
        select user_id from event_waitlist where event_id = p_event_id
        union
        select user_id from event_invitation_request where event_id = p_event_id
        union
        select user_id from event_purchase where event_id = p_event_id
    ),
    confirmed as (
        select ea.user_id, ea.created_at, ea.checked_in
        from event_attendee ea
        where ea.event_id = p_event_id and ea.status = 'confirmed'
    ),
    attendee_segments as (
        select
            c.user_id,
            exists (
                select 1
                from event_attendee previous_attendee
                join event previous_event using (event_id)
                where previous_attendee.user_id = c.user_id
                and previous_attendee.status = 'confirmed'
                and previous_event.group_id = p_group_id
                and previous_event.event_id <> p_event_id
                and previous_attendee.created_at < c.created_at
            ) as repeat_attendee
        from confirmed c
    ),
    funnel as (
        select jsonb_build_object(
            'views', coalesce((select sum(total) from event_views where event_id = p_event_id), 0),
            'registrations', (select count(*) from registration_users),
            'waitlisted', (select count(*) from event_waitlist where event_id = p_event_id),
            'pending', (
                select count(distinct user_id) from (
                    select user_id from event_invitation_request
                    where event_id = p_event_id and status = 'pending'
                    union all
                    select user_id from event_attendee
                    where event_id = p_event_id
                    and status in ('invitation-pending', 'registration-questions-pending')
                    union all
                    select user_id from event_purchase
                    where event_id = p_event_id
                    and status = 'pending'
                    and (hold_expires_at is null or hold_expires_at > current_timestamp)
                ) pending_users
            ),
            'confirmed', (select count(*) from confirmed),
            'check_ins', (select count(*) from confirmed where checked_in),
            'conversion_rate', case
                when coalesce((select sum(total) from event_views where event_id = p_event_id), 0) = 0
                then 0
                else round(
                    (select count(*) from confirmed)::numeric * 100
                    / (select sum(total) from event_views where event_id = p_event_id),
                    2
                )
            end,
            'unique_attendees', (select count(*) from attendee_segments where not repeat_attendee),
            'repeat_attendees', (select count(*) from attendee_segments where repeat_attendee),
            'new_members', (
                select count(distinct c.user_id)
                from confirmed c
                join group_member gm on gm.group_id = p_group_id and gm.user_id = c.user_id
                where gm.created_at >= coalesce(
                    (select r.captured_at from registrations r where r.user_id = c.user_id),
                    c.created_at
                )
            ),
            'follow_up_collaborators', (
                select count(distinct c.user_id)
                from confirmed c
                join event_attendee later_attendee
                    on later_attendee.user_id = c.user_id
                    and later_attendee.status = 'confirmed'
                join event later_event
                    on later_event.event_id = later_attendee.event_id
                    and later_event.group_id = p_group_id
                    and later_event.event_id <> p_event_id
                cross join target t
                where coalesce(later_event.starts_at, later_event.created_at)
                    > coalesce(t.starts_at, t.created_at)
            )
        )
    ),
        source_breakdown as (
            select coalesce(utm_source, source, 'direct') as label, count(*) as total
            from registrations
            group by coalesce(utm_source, source, 'direct')
        ),
        referral_breakdown as (
            select referral_code as label, count(*) as total
            from registrations
            where referral_code is not null
            group by referral_code
        ),
        purchase_totals as (
            select
                currency_code,
                sum(amount_minor) filter (where status in ('completed', 'refunded')) as gross_minor,
                sum(amount_minor) filter (where status = 'refunded') as refunds_minor
            from event_purchase
            where event_id = p_event_id
            and status in ('completed', 'refunded')
            group by currency_code
        ),
        manual_totals as (
            select
                currency_code,
                sum(amount_minor) filter (where kind = 'income') as income_minor,
                sum(amount_minor) filter (where kind = 'expense') as expense_minor
            from event_finance_entry
            where event_id = p_event_id
            group by currency_code
        ),
        currencies as (
            select currency_code from purchase_totals
            union
            select currency_code from manual_totals
        )
    select jsonb_build_object(
        'funnel', (select * from funnel),
        'source_breakdown', coalesce((
            select jsonb_agg(jsonb_build_object('label', label, 'total', total) order by total desc, label)
            from source_breakdown
        ), '[]'::jsonb),
        'referral_breakdown', coalesce((
            select jsonb_agg(jsonb_build_object('label', label, 'total', total) order by total desc, label)
            from referral_breakdown
        ), '[]'::jsonb),
        'finances', coalesce((
            select jsonb_agg(jsonb_build_object(
                'currency_code', c.currency_code,
                'gross_purchases_minor', coalesce(p.gross_minor, 0),
                'refunds_minor', coalesce(p.refunds_minor, 0),
                'net_purchases_minor', coalesce(p.gross_minor, 0) - coalesce(p.refunds_minor, 0),
                'manual_income_minor', coalesce(m.income_minor, 0),
                'manual_expense_minor', coalesce(m.expense_minor, 0),
                'net_total_minor',
                    coalesce(p.gross_minor, 0) - coalesce(p.refunds_minor, 0)
                    + coalesce(m.income_minor, 0) - coalesce(m.expense_minor, 0)
            ) order by c.currency_code)
            from currencies c
            left join purchase_totals p using (currency_code)
            left join manual_totals m using (currency_code)
        ), '[]'::jsonb),
        'finance_entries', coalesce((
            select jsonb_agg(jsonb_build_object(
                'event_finance_entry_id', event_finance_entry_id,
                'kind', kind,
                'category', category,
                'description', description,
                'amount_minor', amount_minor,
                'currency_code', currency_code,
                'occurred_at', occurred_at
            ) order by occurred_at desc, created_at desc)
            from event_finance_entry
            where event_id = p_event_id
        ), '[]'::jsonb),
        'sponsor_report', get_event_sponsor_report(p_event_id)
    ) into v_result;

    return v_result;
end;
$$ language plpgsql stable;

create or replace function add_event_finance_entry(
    p_actor_user_id uuid,
    p_group_id uuid,
    p_event_id uuid,
    p_entry jsonb
) returns uuid as $$
declare
    v_entry_id uuid;
begin
    insert into event_finance_entry (
        event_id,
        created_by,
        kind,
        category,
        description,
        amount_minor,
        currency_code,
        occurred_at
    )
    select
        p_event_id,
        p_actor_user_id,
        p_entry->>'kind',
        btrim(p_entry->>'category'),
        nullif(btrim(p_entry->>'description'), ''),
        (p_entry->>'amount_minor')::bigint,
        upper(p_entry->>'currency_code'),
        coalesce((p_entry->>'occurred_at')::date, current_date)
    where exists (
        select 1 from event
        where event_id = p_event_id and group_id = p_group_id and deleted = false
    )
    returning event_finance_entry_id into v_entry_id;

    if v_entry_id is null then
        raise exception 'event not found';
    end if;
    return v_entry_id;
end;
$$ language plpgsql;

create or replace function delete_event_finance_entry(
    p_group_id uuid,
    p_event_id uuid,
    p_event_finance_entry_id uuid
) returns void as $$
begin
    delete from event_finance_entry efe
    using event e
    where efe.event_finance_entry_id = p_event_finance_entry_id
    and efe.event_id = p_event_id
    and e.event_id = efe.event_id
    and e.group_id = p_group_id;
end;
$$ language plpgsql;
