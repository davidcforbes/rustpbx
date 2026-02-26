-- 4iiz Schema Foundation
CREATE SCHEMA IF NOT EXISTS iiz;

CREATE TABLE IF NOT EXISTS iiz.schema_migrations (
    version TEXT PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Auto-update updated_at on row modification
CREATE OR REPLACE FUNCTION iiz.set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-create monthly partitions on INSERT.
-- TG_ARGV[0] = partition column name.
CREATE OR REPLACE FUNCTION iiz.create_monthly_partition()
RETURNS TRIGGER AS $$
DECLARE
    partition_name TEXT;
    start_date DATE;
    end_date DATE;
    partition_col TEXT;
BEGIN
    partition_col := TG_ARGV[0];
    EXECUTE format('SELECT date_trunc(''month'', ($1).%I)::date', partition_col)
        INTO start_date USING NEW;
    end_date := (start_date + interval '1 month')::date;
    partition_name := format('%I.%I', TG_TABLE_SCHEMA,
        TG_TABLE_NAME || '_' || to_char(start_date, 'YYYY_MM'));
    IF NOT EXISTS (
        SELECT 1 FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = TG_TABLE_SCHEMA
          AND c.relname = TG_TABLE_NAME || '_' || to_char(start_date, 'YYYY_MM')
    ) THEN
        EXECUTE format(
            'CREATE TABLE IF NOT EXISTS %s PARTITION OF %I.%I FOR VALUES FROM (%L) TO (%L)',
            partition_name, TG_TABLE_SCHEMA, TG_TABLE_NAME, start_date, end_date
        );
        RAISE NOTICE 'Created partition: %', partition_name;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- NOTIFY on config table changes for moka cache invalidation
CREATE OR REPLACE FUNCTION iiz.notify_config_change()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'iiz_config_changed',
        json_build_object(
            'table', TG_TABLE_NAME,
            'op', TG_OP,
            'id', COALESCE(NEW.id, OLD.id)
        )::text
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Helper: apply updated_at trigger to a table
CREATE OR REPLACE FUNCTION iiz.add_updated_at_trigger(table_name TEXT)
RETURNS void AS $$
BEGIN
    EXECUTE format(
        'CREATE TRIGGER set_updated_at BEFORE UPDATE ON iiz.%I FOR EACH ROW EXECUTE FUNCTION iiz.set_updated_at()',
        table_name
    );
END;
$$ LANGUAGE plpgsql;

-- Helper: apply config change notification trigger
CREATE OR REPLACE FUNCTION iiz.add_notify_trigger(table_name TEXT)
RETURNS void AS $$
BEGIN
    EXECUTE format(
        'CREATE TRIGGER notify_change AFTER INSERT OR UPDATE OR DELETE ON iiz.%I FOR EACH ROW EXECUTE FUNCTION iiz.notify_config_change()',
        table_name
    );
END;
$$ LANGUAGE plpgsql;
