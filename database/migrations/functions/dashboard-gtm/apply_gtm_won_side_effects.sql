create or replace function apply_gtm_won_side_effects(
    p_actor_user_id uuid,
    p_gtm_lead_id uuid,
    p_effects jsonb
) returns jsonb language plpgsql as $$
declare
    v_lead gtm_lead;
    v_result jsonb := '{}'::jsonb;
    v_sponsor_id uuid;
    v_entry_id uuid;
    v_logo_url text;
    v_summary text;
begin
    select * into v_lead from gtm_lead where gtm_lead_id = p_gtm_lead_id;
    if not found then
        raise exception 'gtm lead not found';
    end if;

    if coalesce((p_effects->>'create_sponsor')::boolean, false)
       and v_lead.kind = 'sponsor'
       and v_lead.group_id is not null
       and v_lead.group_sponsor_id is null
    then
        v_logo_url := coalesce(
            nullif(v_lead.payload->>'logo_url', ''),
            'https://www.google.com/s2/favicons?sz=128&domain='
                || coalesce(nullif(v_lead.website_url, ''), 'goup.vc')
        );
        v_sponsor_id := add_group_sponsor(
            p_actor_user_id,
            v_lead.group_id,
            jsonb_build_object(
                'name', coalesce(v_lead.org_name, v_lead.name),
                'logo_url', v_logo_url,
                'website_url', v_lead.website_url,
                'featured', true
            )
        );
        update gtm_lead
        set group_sponsor_id = v_sponsor_id, updated_at = current_timestamp
        where gtm_lead_id = p_gtm_lead_id;
        v_result := v_result || jsonb_build_object('group_sponsor_id', v_sponsor_id);
    end if;

    if coalesce((p_effects->>'create_landscape_entry')::boolean, false)
       and v_lead.kind in ('startup', 'investor')
       and v_lead.landscape_entry_id is null
    then
        v_summary := coalesce(
            nullif(v_lead.notes, ''),
            coalesce(v_lead.org_name, v_lead.name) || ' added from GTM'
        );
        v_entry_id := add_landscape_entry(
            p_actor_user_id,
            v_lead.alliance_id,
            jsonb_build_object(
                'name', coalesce(v_lead.org_name, v_lead.name),
                'kind', v_lead.kind,
                'summary', left(v_summary, 500),
                'website_url', v_lead.website_url
            ),
            '{}'::text[],
            '{}'::text[]
        );
        update gtm_lead
        set landscape_entry_id = v_entry_id, updated_at = current_timestamp
        where gtm_lead_id = p_gtm_lead_id;
        v_result := v_result || jsonb_build_object('landscape_entry_id', v_entry_id);
    end if;

    if coalesce((p_effects->>'invite_organizer')::boolean, false)
       and v_lead.kind = 'organizer'
       and v_lead.user_id is not null
       and v_lead.group_id is not null
    then
        begin
            perform add_group_team_member(
                p_actor_user_id,
                v_lead.group_id,
                v_lead.user_id,
                'viewer'
            );
            v_result := v_result || jsonb_build_object('organizer_invited', true);
        exception
            when others then
                v_result := v_result || jsonb_build_object(
                    'organizer_invited', false,
                    'organizer_invite_error', sqlerrm
                );
        end;
    end if;

    if v_result <> '{}'::jsonb then
        perform add_gtm_lead_activity(
            p_actor_user_id,
            p_gtm_lead_id,
            null,
            'side_effect',
            'Applied won side effects',
            v_result
        );
    end if;

    return v_result;
end;
$$;
