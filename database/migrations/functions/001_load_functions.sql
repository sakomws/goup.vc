{{ template "internal/json/jsonb_geography_point.sql" }} -- Dependency for payload location mappings and distance search filters
{{ template "internal/json/jsonb_text_array.sql" }} -- Dependency for payload text-array mappings
{{ template "internal/search/escape_ilike_pattern.sql" }} -- Dependency for inbox search filters
{{ template "internal/search/prefix_tsquery.sql" }} -- Dependency for inbox search filters
{{ template "internal/search/parse_search_filters.sql" }} -- Dependency for inbox searches
{{ template "internal/audit/insert_audit_log.sql" }} -- Dependency for inbox and custom-notification audit entries
{{ template "common/audit_log_resource_name.sql" }} -- Dependency for inbox and custom-notification audit entries
{{ template "internal/groups/user_is_group_team_member.sql" }} -- Dependency for inbox authorization
{{ template "internal/users/public_user_summary.sql" }} -- Dependency for inbox conversation payloads
{{ template "internal/json/epoch_seconds.sql" }} -- Dependency for inbox conversation payloads
{{ template "internal/inbox/is_inbox_event_public.sql" }} -- Dependency for inbox conversation payloads
{{ template "internal/inbox/inbox_conversation_json.sql" }} -- Dependency for inbox reads and searches
{{ template "internal/inbox/append_inbox_message.sql" }}
{{ template "internal/inbox/inbox_user_rate_limits.sql" }}
{{ template "internal/inbox/is_inbox_user_blocked.sql" }}
{{ template "internal/inbox/resolve_inbox_contact_event.sql" }}
{{ template "internal/inbox/search_inbox_conversations.sql" }}
{{ template "internal/enrollment/admission_offer_is_active.sql" }}
{{ template "internal/enrollment/event_purchase_holds_seat.sql" }}
{{ template "internal/enrollment/admission_offer_reserves_seat.sql" }}
{{ template "internal/enrollment/event_purchase_reserves_seat.sql" }}
{{ template "internal/events/event_effective_ends_at.sql" }}
{{ template "internal/ticketing/is_event_paid_capable.sql" }}
{{ template "internal/ticketing/get_event_ticket_type_allocated_seat_count.sql" }}

{{ template "notifications/alliance_contact_filter_values.sql" }}
{{ template "notifications/alliance_contact_group_labels.sql" }}
{{ template "notifications/alliance_contact_groups.sql" }}
{{ template "notifications/alliance_contact_team_seats.sql" }}
{{ template "notifications/describe_alliance_contact_filters.sql" }}
{{ template "notifications/list_user_notification_group_ids.sql" }}
{{ template "notifications/users_accepting_notification.sql" }}
{{ template "notifications/validate_alliance_contact_filters.sql" }}

{{ template "auth/get_user_by_id.sql" }} -- Do not sort alphabetically, has dependency
{{ template "internal/users/resolve_unique_username.sql" }} -- Dependency for signup and pre-registration activation
{{ template "auth/activate_pre_registered_user_email_password.sql" }}
{{ template "auth/activate_pre_registered_user_external_provider.sql" }}
{{ template "auth/add_coffee_meet_request.sql" }}
{{ template "auth/add_mentorship_request.sql" }}
{{ template "auth/get_user_by_email.sql" }}
{{ template "auth/get_user_by_email_for_external_auth.sql" }}
{{ template "auth/get_user_by_id_verified.sql" }}
{{ template "auth/get_user_by_username.sql" }}
{{ template "auth/get_public_user_provider.sql" }}
{{ template "auth/is_linkedin_subject_blocked.sql" }}
{{ template "auth/sign_up_user.sql" }}
{{ template "auth/update_user_details.sql" }}
{{ template "auth/update_user_external_profile.sql" }}
{{ template "auth/update_user_password.sql" }}
{{ template "auth/update_user_provider.sql" }}
{{ template "auth/user_has_alliance_permission.sql" }}
{{ template "auth/user_has_group_permission.sql" }}
{{ template "auth/verify_email.sql" }}

