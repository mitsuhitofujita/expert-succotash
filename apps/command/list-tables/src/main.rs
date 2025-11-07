use anyhow::{Context, Result};
use sqlx::Row;
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    println!("=== Database Tables List ===\n");

    // Get DATABASE_URL from environment
    let database_url =
        std::env::var("DATABASE_URL").context("DATABASE_URL environment variable is not set")?;

    println!("Connecting to database...\n");

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .context("Failed to connect to database")?;

    // Query to get all user tables with their column information
    let query = r#"
        SELECT
            c.table_name,
            c.column_name,
            c.data_type,
            c.character_maximum_length,
            c.is_nullable,
            c.column_default,
            col_description(('"' || c.table_schema || '"."' || c.table_name || '"')::regclass, c.ordinal_position) as column_comment
        FROM
            information_schema.columns c
        JOIN
            information_schema.tables t
            ON c.table_name = t.table_name
            AND c.table_schema = t.table_schema
        WHERE
            t.table_type = 'BASE TABLE'
            AND c.table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY
            c.table_name, c.ordinal_position
    "#;

    let rows = sqlx::query(query)
        .fetch_all(&pool)
        .await
        .context("Failed to fetch table information")?;

    if rows.is_empty() {
        println!("No user tables found in the database.");
    } else {
        let mut current_table = String::new();

        for row in rows {
            let table_name: String = row.get("table_name");
            let column_name: String = row.get("column_name");
            let data_type: String = row.get("data_type");
            let char_max_length: Option<i32> = row.get("character_maximum_length");
            let is_nullable: String = row.get("is_nullable");
            let column_default: Option<String> = row.get("column_default");
            let column_comment: Option<String> = row.get("column_comment");

            // Print table header when we encounter a new table
            if table_name != current_table {
                if !current_table.is_empty() {
                    println!(); // Add blank line between tables
                }

                // Get table comment
                let table_comment_query =
                    format!("SELECT obj_description('{table_name}'::regclass)");
                let table_comment: Option<String> = sqlx::query_scalar(&table_comment_query)
                    .fetch_one(&pool)
                    .await
                    .ok()
                    .flatten();

                println!("Table: {table_name}");
                if let Some(comment) = table_comment {
                    println!("  Description: {comment}");
                }
                println!("  Columns:");

                current_table = table_name;
            }

            // Format data type with length if applicable
            let full_type = if let Some(length) = char_max_length {
                format!("{data_type}({length})")
            } else {
                data_type
            };

            // Format nullable
            let nullable = if is_nullable == "YES" {
                "NULL"
            } else {
                "NOT NULL"
            };

            // Format default value
            let default = column_default
                .as_ref()
                .map_or(String::new(), |d| format!(" DEFAULT {d}"));

            // Format comment
            let comment = column_comment
                .as_ref()
                .map_or(String::new(), |c| format!(" -- {c}"));

            println!("    - {column_name}: {full_type} {nullable}{default}{comment}");
        }
    }

    println!();

    // Get table constraints (primary keys, unique constraints, foreign keys)
    let constraints_query = r"
        SELECT
            tc.table_name,
            tc.constraint_name,
            tc.constraint_type,
            kcu.column_name
        FROM
            information_schema.table_constraints tc
        LEFT JOIN
            information_schema.key_column_usage kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        WHERE
            tc.table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY
            tc.table_name, tc.constraint_type, kcu.ordinal_position
    ";

    let constraint_rows = sqlx::query(constraints_query)
        .fetch_all(&pool)
        .await
        .context("Failed to fetch constraints")?;

    if !constraint_rows.is_empty() {
        println!("Constraints:");
        let mut current_table = String::new();

        for row in constraint_rows {
            let table_name: String = row.get("table_name");
            let constraint_name: String = row.get("constraint_name");
            let constraint_type: String = row.get("constraint_type");
            let column_name: Option<String> = row.get("column_name");

            if table_name != current_table {
                if !current_table.is_empty() {
                    println!();
                }
                println!("  {table_name}:");
                current_table = table_name;
            }

            let column_info = column_name
                .as_ref()
                .map_or(String::new(), |c| format!(" ({c})"));
            println!("    - {constraint_type}: {constraint_name}{column_info}");
        }
    }

    // Get indexes
    println!();
    let indexes_query = r"
        SELECT
            tablename,
            indexname,
            indexdef
        FROM
            pg_indexes
        WHERE
            schemaname NOT IN ('pg_catalog', 'information_schema')
        ORDER BY
            tablename, indexname
    ";

    let index_rows = sqlx::query(indexes_query)
        .fetch_all(&pool)
        .await
        .context("Failed to fetch indexes")?;

    if !index_rows.is_empty() {
        println!("Indexes:");
        let mut current_table = String::new();

        for row in index_rows {
            let table_name: String = row.get("tablename");
            let index_name: String = row.get("indexname");
            let index_def: String = row.get("indexdef");

            if table_name != current_table {
                if !current_table.is_empty() {
                    println!();
                }
                println!("  {table_name}:");
                current_table = table_name;
            }

            println!("    - {index_name}");
            println!("      {index_def}");
        }
    }

    println!();

    // Close the connection
    pool.close().await;

    Ok(())
}
