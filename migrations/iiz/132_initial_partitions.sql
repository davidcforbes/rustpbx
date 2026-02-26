-- 132_initial_partitions.sql
-- Pre-create current and next month partitions for all partitioned tables.
-- A background job should create future partitions on an ongoing basis;
-- this migration bootstraps the initial two months so the system is
-- immediately usable after migration.

DO $$
DECLARE
    current_start DATE := date_trunc('month', now())::date;
    current_end DATE := (current_start + interval '1 month')::date;
    next_start DATE := current_end;
    next_end DATE := (next_start + interval '1 month')::date;
    suffix_current TEXT := to_char(current_start, 'YYYY_MM');
    suffix_next TEXT := to_char(next_start, 'YYYY_MM');
    tbl RECORD;
BEGIN
    FOR tbl IN
        SELECT unnest(ARRAY[
            'call_records', 'call_flow_events', 'call_transcription_segments',
            'call_ai_summaries', 'text_messages', 'agent_state_log',
            'webhook_deliveries', 'api_log_entries'
        ]) AS name
    LOOP
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS iiz.%I PARTITION OF iiz.%I FOR VALUES FROM (%L) TO (%L)',
            tbl.name || '_' || suffix_current, tbl.name, current_start, current_end
        );
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS iiz.%I PARTITION OF iiz.%I FOR VALUES FROM (%L) TO (%L)',
            tbl.name || '_' || suffix_next, tbl.name, next_start, next_end
        );
    END LOOP;
END $$;
