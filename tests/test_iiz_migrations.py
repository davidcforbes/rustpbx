"""
Integration test: apply all iiz migrations to a fresh PostgreSQL database
and verify the schema is correct.

Requires: PostgreSQL running on localhost:5433 (or set IIZ_TEST_DATABASE_URL)

Setup:
    docker run -d --name iiz-test-pg -e POSTGRES_PASSWORD=test -e POSTGRES_DB=iiz_test -p 5433:5432 postgres:16

Run:
    python -m pytest tests/test_iiz_migrations.py -v
"""
import os
import psycopg2
import pytest

DATABASE_URL = os.environ.get(
    "IIZ_TEST_DATABASE_URL",
    "postgresql://postgres:test@localhost:5433/iiz_test"
)

MIGRATIONS_DIR = os.path.join(os.path.dirname(__file__), "..", "migrations", "iiz")


@pytest.fixture(scope="session")
def db_conn():
    """Connect to test database, drop and recreate iiz schema, apply all migrations."""
    conn = psycopg2.connect(DATABASE_URL)
    conn.autocommit = True
    cur = conn.cursor()

    # Clean slate
    cur.execute("DROP SCHEMA IF EXISTS iiz CASCADE")

    # Apply migrations in order
    sql_files = sorted([
        f for f in os.listdir(MIGRATIONS_DIR)
        if f.endswith(".sql") and not f.endswith("_down.sql")
    ])

    for sql_file in sql_files:
        path = os.path.join(MIGRATIONS_DIR, sql_file)
        with open(path, "r") as f:
            sql = f.read()
        try:
            cur.execute(sql)
            conn.commit()
        except Exception as e:
            conn.rollback()
            pytest.fail(f"Migration {sql_file} failed: {e}")

    yield conn
    conn.close()


def test_schema_exists(db_conn):
    """Verify the iiz schema was created."""
    cur = db_conn.cursor()
    cur.execute("""
        SELECT schema_name FROM information_schema.schemata
        WHERE schema_name = 'iiz'
    """)
    assert cur.fetchone() is not None


def test_table_count(db_conn):
    """Verify we have ~80+ tables (base tables, excluding partition children)."""
    cur = db_conn.cursor()
    cur.execute("""
        SELECT COUNT(*) FROM information_schema.tables
        WHERE table_schema = 'iiz'
          AND table_type = 'BASE TABLE'
    """)
    count = cur.fetchone()[0]
    # ~80 base tables + schema_migrations + partition children
    assert count >= 80, f"Expected 80+ tables, got {count}"


def test_partitioned_tables_exist(db_conn):
    """Verify the 8 partitioned parent tables exist."""
    cur = db_conn.cursor()
    partitioned = [
        "call_records", "call_flow_events", "call_transcription_segments",
        "call_ai_summaries", "text_messages", "agent_state_log",
        "webhook_deliveries", "api_log_entries"
    ]
    for table in partitioned:
        cur.execute("""
            SELECT 1 FROM pg_partitioned_table pt
            JOIN pg_class c ON c.oid = pt.partrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'iiz' AND c.relname = %s
        """, (table,))
        result = cur.fetchone()
        assert result is not None, f"{table} is not partitioned"


def test_rls_enabled(db_conn):
    """Verify RLS is enabled on key tables."""
    cur = db_conn.cursor()
    tables = ["users", "call_records", "queues", "tracking_numbers", "accounts"]
    for table in tables:
        cur.execute("""
            SELECT relrowsecurity, relforcerowsecurity
            FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'iiz' AND c.relname = %s
        """, (table,))
        row = cur.fetchone()
        assert row is not None, f"Table {table} not found"
        assert row[0] is True, f"RLS not enabled on {table}"
        assert row[1] is True, f"RLS not forced on {table}"


def test_unlogged_tables(db_conn):
    """Verify ephemeral tables are UNLOGGED."""
    cur = db_conn.cursor()
    unlogged = ["active_calls", "presence", "locations"]
    for table in unlogged:
        cur.execute("""
            SELECT relpersistence FROM pg_class c
            JOIN pg_namespace n ON n.oid = c.relnamespace
            WHERE n.nspname = 'iiz' AND c.relname = %s
        """, (table,))
        row = cur.fetchone()
        assert row is not None, f"Table {table} not found"
        assert row[0] == 'u', f"{table} is not UNLOGGED (persistence={row[0]})"


