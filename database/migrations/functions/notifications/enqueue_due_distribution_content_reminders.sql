create or replace function enqueue_due_distribution_content_reminders(p_base_url text)
returns integer as $$
declare
    v_content record;
    v_recipients uuid[];
    v_count integer := 0;
begin
    if not pg_try_advisory_xact_lock(hashtextextended('ocg:distribution-content-due', 0)) then
        return 0;
    end if;

    for v_content in
        select d.*, c.group_id, g.name as group_name
        from distribution_content d
        join distribution_campaign c using (distribution_campaign_id)
        join "group" g using (group_id)
        where d.state = 'ready'
        and d.remind_when_due = true
        and d.reminder_sent_for is null
        and d.scheduled_for <= current_timestamp
        order by d.scheduled_for, d.distribution_content_id
        for update of d skip locked
    loop
        select coalesce(array_agg(gt.user_id order by gt.user_id), '{}')
        into v_recipients
        from group_team gt
        join group_role_group_permission rp on rp.group_role_id = gt.role
        where gt.group_id = v_content.group_id
        and gt.accepted = true
        and rp.group_permission_id = 'group.distribution.write';

        if cardinality(v_recipients) > 0 then
            perform enqueue_notification(
                'distribution-content-due',
                jsonb_build_object(
                    'channel', v_content.channel,
                    'content_id', v_content.distribution_content_id,
                    'group_name', v_content.group_name,
                    'scheduled_for', v_content.scheduled_for,
                    'title', v_content.title,
                    'link', regexp_replace(p_base_url, '/+$', '') || '/dashboard/group?tab=distribution'
                ),
                '[]'::jsonb,
                v_recipients
            );
            v_count := v_count + cardinality(v_recipients);
        end if;

        update distribution_content
        set reminder_sent_for = scheduled_for, updated_at = current_timestamp
        where distribution_content_id = v_content.distribution_content_id;
    end loop;
    return v_count;
end;
$$ language plpgsql;