{{ template "common/custom_domain_json.sql" }}
{{ template "common/collaboration_project_json.sql" }}
{{ template "internal/text/generate_slug.sql" }}
{{ template "internal/text/generate_slug_from_source.sql" }}
{{ template "common/get_alliance_full.sql" }}
{{ template "common/get_alliance_summary.sql" }} -- Do not sort alphabetically, has dependency
{{ template "internal/enrollment/get_event_occupied_seat_count.sql" }} -- Dependency for event capacity counts
{{ template "common/registration_window_closes_at.sql" }} -- Dependency for is_registration_window_open
{{ template "common/is_registration_window_open.sql" }} -- Dependency for attendee registration flows
{{ template "internal/ticketing/list_event_discount_codes.sql" }} -- Dependency for get_event_full and payments
{{ template "internal/ticketing/list_event_ticket_types.sql" }} -- Dependency for get_event_full and payments
{{ template "common/get_group_summary.sql" }} -- Do not sort alphabetically, has dependency
{{ template "common/get_collaboration_outcome_summary.sql" }}
{{ template "internal/enrollment/questionnaire_answers_exist_for_event.sql" }} -- Do not sort alphabetically, dependency for get_event_full and update_event
{{ template "common/query_community_analytics.sql" }}
{{ template "common/stats_day_count_series.sql" }}
{{ template "common/stats_event_audience.sql" }}
{{ template "common/stats_event_check_ins.sql" }}
{{ template "common/stats_event_enrollment.sql" }}
{{ template "common/stats_event_traffic.sql" }}
{{ template "common/stats_event_utm.sql" }}
{{ template "common/stats_event_tickets.sql" }} -- Depends on get_event_ticket_type_allocated_seat_count
{{ template "internal/stats/stats_label_count_series.sql" }}
{{ template "internal/stats/stats_label_count_series_by_name.sql" }}
{{ template "internal/stats/stats_running_total_series.sql" }}
{{ template "internal/stats/stats_running_total_series_by_name.sql" }}
{{ template "internal/events/validate_cfs_submission_label_ids.sql" }} -- Dependency for CFS submission label sync
{{ template "internal/events/sync_cfs_submission_labels.sql" }} -- Dependency for add/update_cfs_submission
{{ template "internal/events/validate_questionnaire_questions_payload.sql" }} -- Do not sort alphabetically, dependency for add/update_event and validate_questionnaire_answers_payload
{{ template "internal/enrollment/validate_questionnaire_answers_payload.sql" }} -- Do not sort alphabetically, dependency for attend_event, submit_event_registration_answers and prepare_event_checkout_purchase
{{ template "common/get_event_full.sql" }}
{{ template "common/get_event_summary.sql" }}
{{ template "event/get_event_registration_questions.sql" }} -- Do not sort alphabetically, dependency for dashboard-user/list_user_events
{{ template "common/list_group_organizers.sql" }} -- Dependency for get_group_full
{{ template "common/get_group_full.sql" }}
{{ template "images/is_open_graph_image.sql" }}
{{ template "common/list_event_cfs_labels.sql" }}
{{ template "common/list_redirect_alliances.sql" }}
{{ template "redirector/list_redirects.sql" }}
{{ template "common/mark_custom_domain_active.sql" }}
{{ template "common/normalize_custom_domain_hostname.sql" }}
{{ template "common/mark_custom_domain_verified.sql" }}
{{ template "common/resolve_active_custom_domain.sql" }}
{{ template "common/resolve_distribution_link.sql" }}
{{ template "common/search_events.sql" }}
{{ template "common/search_groups.sql" }}

{{ template "jobs/job_application_summary_json.sql" }}
{{ template "jobs/job_summary_json.sql" }}
{{ template "jobs/add_job.sql" }}
{{ template "jobs/add_job_application.sql" }}
{{ template "jobs/delete_job.sql" }}
{{ template "jobs/get_job_by_slug.sql" }}
{{ template "jobs/list_user_jobs.sql" }}
{{ template "jobs/search_jobs.sql" }}
{{ template "jobs/update_job.sql" }}
{{ template "jobs/update_job_published.sql" }}

{{ template "opportunities/opportunity_board.sql" }}

