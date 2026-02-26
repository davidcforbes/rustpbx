-- 130_rls_policies.sql
-- Apply Row-Level Security policies to all tenant-scoped tables.
-- Each table with account_id gets a tenant_isolation policy that filters
-- on app.current_account_id and excludes soft-deleted rows.

-- Helper to apply standard RLS policy
CREATE OR REPLACE FUNCTION iiz.apply_rls(table_name TEXT)
RETURNS void AS $$
BEGIN
    EXECUTE format('ALTER TABLE iiz.%I ENABLE ROW LEVEL SECURITY', table_name);
    EXECUTE format('ALTER TABLE iiz.%I FORCE ROW LEVEL SECURITY', table_name);
    EXECUTE format(
        'CREATE POLICY tenant_isolation ON iiz.%I
         FOR ALL
         USING (account_id = current_setting(''app.current_account_id'')::uuid AND deleted_at IS NULL)
         WITH CHECK (account_id = current_setting(''app.current_account_id'')::uuid)',
        table_name
    );
END;
$$ LANGUAGE plpgsql;

-- Apply to ALL tables with account_id (in alphabetical order for clarity):
SELECT iiz.apply_rls('a2p_campaigns');
SELECT iiz.apply_rls('account_variables');
SELECT iiz.apply_rls('active_calls');
SELECT iiz.apply_rls('agent_scripts');
SELECT iiz.apply_rls('agent_state_log');
SELECT iiz.apply_rls('api_log_entries');
SELECT iiz.apply_rls('appointments');
SELECT iiz.apply_rls('ask_ai_configs');
SELECT iiz.apply_rls('authorized_contacts');
SELECT iiz.apply_rls('blocked_numbers');
SELECT iiz.apply_rls('bulk_messages');
SELECT iiz.apply_rls('business_info');
SELECT iiz.apply_rls('call_ai_summaries');
SELECT iiz.apply_rls('call_annotations');
SELECT iiz.apply_rls('call_daily_summary');
SELECT iiz.apply_rls('call_flow_events');
SELECT iiz.apply_rls('call_keyword_hits');
SELECT iiz.apply_rls('call_records');
SELECT iiz.apply_rls('call_settings');
SELECT iiz.apply_rls('call_tags');
SELECT iiz.apply_rls('call_transcription_segments');
SELECT iiz.apply_rls('call_visitor_sessions');
SELECT iiz.apply_rls('caller_id_cnam');
SELECT iiz.apply_rls('chat_ai_agents');
SELECT iiz.apply_rls('chat_ai_configs');
SELECT iiz.apply_rls('chat_records');
SELECT iiz.apply_rls('chat_widgets');
SELECT iiz.apply_rls('compliance_addresses');
SELECT iiz.apply_rls('compliance_applications');
SELECT iiz.apply_rls('compliance_requirements');
SELECT iiz.apply_rls('contact_list_members');
SELECT iiz.apply_rls('contact_lists');
SELECT iiz.apply_rls('custom_reports');
SELECT iiz.apply_rls('dialogflow_configs');
SELECT iiz.apply_rls('dnc_entries');
SELECT iiz.apply_rls('dnt_entries');
SELECT iiz.apply_rls('export_records');
SELECT iiz.apply_rls('fax_records');
SELECT iiz.apply_rls('form_reactor_entries');
SELECT iiz.apply_rls('form_records');
SELECT iiz.apply_rls('frequency_limits');
SELECT iiz.apply_rls('geo_router_rules');
SELECT iiz.apply_rls('geo_routers');
SELECT iiz.apply_rls('keyword_spotting_configs');
SELECT iiz.apply_rls('keyword_spotting_keywords');
SELECT iiz.apply_rls('keyword_spotting_numbers');
SELECT iiz.apply_rls('knowledge_bank_documents');
SELECT iiz.apply_rls('knowledge_bank_embeddings');
SELECT iiz.apply_rls('knowledge_banks');
SELECT iiz.apply_rls('lambda_env_vars');
SELECT iiz.apply_rls('lambdas');
SELECT iiz.apply_rls('lead_reactor_actions');
SELECT iiz.apply_rls('lead_reactor_configs');
SELECT iiz.apply_rls('monitoring_events');
SELECT iiz.apply_rls('notification_rules');
SELECT iiz.apply_rls('notifications');
SELECT iiz.apply_rls('number_pool_members');
SELECT iiz.apply_rls('number_pools');
SELECT iiz.apply_rls('port_requests');
SELECT iiz.apply_rls('queue_agents');
SELECT iiz.apply_rls('queues');
SELECT iiz.apply_rls('receiving_numbers');
SELECT iiz.apply_rls('reminders');
SELECT iiz.apply_rls('routing_table_routes');
SELECT iiz.apply_rls('routing_tables');
SELECT iiz.apply_rls('schedule_holidays');
SELECT iiz.apply_rls('schedules');
SELECT iiz.apply_rls('scoring_configs');
SELECT iiz.apply_rls('smart_dialer_configs');
SELECT iiz.apply_rls('smart_router_rules');
SELECT iiz.apply_rls('smart_routers');
SELECT iiz.apply_rls('summary_configs');
SELECT iiz.apply_rls('tags');
SELECT iiz.apply_rls('target_numbers');
SELECT iiz.apply_rls('text_messages');
SELECT iiz.apply_rls('text_numbers');
SELECT iiz.apply_rls('text_records');
SELECT iiz.apply_rls('toll_free_registrations');
SELECT iiz.apply_rls('tracking_numbers');
SELECT iiz.apply_rls('tracking_sources');
SELECT iiz.apply_rls('trigger_actions');
SELECT iiz.apply_rls('trigger_conditions');
SELECT iiz.apply_rls('triggers');
SELECT iiz.apply_rls('users');
SELECT iiz.apply_rls('video_records');
SELECT iiz.apply_rls('voice_ai_agents');
SELECT iiz.apply_rls('voice_menu_options');
SELECT iiz.apply_rls('voice_menus');
SELECT iiz.apply_rls('voice_registration_history');
SELECT iiz.apply_rls('voice_registrations');
SELECT iiz.apply_rls('voicemail_boxes');
SELECT iiz.apply_rls('voicemail_messages');
SELECT iiz.apply_rls('webhook_deliveries');
SELECT iiz.apply_rls('webhook_subscriptions');
SELECT iiz.apply_rls('webhooks');
SELECT iiz.apply_rls('workflow_edges');
SELECT iiz.apply_rls('workflow_nodes');
SELECT iiz.apply_rls('workflows');

-- Accounts: special policy (id = current_account_id, not account_id)
ALTER TABLE iiz.accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE iiz.accounts FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON iiz.accounts
    FOR ALL
    USING (id = current_setting('app.current_account_id')::uuid AND deleted_at IS NULL)
    WITH CHECK (id = current_setting('app.current_account_id')::uuid);

-- Skip RLS on presence and locations (nullable account_id, ephemeral)
