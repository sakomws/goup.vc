create or replace function resolve_distribution_link(p_code text, p_fingerprint_hash text)
returns jsonb as $$
declare
    v_link record;
begin
    select l.*, p.referral_code
    into v_link
    from distribution_link l
    join distribution_campaign c using (distribution_campaign_id)
    left join distribution_partner p using (distribution_partner_id)
    where lower(l.code) = lower(p_code)
    and l.active = true
    and c.status in ('draft', 'active');

    if not found then
        return null;
    end if;

    insert into distribution_link_click_daily (
        distribution_link_id, clicked_on, fingerprint_hash
    ) values (
        v_link.distribution_link_id, current_date, p_fingerprint_hash
    ) on conflict do nothing;

    return jsonb_strip_nulls(jsonb_build_object(
        'target_url', v_link.target_url,
        'utm_source', v_link.utm_source,
        'utm_medium', v_link.utm_medium,
        'utm_campaign', v_link.utm_campaign,
        'utm_content', v_link.utm_content,
        'referral_code', v_link.referral_code
    ));
end;
$$ language plpgsql volatile security definer
set search_path = public, pg_temp;