{{ template "landscape/landscape_entry_json.sql" }}
{{ template "landscape/add_landscape_entry.sql" }}
{{ template "landscape/delete_landscape_entry.sql" }}
{{ template "landscape/list_alliance_landscape_entries.sql" }}
{{ template "landscape/list_alliance_landscape_entries_for_export.sql" }}
{{ template "landscape/search_landscape_entries.sql" }}
{{ template "landscape/update_landscape_entry.sql" }}
{{ template "landscape/update_landscape_entry_published.sql" }}

{{ template "alliance/get_alliance_id_by_name.sql" }}
{{ template "alliance/get_alliance_name_by_id.sql" }}
{{ template "alliance/get_alliance_recently_added_groups.sql" }}
{{ template "alliance/get_alliance_site_stats.sql" }}
{{ template "alliance/get_alliance_upcoming_events.sql" }}
{{ template "alliance/list_alliance_members.sql" }}
{{ template "alliance/list_alliance_members_for_export.sql" }}
{{ template "alliance/list_public_partner_integrations.sql" }}
{{ template "alliance/update_alliance_views.sql" }}

{{ template "dashboard-common/search_user.sql" }}
{{ template "dashboard-common/add_intentional_dating_intro.sql" }}
{{ template "dashboard-common/list_book_exchange_members.sql" }}
{{ template "dashboard-common/list_intentional_dating_opt_ins.sql" }}
{{ template "dashboard-common/update_group.sql" }}

{{ template "dashboard-alliance/activate_group.sql" }}
{{ template "dashboard-alliance/add_alliance.sql" }}
{{ template "dashboard-alliance/add_alliance_team_member.sql" }}
{{ template "dashboard-alliance/add_event_category.sql" }}
{{ template "dashboard-alliance/add_group.sql" }}
{{ template "dashboard-alliance/add_group_category.sql" }}
{{ template "dashboard-alliance/add_partner_integration.sql" }}
{{ template "dashboard-alliance/add_region.sql" }}
{{ template "dashboard-alliance/deactivate_group.sql" }}
{{ template "dashboard-alliance/delete_alliance_team_member.sql" }}
{{ template "dashboard-alliance/delete_event_category.sql" }}
{{ template "dashboard-alliance/delete_group.sql" }}
{{ template "dashboard-alliance/delete_group_category.sql" }}
{{ template "dashboard-alliance/delete_partner_integration.sql" }}
{{ template "dashboard-alliance/delete_region.sql" }}
{{ template "dashboard-alliance/get_alliance_stats.sql" }}
{{ template "dashboard-alliance/get_alliance_contact_recipients_summary.sql" }}
{{ template "dashboard-alliance/list_alliance_audit_logs.sql" }}
{{ template "dashboard-alliance/list_alliance_contact_filter_options.sql" }}
{{ template "dashboard-alliance/list_alliance_roles.sql" }}
{{ template "dashboard-alliance/list_alliance_team_members.sql" }}
{{ template "dashboard-alliance/list_group_categories.sql" }}
{{ template "dashboard-alliance/list_partner_integrations.sql" }}
{{ template "dashboard-alliance/list_regions.sql" }}
{{ template "dashboard-alliance/list_user_alliances.sql" }}
{{ template "dashboard-alliance/update_alliance.sql" }}
{{ template "dashboard-alliance/update_alliance_report_public_enabled.sql" }}
{{ template "dashboard-alliance/update_alliance_team_member_role.sql" }}
{{ template "dashboard-alliance/update_event_category.sql" }}
{{ template "dashboard-alliance/update_group_category.sql" }}
{{ template "dashboard-alliance/update_partner_integration.sql" }}
{{ template "dashboard-alliance/update_region.sql" }}

