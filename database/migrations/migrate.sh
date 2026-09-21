#!/bin/sh

schemaVersionTable=version_schema
functionsVersionTable=version_functions

# Schema migrations such as event co-hosting compile SQL functions that call
# user_has_group_permission. On a fresh database that helper is loaded later,
# so install a typed stub first. The real function replaces it below.
if [ -n "$TERN_CONF" ] && [ -f "$TERN_CONF" ]; then
    echo "- Installing permission helper stub.."
    tern_db_field() {
        sed -n '/^\[database\]/,/^\[/p' "$TERN_CONF" \
            | sed -n "s/^$1[[:space:]]*=[[:space:]]*//p" \
            | sed 's/^["'\'']//;s/["'\'']$//' \
            | head -n 1
    }
    db_host=$(tern_db_field host)
    db_port=$(tern_db_field port)
    db_name=$(tern_db_field database)
    db_user=$(tern_db_field user)
    db_password=$(tern_db_field password)
    PGPASSWORD="$db_password" psql -h "$db_host" -p "$db_port" -U "$db_user" -d "$db_name" -v ON_ERROR_STOP=1 -c \
        "create or replace function user_has_group_permission(uuid, uuid, uuid, text) returns boolean language sql as \$\$ select false; \$\$;"
    if [ $? -ne 0 ]; then exit 1; fi
    echo "Done"
fi

echo "- Applying schema migrations.."
cd schema
tern status --config $TERN_CONF --version-table $schemaVersionTable
tern migrate --config $TERN_CONF --version-table $schemaVersionTable
if [ $? -ne 0 ]; then exit 1; fi
echo "Done"
cd ..

echo "- Loading functions.."
cd functions
tern status --config $TERN_CONF --version-table $functionsVersionTable | grep "version:  1 of 1"
if [ $? -eq 0 ]; then
    tern migrate --config $TERN_CONF --version-table $functionsVersionTable --destination -+1
else
    tern migrate --config $TERN_CONF --version-table $functionsVersionTable
fi
if [ $? -ne 0 ]; then exit 1; fi
echo "Done"
