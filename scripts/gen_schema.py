#!/usr/bin/env python3
"""Generate Diesel schema.rs from PostgreSQL iiz schema metadata."""

import sys

# Map PostgreSQL types to Diesel SQL types
TYPE_MAP = {
    'uuid': 'Uuid',
    'text': 'Text',
    'varchar': 'Text',
    'int4': 'Int4',
    'int8': 'Int8',
    'int2': 'Int2',
    'float4': 'Float4',
    'float8': 'Float8',
    'bool': 'Bool',
    'numeric': 'Numeric',
    'timestamptz': 'Timestamptz',
    'timestamp': 'Timestamp',
    'date': 'Date',
    'jsonb': 'Jsonb',
    'json': 'Json',
    'bytea': 'Bytea',
    'inet': 'Inet',
    'interval': 'Interval',
    '_text': 'Array<Text>',  # text[]
}

def main():
    input_file = sys.argv[1]
    output_file = sys.argv[2]
    pk_file = sys.argv[3] if len(sys.argv) > 3 else None

    # Read primary key data
    pk_map = {}
    if pk_file:
        with open(pk_file) as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                parts = line.split('|')
                if len(parts) == 2:
                    pk_map[parts[0]] = parts[1]

    # Read column data
    tables = {}
    with open(input_file) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            parts = line.split('|')
            if len(parts) != 4:
                continue
            table, col, udt, nullable = parts
            if table not in tables:
                tables[table] = []
            tables[table].append((col, udt, nullable == 'YES'))

    # Collect all custom enum types used
    enum_types = set()
    for table, cols in tables.items():
        for col, udt, nullable in cols:
            if udt not in TYPE_MAP and udt != 'vector':
                enum_types.add(udt)

    lines = []
    lines.append('//! Diesel schema definitions for the iiz PostgreSQL schema.')
    lines.append('//!')
    lines.append('//! Generated from live database. Regenerate with diesel print-schema')
    lines.append('//! or by running: python3 scripts/gen_schema.py')
    lines.append('')
    lines.append('// Custom SQL types for PostgreSQL enums')
    lines.append('#[allow(non_camel_case_types)]')
    lines.append('pub mod sql_types {')
    lines.append('    use diesel::sql_types::SqlType;')
    lines.append('')
    for enum_name in sorted(enum_types):
        rust_name = ''.join(word.capitalize() for word in enum_name.split('_'))
        lines.append(f'    #[derive(SqlType, Debug, Clone, Copy)]')
        lines.append(f'    #[diesel(postgres_type(name = "{enum_name}"))]')
        lines.append(f'    pub struct {rust_name};')
        lines.append('')
    # pgvector type
    lines.append('    #[derive(SqlType, Debug, Clone, Copy)]')
    lines.append('    #[diesel(postgres_type(name = "vector"))]')
    lines.append('    pub struct Vector;')
    lines.append('}')
    lines.append('')

    # Generate table! macros
    for table in sorted(tables.keys()):
        cols = tables[table]
        lines.append('diesel::table! {')

        # Check if any custom types are used
        custom_used = set()
        for col, udt, nullable in cols:
            if udt not in TYPE_MAP:
                if udt == 'vector':
                    custom_used.add('Vector')
                else:
                    rust_name = ''.join(word.capitalize() for word in udt.split('_'))
                    custom_used.add(rust_name)
        if custom_used:
            lines.append('    use diesel::sql_types::*;')
            imports = ', '.join(sorted(custom_used))
            lines.append(f'    use super::sql_types::{{{imports}}};')
            lines.append('')

        pk_cols = pk_map.get(table, 'id')
        lines.append(f'    iiz.{table} ({pk_cols}) {{')
        for col, udt, nullable in cols:
            # Map type
            if udt == 'vector':
                diesel_type = 'Vector'
            elif udt in TYPE_MAP:
                diesel_type = TYPE_MAP[udt]
            else:
                diesel_type = ''.join(word.capitalize() for word in udt.split('_'))

            if nullable:
                diesel_type = f'Nullable<{diesel_type}>'

            lines.append(f'        {col} -> {diesel_type},')

        lines.append('    }')
        lines.append('}')
        lines.append('')

    lines.append('// Joinable declarations will be added as relationships are implemented.')
    lines.append('// Use diesel::allow_tables_to_query! for cross-table queries as needed.')
    lines.append('')

    with open(output_file, 'w') as f:
        f.write('\n'.join(lines))

    print(f"Generated {len(tables)} table definitions with {sum(len(c) for c in tables.values())} columns")

if __name__ == '__main__':
    main()