{{ template "internal/ticketing/get_event_ticket_capacity.sql" }} -- Dependency for add/update_event
{{ template "dashboard-group/list_payment_currency_codes.sql" }} -- Dependency for payment currency validation and dashboard forms
{{ template "internal/payments/validate_payment_currency_code.sql" }} -- Dependency for payment amount validation
{{ template "internal/payments/validate_payment_amount.sql" }} -- Dependency for event ticketing and checkout validation
{{ template "internal/ticketing/validate_event_capacity.sql" }} -- Dependency for add/update_event
{{ template "internal/events/validate_event_cfs_labels_payload.sql" }} -- Dependency for add/update_event
{{ template "internal/ticketing/validate_event_discount_codes_payload.sql" }} -- Dependency for validate_event_ticketing_payload
{{ template "internal/events/validate_event_enrollment_payload.sql" }} -- Dependency for add/update_event
{{ template "internal/events/validate_event_series_action_event_ids.sql" }} -- Dependency for series actions
{{ template "internal/ticketing/validate_event_ticket_types_payload.sql" }} -- Dependency for validate_event_ticketing_payload
{{ template "internal/ticketing/validate_event_ticketing_payload.sql" }} -- Dependency for add/update_event
{{ template "internal/events/validate_add_event_dates.sql" }} -- Dependency for add_event
{{ template "event/promote_event_waitlist.sql" }} -- Dependency for update_event and leave_event
{{ template "internal/ticketing/sync_event_discount_codes.sql" }} -- Dependency for add/update_event
{{ template "internal/ticketing/sync_event_ticket_types.sql" }} -- Dependency for add/update_event
{{ template "dashboard-group/is_event_meeting_in_sync.sql" }} -- Dependency for update_event
{{ template "dashboard-group/is_session_meeting_in_sync.sql" }} -- Dependency for sync_event_sessions
{{ template "internal/events/sync_event_cfs_labels.sql" }} -- Dependency for add/update_event
{{ template "internal/events/sync_event_hosts_speakers_sponsors.sql" }} -- Dependency for add/update_event
{{ template "dashboard-group/sync_event_organizers.sql" }} -- Dependency for add/update_event
{{ template "dashboard-group/sync_event_sessions.sql" }} -- Dependency for add/update_event
{{ template "dashboard-group/list_event_series_cancelable_event_ids.sql" }} -- Dependency for cancel_event
{{ template "dashboard-group/lock_events_for_cancellation.sql" }} -- Dependency for cancel_event
{{ template "dashboard-group/accept_event_invitation_request.sql" }}
{{ template "dashboard-group/approve_group_member_phone_request.sql" }}
{{ template "dashboard-group/add_event.sql" }}
{{ template "dashboard-group/add_event_series.sql" }}
{{ template "dashboard-group/add_group_sponsor.sql" }}
{{ template "dashboard-group/add_group_team_member.sql" }}
{{ template "dashboard-group/approve_group_join_request.sql" }}
{{ template "dashboard-group/cancel_event_attendee_attendance.sql" }}
{{ template "dashboard-group/cancel_event_attendee_invitation.sql" }}
{{ template "dashboard-group/cancel_event.sql" }}
{{ template "dashboard-group/cancel_event_series_events.sql" }}
{{ template "dashboard-group/check_in_event.sql" }}
{{ template "dashboard-group/check_in_attendee_by_code.sql" }} -- Depends on check_in_event
{{ template "dashboard-group/block_group_member_linkedin.sql" }}
{{ template "dashboard-group/claim_event_cohost_invitation_recipients.sql" }}
{{ template "dashboard-group/delete_custom_domain.sql" }}
{{ template "dashboard-group/delete_event.sql" }}
{{ template "dashboard-group/delete_event_series_events.sql" }}
{{ template "dashboard-group/delete_group_member.sql" }}
{{ template "dashboard-group/delete_group_sponsor.sql" }}
{{ template "dashboard-group/delete_group_team_member.sql" }}
{{ template "dashboard-group/get_cfs_submission_notification_data.sql" }}
{{ template "dashboard-group/create_collaboration_project.sql" }}
{{ template "dashboard-group/get_custom_domain.sql" }}
{{ template "dashboard-group/get_event_cohost_notification_data.sql" }}
{{ template "dashboard-group/get_event_analytics.sql" }}
{{ template "internal/events/get_event_delete_eligibility.sql" }} -- Dependency for dashboard event summaries
{{ template "dashboard-group/get_event_summary_dashboard.sql" }} -- Dependency for list_group_events
{{ template "dashboard-group/event_sponsor_reporting.sql" }}
{{ template "dashboard-group/event_growth.sql" }}
{{ template "dashboard-group/get_group_sponsor.sql" }}
{{ template "dashboard-group/get_group_collaboration_dashboard.sql" }}
{{ template "dashboard-group/get_group_stats.sql" }}
{{ template "dashboard-group/get_group_utm_settings.sql" }}
{{ template "dashboard-group/group_has_event.sql" }}
{{ template "dashboard-group/get_distribution_dashboard.sql" }}
{{ template "dashboard-group/invite_event_attendee.sql" }}
{{ template "dashboard-group/invite_collaboration_project_member.sql" }}
{{ template "dashboard-group/list_cfs_submission_statuses_for_review.sql" }}
{{ template "dashboard-group/list_event_approved_cfs_submissions.sql" }}
{{ template "dashboard-group/list_event_attendees_ids.sql" }}
{{ template "dashboard-group/list_event_categories.sql" }}
{{ template "dashboard-group/list_event_cfs_submissions.sql" }}
{{ template "dashboard-group/list_event_kinds.sql" }}
{{ template "dashboard-group/list_event_series_event_ids.sql" }}
{{ template "dashboard-group/list_group_refunds.sql" }}
{{ template "dashboard-group/list_event_series_publishable_event_ids.sql" }}
{{ template "dashboard-group/list_event_waitlist_ids.sql" }}
{{ template "dashboard-group/list_group_coffee_meet_subscribers.sql" }}
{{ template "dashboard-group/list_group_audit_logs.sql" }}
{{ template "dashboard-group/list_group_join_requests.sql" }}
{{ template "dashboard-group/list_group_check_in_events.sql" }}
{{ template "dashboard-group/list_group_events.sql" }}
{{ template "dashboard-group/list_group_members.sql" }}
{{ template "dashboard-group/list_group_parent_options.sql" }}
{{ template "dashboard-group/list_group_members_ids.sql" }}
{{ template "dashboard-group/list_group_roles.sql" }}
{{ template "dashboard-group/list_group_sponsors.sql" }}
{{ template "dashboard-group/list_group_team_members.sql" }}
{{ template "dashboard-group/list_group_team_members_ids.sql" }}
{{ template "dashboard-group/list_session_kinds.sql" }}
{{ template "dashboard-group/list_user_groups.sql" }}
{{ template "dashboard-group/manual_check_in_event.sql" }}
{{ template "dashboard-group/publish_event.sql" }}
{{ template "dashboard-group/publish_event_series_events.sql" }}
{{ template "dashboard-group/reject_event_invitation_request.sql" }}
{{ template "dashboard-group/resolve_event_custom_notification_recipient_ids.sql" }}
{{ template "dashboard-group/request_group_member_phone.sql" }}
{{ template "dashboard-group/request_event_cohost.sql" }}
{{ template "dashboard-group/search_event_attendees.sql" }}
{{ template "dashboard-group/search_event_invitation_requests.sql" }}
{{ template "dashboard-group/search_event_waitlist.sql" }}
{{ template "dashboard-group/reject_group_join_request.sql" }}
{{ template "dashboard-group/decide_event_cohost.sql" }}
{{ template "dashboard-group/unpublish_event.sql" }}
{{ template "dashboard-group/unpublish_event_series_events.sql" }}
{{ template "dashboard-group/update_cfs_submission.sql" }}
{{ template "dashboard-group/validate_update_event_dates.sql" }} -- Dependency for update_event
{{ template "dashboard-group/update_group_event_defaults.sql" }}
{{ template "dashboard-group/update_group_report_public_enabled.sql" }}
{{ template "dashboard-group/update_group_sponsor.sql" }}
{{ template "dashboard-group/update_group_sponsor_featured.sql" }}
{{ template "dashboard-group/update_group_team_member_role.sql" }}
{{ template "dashboard-group/upsert_custom_domain.sql" }}
{{ template "dashboard-group/update_event.sql" }}
{{ template "dashboard-group/move_event.sql" }}
{{ template "dashboard-group/list_group_move_targets.sql" }}

