-- 131_notify_triggers.sql
-- Apply NOTIFY triggers to config tables that don't already have them.
-- Many tables already have triggers from their individual migration files;
-- this catches any that were missed and serves as a single reference.

DO $$
DECLARE
    tbl TEXT;
    config_tables TEXT[] := ARRAY[
        'tracking_sources', 'receiving_numbers', 'number_pools', 'number_pool_members',
        'call_settings', 'tracking_numbers', 'text_numbers', 'target_numbers',
        'schedules', 'schedule_holidays', 'voice_menus', 'voice_menu_options',
        'queues', 'queue_agents', 'smart_routers', 'smart_router_rules',
        'geo_routers', 'geo_router_rules', 'routing_tables', 'routing_table_routes',
        'agent_scripts', 'voicemail_boxes',
        'workflows', 'workflow_nodes', 'workflow_edges',
        'triggers', 'trigger_conditions', 'trigger_actions',
        'lambdas', 'lambda_env_vars',
        'webhooks', 'webhook_subscriptions',
        'lead_reactor_configs', 'lead_reactor_actions',
        'smart_dialer_configs', 'form_reactor_entries',
        'keyword_spotting_configs', 'keyword_spotting_keywords', 'keyword_spotting_numbers',
        'chat_widgets',
        'ask_ai_configs', 'summary_configs', 'knowledge_banks', 'knowledge_bank_documents',
        'voice_ai_agents', 'chat_ai_agents', 'chat_ai_configs', 'dialogflow_configs',
        'tags', 'scoring_configs', 'notification_rules',
        'account_variables'
    ];
BEGIN
    FOREACH tbl IN ARRAY config_tables
    LOOP
        IF NOT EXISTS (
            SELECT 1 FROM pg_trigger
            WHERE tgname = 'notify_change'
              AND tgrelid = format('iiz.%I', tbl)::regclass
        ) THEN
            PERFORM iiz.add_notify_trigger(tbl);
        END IF;
    END LOOP;
END $$;