def test_uuid_primary_keys(db_conn):
    """Verify key tables use UUID primary keys."""
    cur = db_conn.cursor()
    tables = ["accounts", "users", "tracking_numbers", "queues", "tags", "webhooks"]
    for table in tables:
        cur.execute("""
            SELECT data_type FROM information_schema.columns
            WHERE table_schema = 'iiz'
              AND table_name = %s
              AND column_name = 'id'
        """, (table,))
        row = cur.fetchone()
        assert row is not None, f"No 'id' column on {table}"
        assert row[0] == 'uuid', f"{table}.id is {row[0]}, expected uuid"


def test_deleted_at_on_all_tables(db_conn):
    """Verify every table has a deleted_at column (soft delete everywhere)."""
    cur = db_conn.cursor()
    cur.execute("""
        SELECT t.table_name
        FROM information_schema.tables t
        WHERE t.table_schema = 'iiz'
          AND t.table_type = 'BASE TABLE'
          AND t.table_name != 'schema_migrations'
          AND NOT EXISTS (
              SELECT 1 FROM information_schema.columns c
              WHERE c.table_schema = 'iiz'
                AND c.table_name = t.table_name
                AND c.column_name = 'deleted_at'
          )
    """)
    missing = [row[0] for row in cur.fetchall()]
    # Filter out partition children (they inherit deleted_at from parent)
    partition_prefixes = [
        "call_records_", "call_flow_events_", "call_transcription_segments_",
        "call_ai_summaries_", "text_messages_", "agent_state_log_",
        "webhook_deliveries_", "api_log_entries_"
    ]
    missing = [t for t in missing if not any(t.startswith(p) for p in partition_prefixes)]
    assert missing == [], f"Tables missing deleted_at: {missing}"


def test_initial_partitions_created(db_conn):
    """Verify current and next month partitions exist for call_records."""
    cur = db_conn.cursor()
    cur.execute("""
        SELECT COUNT(*) FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'iiz'
          AND c.relname LIKE 'call_records_20%%'
          AND c.relkind = 'r'
    """)
    count = cur.fetchone()[0]
    assert count >= 2, f"Expected at least 2 call_records partitions, got {count}"


def test_enum_types_exist(db_conn):
    """Verify key enum types were created."""
    cur = db_conn.cursor()
    enums = [
        "call_direction", "call_status", "user_role", "agent_status",
        "queue_strategy", "monitor_mode", "summary_type", "compliance_status"
    ]
    for enum_name in enums:
        cur.execute("""
            SELECT 1 FROM pg_type t
            JOIN pg_namespace n ON n.oid = t.typnamespace
            WHERE n.nspname = 'iiz' AND t.typname = %s
        """, (enum_name,))
        assert cur.fetchone() is not None, f"Enum type {enum_name} not found"


def test_notify_triggers_on_config_tables(db_conn):
    """Verify NOTIFY triggers exist on key config tables."""
    cur = db_conn.cursor()
    config_tables = ["queues", "tracking_numbers", "webhooks", "tags", "users"]
    for table in config_tables:
        cur.execute("""
            SELECT 1 FROM pg_trigger
            WHERE tgname = 'notify_change'
              AND tgrelid = %s::regclass
        """, (f"iiz.{table}",))
        assert cur.fetchone() is not None, f"No notify_change trigger on {table}"


def test_updated_at_triggers(db_conn):
    """Verify updated_at triggers exist on mutable tables."""
    cur = db_conn.cursor()
    mutable_tables = ["users", "queues", "call_annotations", "notifications"]
    for table in mutable_tables:
        cur.execute("""
            SELECT 1 FROM pg_trigger
            WHERE tgname = 'set_updated_at'
              AND tgrelid = %s::regclass
        """, (f"iiz.{table}",))
        assert cur.fetchone() is not None, f"No set_updated_at trigger on {table}"


def test_foreign_key_constraints(db_conn):
    """Verify key FK relationships exist."""
    cur = db_conn.cursor()
    # users.account_id -> accounts.id
    cur.execute("""
        SELECT 1 FROM information_schema.table_constraints
        WHERE table_schema = 'iiz'
          AND table_name = 'users'
          AND constraint_type = 'FOREIGN KEY'
    """)
    assert cur.fetchone() is not None, "No FK constraints on users table"