{{ template "dashboard-gtm/gtm_lead_json.sql" }}
{{ template "dashboard-gtm/gtm_agent_draft_json.sql" }}
{{ template "dashboard-gtm/gtm_is_legal_transition.sql" }}
{{ template "dashboard-gtm/add_gtm_lead_activity.sql" }}
{{ template "dashboard-gtm/add_gtm_lead.sql" }}
{{ template "dashboard-gtm/update_gtm_lead.sql" }}
{{ template "dashboard-gtm/delete_gtm_lead.sql" }}
{{ template "dashboard-gtm/transition_gtm_lead.sql" }}
{{ template "dashboard-gtm/list_gtm_leads.sql" }}
{{ template "dashboard-gtm/get_gtm_lead.sql" }}
{{ template "dashboard-gtm/add_gtm_agent_draft.sql" }}
{{ template "dashboard-gtm/list_gtm_agent_drafts.sql" }}
{{ template "dashboard-gtm/apply_gtm_won_side_effects.sql" }}
{{ template "dashboard-gtm/review_gtm_agent_draft.sql" }}
{{ template "dashboard-gtm/suggest_gtm_lead_candidates.sql" }}
{{ template "dashboard-gtm/sponsor_campaign.sql" }}

{{ template "dashboard-user/accept_alliance_team_invitation.sql" }}
{{ template "dashboard-user/accept_event_attendee_invitation.sql" }}
{{ template "dashboard-user/accept_group_team_invitation.sql" }}
{{ template "dashboard-user/accept_session_proposal_co_speaker_invitation.sql" }}
{{ template "dashboard-user/add_session_proposal.sql" }}
{{ template "dashboard-user/add_user_affiliation.sql" }}
{{ template "dashboard-user/count_user_pending_invitations.sql" }}
{{ template "dashboard-user/delete_session_proposal.sql" }}
{{ template "dashboard-user/delete_user_affiliation.sql" }}
{{ template "dashboard-user/get_user_notification_preferences.sql" }}
{{ template "dashboard-user/list_landscape_entry_options.sql" }}
{{ template "dashboard-user/list_session_proposal_levels.sql" }}
{{ template "dashboard-user/list_user_coffee_meet_subscriptions.sql" }}
{{ template "dashboard-user/list_user_audit_logs.sql" }}
{{ template "dashboard-user/list_user_cfs_submissions.sql" }}
{{ template "dashboard-user/list_user_affiliations.sql" }}
{{ template "dashboard-user/list_user_alliance_team_invitations.sql" }}
{{ template "dashboard-user/list_user_event_invitations.sql" }}
{{ template "dashboard-user/list_user_event_cohost_invitations.sql" }}
{{ template "dashboard-user/list_user_check_in_events.sql" }}
{{ template "dashboard-user/list_user_events.sql" }}
{{ template "dashboard-user/list_user_group_team_invitations.sql" }}
{{ template "dashboard-user/list_user_notification_group_options.sql" }}
{{ template "dashboard-user/list_user_pending_session_proposal_co_speaker_invitations.sql" }}
{{ template "dashboard-user/list_user_session_proposals.sql" }}
{{ template "dashboard-user/reject_alliance_team_invitation.sql" }}
{{ template "dashboard-user/reject_event_attendee_invitation.sql" }}
{{ template "dashboard-user/reject_group_team_invitation.sql" }}
{{ template "dashboard-user/reject_session_proposal_co_speaker_invitation.sql" }}
{{ template "dashboard-user/resubmit_cfs_submission.sql" }}
{{ template "dashboard-user/mute_user_group_notifications.sql" }}
{{ template "dashboard-user/submit_event_registration_answers.sql" }}
{{ template "dashboard-user/unsubscribe_coffee_meet.sql" }}
{{ template "dashboard-user/unmute_user_group_notifications.sql" }}
{{ template "dashboard-user/upsert_coffee_meet_subscription.sql" }}
{{ template "dashboard-user/update_user_notification_preferences.sql" }}
{{ template "dashboard-user/update_session_proposal.sql" }}
{{ template "dashboard-user/withdraw_cfs_submission.sql" }}

{{ template "event/add_cfs_submission.sql" }}
{{ template "event/attend_event.sql" }}
{{ template "event/check_in_event.sql" }}
{{ template "event/ensure_event_is_active.sql" }}
{{ template "event/get_event_attendance.sql" }}
{{ template "event/get_event_full_by_slug.sql" }}
{{ template "event/get_event_summary_by_id.sql" }}
{{ template "event/get_event_utm_allowlist.sql" }}
{{ template "event/event_surveys.sql" }}
{{ template "event/is_event_check_in_window_open.sql" }}
{{ template "internal/ticketing/release_event_discount_code_availability.sql" }} -- Dependency for event and payments flows
{{ template "internal/enrollment/release_event_checkout_attendee_hold.sql" }} -- Dependency for checkout expiration flows
{{ template "internal/payments/refund_free_event_purchase.sql" }} -- Dependency for leave_event
{{ template "event/leave_event.sql" }}
{{ template "event/list_user_session_proposals_for_cfs_event.sql" }}
{{ template "event/update_event_views.sql" }}
{{ template "event/update_event_views_utm.sql" }}

{{ template "group/get_group_full_by_slug.sql" }}
{{ template "group/get_public_collaboration_project.sql" }}
{{ template "group/get_group_membership_status.sql" }}
{{ template "group/get_group_past_events.sql" }}
{{ template "group/get_group_upcoming_events.sql" }}
{{ template "group/is_group_member.sql" }}
{{ template "group/join_group.sql" }}
{{ template "group/leave_group.sql" }}
{{ template "group/book_collaboration_office_hour.sql" }}
{{ template "group/submit_collaboration_update.sql" }}
{{ template "group/update_group_views.sql" }}

{{ template "inbox/add_inbox_group_reply.sql" }}
{{ template "inbox/add_inbox_user_message.sql" }}
{{ template "inbox/close_inbox_conversation.sql" }}
{{ template "inbox/count_group_open_inbox_conversations.sql" }}
{{ template "inbox/get_group_inbox_conversation.sql" }}
{{ template "inbox/get_inbox_contact_context.sql" }}
{{ template "inbox/get_user_inbox_conversation.sql" }}
{{ template "inbox/list_group_inbox_conversations.sql" }}
{{ template "inbox/list_inbox_recipient_ids.sql" }}
{{ template "inbox/list_user_inbox_conversations.sql" }}
{{ template "inbox/mark_inbox_conversation_as_spam.sql" }}
{{ template "inbox/start_inbox_conversation.sql" }}
{{ template "inbox/unmark_inbox_conversation_as_spam.sql" }}

{{ template "internal/meetings/get_event_meeting_sync_state_hash.sql" }} -- Dependency for meeting sync completion functions
{{ template "internal/meetings/get_session_meeting_sync_state_hash.sql" }} -- Dependency for meeting sync completion functions
{{ template "meetings/add_meeting.sql" }}
{{ template "meetings/append_meeting_recording_url.sql" }}
{{ template "meetings/assign_zoom_host_user.sql" }}
{{ template "meetings/claim_google_meet_recording_for_publish.sql" }}
{{ template "meetings/claim_meeting_for_auto_end.sql" }}
{{ template "meetings/claim_meeting_out_of_sync.sql" }}
{{ template "meetings/delete_meeting.sql" }}
{{ template "meetings/mark_google_meet_recording_published.sql" }}
{{ template "meetings/mark_stale_google_meet_recording_publish_claims_unknown.sql" }}
{{ template "meetings/mark_stale_meeting_auto_end_checks_unknown.sql" }}
{{ template "meetings/mark_stale_meeting_syncs_unknown.sql" }}
{{ template "meetings/release_google_meet_recording_publish_claim.sql" }}
{{ template "meetings/release_meeting_auto_end_check_claim.sql" }}
{{ template "meetings/release_meeting_sync_claim.sql" }}
{{ template "meetings/set_meeting_auto_end_check_outcome.sql" }}
{{ template "meetings/set_meeting_error.sql" }}
{{ template "meetings/update_meeting.sql" }}

{{ template "notifications/claim_pending_notification.sql" }}
{{ template "notifications/enqueue_notification.sql" }} -- Dependency for due, tracked custom and auth notification helpers
{{ template "notifications/filter_notification_recipient_ids.sql" }} -- Dependency for optional notification preferences
{{ template "notifications/opt_out_user_from_optional_notifications.sql" }}
{{ template "notifications/enqueue_due_coffee_meet_suggestions.sql" }}
{{ template "notifications/enqueue_due_distribution_content_reminders.sql" }}
{{ template "notifications/enqueue_due_opportunity_digests.sql" }}
{{ template "notifications/enqueue_due_event_reminders.sql" }}
{{ template "notifications/enqueue_due_collaboration_reminders.sql" }}
{{ template "notifications/manual_requeue_notifications.sql" }}
{{ template "notifications/mark_stale_processing_notifications_unknown.sql" }}
{{ template "notifications/requeue_notification.sql" }}
{{ template "internal/notifications/track_custom_notification.sql" }} -- Dependency for enqueue_tracked_custom_notification
{{ template "notifications/enqueue_tracked_custom_notification.sql" }}
{{ template "notifications/enqueue_tracked_alliance_custom_notification.sql" }}
{{ template "notifications/update_notification.sql" }}

{{ template "internal/payments/create_event_purchase_refund_payment_job.sql" }}
{{ template "internal/payments/payment_job_max_attempts.sql" }}
{{ template "internal/payments/payment_job_retry_delay.sql" }}
{{ template "internal/payments/payment_job_is_exhausted.sql" }}
{{ template "internal/payments/payment_job_is_ready.sql" }}
{{ template "internal/payments/payment_job_to_json.sql" }}
{{ template "internal/payments/event_purchase_refund_to_json.sql" }}
{{ template "internal/payments/payment_job_payload.sql" }}
{{ template "internal/payments/complete_payment_job.sql" }}

{{ template "payments/attach_checkout_session_to_event_purchase.sql" }}
{{ template "payments/cancel_event_checkout.sql" }}
{{ template "payments/claim_payment_job.sql" }}
{{ template "payments/complete_event_purchase_refund_recovery.sql" }}
{{ template "payments/complete_external_event_purchase.sql" }}
{{ template "payments/complete_free_event_purchase.sql" }}
{{ template "payments/expire_event_purchase_for_checkout_session.sql" }}
{{ template "payments/finalize_event_purchase_refund.sql" }}
{{ template "payments/get_event_purchase_refund.sql" }}
{{ template "payments/get_event_purchase_refund_recovery_context.sql" }}
{{ template "internal/payments/prepare_event_checkout_expire_previous_hold.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "payments/prepare_event_checkout_expire_stale_holds.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "internal/payments/prepare_event_checkout_find_existing_purchase.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "payments/prepare_event_checkout_get_purchase_summary.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "internal/payments/prepare_event_checkout_reserve_discount_code_availability.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "internal/payments/prepare_event_checkout_validate_and_resolve_pricing.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "payments/prepare_event_checkout_validate_attendee_state.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "payments/prepare_event_checkout_validate_event.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "payments/upsert_pending_registration_answers.sql" }} -- Dependency for prepare_event_checkout_purchase
{{ template "payments/prepare_event_checkout_purchase.sql" }}
{{ template "payments/queue_event_refund_request_approval.sql" }}
{{ template "payments/reconcile_event_purchase_for_checkout_session.sql" }}
{{ template "payments/record_payment_job_failure.sql" }}
{{ template "payments/record_event_purchase_refund_pending.sql" }}
{{ template "payments/record_event_purchase_refund_succeeded.sql" }}
{{ template "payments/record_event_purchase_refund_terminal_failed.sql" }}
{{ template "payments/reject_event_refund_request.sql" }}
{{ template "internal/ticketing/release_event_discount_code_availability.sql" }}
{{ template "payments/request_event_refund.sql" }}
{{ template "payments/requeue_event_purchase_refund.sql" }}
{{ template "payments/requeue_stale_payment_job_claims.sql" }}

{{ template "site/get_filters_options.sql" }}
{{ template "site/get_site_home_stats.sql" }}
{{ template "site/get_site_recently_added_groups.sql" }}
{{ template "site/get_site_settings.sql" }}
{{ template "site/get_site_stats.sql" }}
{{ template "site/get_site_upcoming_events.sql" }}
{{ template "site/list_alliances.sql" }}

{{ template "triggers/check_session_within_event_bounds.sql" }}
{{ template "triggers/prevent_audit_log_mutation.sql" }}

---- create above / drop below ----

-- Nothing to do
