use crate::codegen::drawdb_parser::parse_drawdb_file;
use crate::codegen::schema::{FieldModel, RelationshipModel, SchemaModel, TableModel};
use crate::codegen::sql::{map_sql_type, SqlDialect};
use crate::error::{MigrationError, Result};
use crate::parser::MigrationParser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDiff {
    pub old_field: FieldModel,
    pub new_field: FieldModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDiff {
    pub table_name: String,
    pub added_fields: Vec<FieldModel>,
    pub dropped_fields: Vec<FieldModel>,
    pub modified_fields: Vec<FieldDiff>,
}

impl TableDiff {
    pub fn is_empty(&self) -> bool {
        self.added_fields.is_empty()
            && self.dropped_fields.is_empty()
            && self.modified_fields.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaDiff {
    pub new_tables: Vec<TableModel>,
    pub dropped_tables: Vec<TableModel>,
    pub table_diffs: Vec<TableDiff>,
    pub new_relationships: Vec<RelationshipModel>,
    pub dropped_relationships: Vec<RelationshipModel>,
}

impl SchemaDiff {
    pub fn is_empty(&self) -> bool {
        self.new_tables.is_empty()
            && self.dropped_tables.is_empty()
            && self.new_relationships.is_empty()
            && self.dropped_relationships.is_empty()
            && self.table_diffs.iter().all(|td| td.is_empty())
    }

    pub fn summary(&self) -> String {
        if self.is_empty() {
            return "No schema changes detected.".to_string();
        }

        let mut lines = Vec::new();
        lines.push("Detected Schema Changes:".to_string());

        if !self.new_tables.is_empty() {
            let names: Vec<_> = self.new_tables.iter().map(|t| t.raw_name.as_str()).collect();
            lines.push(format!("  + New Table(s): {}", names.join(", ")));
        }

        if !self.dropped_tables.is_empty() {
            let names: Vec<_> = self.dropped_tables.iter().map(|t| t.raw_name.as_str()).collect();
            lines.push(format!("  - Dropped Table(s): {}", names.join(", ")));
        }

        for td in &self.table_diffs {
            if td.is_empty() {
                continue;
            }
            let mut parts = Vec::new();
            if !td.added_fields.is_empty() {
                let f_names: Vec<_> = td.added_fields.iter().map(|f| f.name.as_str()).collect();
                parts.push(format!("+{} column(s) ({})", td.added_fields.len(), f_names.join(", ")));
            }
            if !td.dropped_fields.is_empty() {
                let f_names: Vec<_> = td.dropped_fields.iter().map(|f| f.name.as_str()).collect();
                parts.push(format!("-{} column(s) ({})", td.dropped_fields.len(), f_names.join(", ")));
            }
            if !td.modified_fields.is_empty() {
                let f_names: Vec<_> = td.modified_fields.iter().map(|f| f.new_field.name.as_str()).collect();
                parts.push(format!("~{} modified column(s) ({})", td.modified_fields.len(), f_names.join(", ")));
            }
            lines.push(format!("  * {}: {}", td.table_name, parts.join(", ")));
        }

        if !self.new_relationships.is_empty() {
            lines.push(format!("  + {} new foreign key relation(s)", self.new_relationships.len()));
        }
        if !self.dropped_relationships.is_empty() {
            lines.push(format!("  - {} dropped foreign key relation(s)", self.dropped_relationships.len()));
        }

        lines.join("\n")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    pub success: bool,
    pub migration_path: Option<String>,
    pub diff_summary: String,
    pub is_empty: bool,
    pub statements: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPlan {
    pub success: bool,
    pub is_empty: bool,
    pub diff_summary: String,
    pub statements: Vec<String>,
    pub sql: String,
    pub dialect: String,
    pub error: Option<String>,
}

/// Finds the most relevant existing schema snapshot file.
pub fn find_snapshot(migrations_dir: &Path, schema_dir: &Path) -> Option<PathBuf> {
    let candidates = [
        migrations_dir.join(".schema_snapshot.json"),
        schema_dir.join("drawdb_snapshot.json"),
        migrations_dir.join("schema_snapshot.json"),
        PathBuf::from("schema").join("drawdb_snapshot.json"),
        PathBuf::from("migrations").join(".schema_snapshot.json"),
    ];

    for c in &candidates {
        if c.is_file() {
            return Some(c.clone());
        }
    }
    None
}

/// Saves the schema snapshot JSON to both schema/ and migrations/ directories, auto-creating them if needed.
pub fn save_snapshots(migrations_dir: &Path, schema_dir: &Path, schema_json_content: &str) -> Result<()> {
    if !schema_dir.exists() {
        fs::create_dir_all(schema_dir)?;
    }
    if !migrations_dir.exists() {
        fs::create_dir_all(migrations_dir)?;
    }

    let schema_target = schema_dir.join("drawdb_snapshot.json");
    fs::write(schema_target, schema_json_content)?;

    let migrations_target = migrations_dir.join(".schema_snapshot.json");
    fs::write(migrations_target, schema_json_content)?;

    Ok(())
}

fn is_field_modified(old: &FieldModel, new: &FieldModel) -> bool {
    if old.db_type != new.db_type {
        return true;
    }
    if old.size != new.size {
        return true;
    }
    if old.not_null != new.not_null {
        return true;
    }
    if old.default_value != new.default_value {
        return true;
    }
    if old.unique != new.unique {
        return true;
    }
    false
}

/// Calculates the structural difference between old and new schema models.
pub fn diff_schemas(old: &SchemaModel, new: &SchemaModel) -> SchemaDiff {
    let mut diff = SchemaDiff::default();

    let old_tables_by_name: HashMap<String, &TableModel> = old
        .tables
        .iter()
        .map(|t| (t.raw_name.to_lowercase(), t))
        .collect();

    let new_tables_by_name: HashMap<String, &TableModel> = new
        .tables
        .iter()
        .map(|t| (t.raw_name.to_lowercase(), t))
        .collect();

    // 1. Identify New and Modified Tables
    for new_t in &new::tables_list(new) {
        if let Some(old_t) = old_tables_by_name.get(&new_t.raw_name.to_lowercase()) {
            let td = diff_single_table(old_t, new_t);
            if !td.is_empty() {
                diff.table_diffs.push(td);
            }
        } else {
            diff.new_tables.push((*new_t).clone());
        }
    }

    // 2. Identify Dropped Tables
    for old_t in &old::tables_list(old) {
        if !new_tables_by_name.contains_key(&old_t.raw_name.to_lowercase()) {
            diff.dropped_tables.push((*old_t).clone());
        }
    }

    // 3. Identify Relationship Changes
    diff_relationships(&mut diff, old, new);

    diff
}

mod old {
    use super::*;
    pub fn tables_list(s: &SchemaModel) -> Vec<&TableModel> {
        s.tables.iter().collect()
    }
}

mod new {
    use super::*;
    pub fn tables_list(s: &SchemaModel) -> Vec<&TableModel> {
        s.tables.iter().collect()
    }
}

fn diff_single_table(old_t: &TableModel, new_t: &TableModel) -> TableDiff {
    let mut td = TableDiff {
        table_name: new_t.raw_name.clone(),
        added_fields: Vec::new(),
        dropped_fields: Vec::new(),
        modified_fields: Vec::new(),
    };

    let old_field_by_name: HashMap<String, &FieldModel> = old_t
        .fields
        .iter()
        .map(|f| (f.name.to_lowercase(), f))
        .collect();

    let new_field_by_name: HashMap<String, &FieldModel> = new_t
        .fields
        .iter()
        .map(|f| (f.name.to_lowercase(), f))
        .collect();

    // Added & Modified Fields
    for new_f in &new_t.fields {
        if let Some(old_f) = old_field_by_name.get(&new_f.name.to_lowercase()) {
            if is_field_modified(old_f, new_f) {
                td.modified_fields.push(FieldDiff {
                    old_field: (*old_f).clone(),
                    new_field: new_f.clone(),
                });
            }
        } else {
            td.added_fields.push(new_f.clone());
        }
    }

    // Dropped Fields
    for old_f in &old_t.fields {
        if !new_field_by_name.contains_key(&old_f.name.to_lowercase()) {
            td.dropped_fields.push(old_f.clone());
        }
    }

    td
}

fn rel_signature(schema: &SchemaModel, rel: &RelationshipModel) -> Option<String> {
    let parent_t = schema.find_table_by_id(&rel.parent_table_id)?;
    let child_t = schema.find_table_by_id(&rel.child_table_id)?;
    let parent_f = schema.find_field(&rel.parent_table_id, &rel.parent_field_id)?;
    let child_f = schema.find_field(&rel.child_table_id, &rel.child_field_id)?;

    Some(format!(
        "{}({})->{}({})",
        child_t.raw_name.to_lowercase(),
        child_f.name.to_lowercase(),
        parent_t.raw_name.to_lowercase(),
        parent_f.name.to_lowercase()
    ))
}

fn diff_relationships(diff: &mut SchemaDiff, old: &SchemaModel, new: &SchemaModel) {
    let mut old_rels: HashMap<String, &RelationshipModel> = HashMap::new();
    for r in &old.relationships {
        if let Some(sig) = rel_signature(old, r) {
            old_rels.insert(sig, r);
        }
    }

    let mut new_rels: HashMap<String, &RelationshipModel> = HashMap::new();
    for r in &new.relationships {
        if let Some(sig) = rel_signature(new, r) {
            new_rels.insert(sig, r);
        }
    }

    for (sig, r) in &new_rels {
        if !old_rels.contains_key(sig) {
            diff.new_relationships.push((*r).clone());
        }
    }

    for (sig, r) in &old_rels {
        if !new_rels.contains_key(sig) {
            diff.dropped_relationships.push((*r).clone());
        }
    }
}

fn quote_ident(ident: &str, dialect: SqlDialect) -> String {
    match dialect {
        SqlDialect::MySql => format!("`{}`", ident),
        _ => format!("\"{}\"", ident),
    }
}

fn format_col_definition(field: &FieldModel, dialect: SqlDialect) -> String {
    let q_name = quote_ident(&field.name, dialect);
    let mapped_type = map_sql_type(&field.db_type, field.size.as_deref(), field.primary, dialect);

    let mut def = format!("{} {}", q_name, mapped_type);
    if field.not_null && !field.primary {
        def.push_str(" NOT NULL");
    }
    if field.unique && !field.primary {
        def.push_str(" UNIQUE");
    }
    if let Some(val) = &field.default_value {
        if !val.is_empty() {
            def.push_str(&format!(" DEFAULT {}", val));
        }
    }
    def
}

fn generate_table_sql(table: &TableModel, schema: &SchemaModel, dialect: SqlDialect) -> (String, Vec<String>) {
    let q_tbl = quote_ident(&table.raw_name, dialect);
    let mut col_defs = Vec::new();
    let mut primary_keys = Vec::new();

    for f in &table.fields {
        if f.primary {
            primary_keys.push(quote_ident(&f.name, dialect));
        }
        col_defs.push(format_col_definition(f, dialect));
    }

    if !primary_keys.is_empty() {
        if dialect == SqlDialect::Postgres || dialect == SqlDialect::MySql || primary_keys.len() > 1 {
            col_defs.push(format!("PRIMARY KEY ({})", primary_keys.join(", ")));
        }
    }

    // Inline Foreign Keys on table creation for SQLite
    if dialect == SqlDialect::Sqlite {
        let incoming = schema.incoming_relationships(&table.id);
        for rel in incoming {
            if let (Some(parent_tbl), Some(child_fld), Some(parent_fld)) = (
                schema.find_table_by_id(&rel.parent_table_id),
                schema.find_field(&table.id, &rel.child_field_id),
                schema.find_field(&rel.parent_table_id, &rel.parent_field_id),
            ) {
                let fk_name = format!("fk_{}_{}", table.raw_name, child_fld.name);
                col_defs.push(format!(
                    "CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE CASCADE ON UPDATE CASCADE",
                    quote_ident(&fk_name, dialect),
                    quote_ident(&child_fld.name, dialect),
                    quote_ident(&parent_tbl.raw_name, dialect),
                    quote_ident(&parent_fld.name, dialect)
                ));
            }
        }
    }

    let mut stmt = format!("CREATE TABLE IF NOT EXISTS {} (\n  {}\n)", q_tbl, col_defs.join(",\n  "));
    if dialect == SqlDialect::MySql {
        stmt.push_str(" ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;");
    } else {
        stmt.push(';');
    }

    let mut statements = vec![stmt.clone()];

    // Separate Foreign Keys for Postgres and MySQL
    if dialect != SqlDialect::Sqlite {
        let incoming = schema.incoming_relationships(&table.id);
        for rel in incoming {
            if let (Some(parent_tbl), Some(child_fld), Some(parent_fld)) = (
                schema.find_table_by_id(&rel.parent_table_id),
                schema.find_field(&table.id, &rel.child_field_id),
                schema.find_field(&rel.parent_table_id, &rel.parent_field_id),
            ) {
                let fk_name = format!("fk_{}_{}", table.raw_name, child_fld.name);
                let fk_sql = format!(
                    "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE CASCADE ON UPDATE CASCADE;",
                    q_tbl,
                    quote_ident(&fk_name, dialect),
                    quote_ident(&child_fld.name, dialect),
                    quote_ident(&parent_tbl.raw_name, dialect),
                    quote_ident(&parent_fld.name, dialect)
                );
                statements.push(fk_sql);
            }
        }
    }

    (stmt, statements)
}

/// Generates SQL DDL statements and a formatted script representing the schema diff for the target dialect.
pub fn generate_diff_sql(
    diff: &SchemaDiff,
    old_schema: Option<&SchemaModel>,
    new_schema: &SchemaModel,
    dialect: SqlDialect,
) -> (String, Vec<String>) {
    if diff.is_empty() {
        return ("-- No schema changes detected.\n".to_string(), Vec::new());
    }

    let mut script = String::new();
    let mut stmts = Vec::new();

    script.push_str("-- -----------------------------------------------------------------------------\n");
    script.push_str("-- Incremental Migration Generated by MigraDB\n");
    script.push_str(&format!("-- Dialect: {:?}\n", dialect));
    script.push_str("-- -----------------------------------------------------------------------------\n\n");

    // 1. New Tables
    if !diff.new_tables.is_empty() {
        script.push_str("-- 1. New Tables\n");
        for t in &diff.new_tables {
            let (_, table_stmts) = generate_table_sql(t, new_schema, dialect);
            for s in table_stmts {
                script.push_str(&format!("{}\n\n", s));
                stmts.push(s);
            }
        }
    }

    // 2. Alter Existing Tables
    if !diff.table_diffs.is_empty() {
        script.push_str("-- 2. Alter Existing Tables\n");
        for td in &diff.table_diffs {
            if td.is_empty() {
                continue;
            }
            let q_tbl = quote_ident(&td.table_name, dialect);

            // Added Columns
            for col in &td.added_fields {
                let col_def = format_col_definition(col, dialect);
                let stmt = format!("ALTER TABLE {} ADD COLUMN {};", q_tbl, col_def);
                script.push_str(&format!("{}\n", stmt));
                stmts.push(stmt);
            }

            // Modified Columns
            for mf in &td.modified_fields {
                let q_col = quote_ident(&mf.new_field.name, dialect);
                match dialect {
                    SqlDialect::Postgres => {
                        let new_type = map_sql_type(&mf.new_field.db_type, mf.new_field.size.as_deref(), mf.new_field.primary, dialect);
                        let stmt = format!("ALTER TABLE {} ALTER COLUMN {} TYPE {};", q_tbl, q_col, new_type);
                        script.push_str(&format!("{}\n", stmt));
                        stmts.push(stmt);

                        if mf.old_field.not_null != mf.new_field.not_null {
                            let null_action = if mf.new_field.not_null { "SET NOT NULL" } else { "DROP NOT NULL" };
                            let n_stmt = format!("ALTER TABLE {} ALTER COLUMN {} {};", q_tbl, q_col, null_action);
                            script.push_str(&format!("{}\n", n_stmt));
                            stmts.push(n_stmt);
                        }

                        if mf.old_field.default_value != mf.new_field.default_value {
                            if let Some(d) = &mf.new_field.default_value {
                                let d_stmt = format!("ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {};", q_tbl, q_col, d);
                                script.push_str(&format!("{}\n", d_stmt));
                                stmts.push(d_stmt);
                            } else {
                                let d_stmt = format!("ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;", q_tbl, q_col);
                                script.push_str(&format!("{}\n", d_stmt));
                                stmts.push(d_stmt);
                            }
                        }
                    }
                    SqlDialect::MySql => {
                        let col_def = format_col_definition(&mf.new_field, dialect);
                        let stmt = format!("ALTER TABLE {} MODIFY COLUMN {};", q_tbl, col_def);
                        script.push_str(&format!("{}\n", stmt));
                        stmts.push(stmt);
                    }
                    SqlDialect::Sqlite => {
                        let comment = format!("-- [Notice] SQLite does not support ALTER COLUMN TYPE for {}.{}. Recreate table if type changed.", td.table_name, mf.new_field.name);
                        script.push_str(&format!("{}\n", comment));
                    }
                }
            }

            // Dropped Columns
            for col in &td.dropped_fields {
                let q_col = quote_ident(&col.name, dialect);
                let stmt = match dialect {
                    SqlDialect::Postgres => format!("ALTER TABLE {} DROP COLUMN IF EXISTS {};", q_tbl, q_col),
                    _ => format!("ALTER TABLE {} DROP COLUMN {};", q_tbl, q_col),
                };
                script.push_str(&format!("{}\n", stmt));
                stmts.push(stmt);
            }

            script.push('\n');
        }
    }

    // 3. Foreign Key Changes
    if dialect != SqlDialect::Sqlite {
        // Dropped relationships
        if !diff.dropped_relationships.is_empty() {
            script.push_str("-- 3. Dropped Foreign Keys\n");
            for rel in &diff.dropped_relationships {
                if let (Some(_old_s), Some(child_tbl), Some(child_fld)) = (
                    old_schema,
                    old_schema.and_then(|s| s.find_table_by_id(&rel.child_table_id)),
                    old_schema.and_then(|s| s.find_field(&rel.child_table_id, &rel.child_field_id)),
                ) {
                    let fk_name = format!("fk_{}_{}", child_tbl.raw_name, child_fld.name);
                    let q_tbl = quote_ident(&child_tbl.raw_name, dialect);
                    let q_fk = quote_ident(&fk_name, dialect);
                    let stmt = match dialect {
                        SqlDialect::MySql => format!("ALTER TABLE {} DROP FOREIGN KEY {};", q_tbl, q_fk),
                        _ => format!("ALTER TABLE {} DROP CONSTRAINT IF EXISTS {};", q_tbl, q_fk),
                    };
                    script.push_str(&format!("{}\n", stmt));
                    stmts.push(stmt);
                }
            }
            script.push('\n');
        }

        // New relationships
        if !diff.new_relationships.is_empty() {
            script.push_str("-- 4. New Foreign Keys\n");
            for rel in &diff.new_relationships {
                if let (Some(parent_tbl), Some(child_tbl), Some(parent_fld), Some(child_fld)) = (
                    new_schema.find_table_by_id(&rel.parent_table_id),
                    new_schema.find_table_by_id(&rel.child_table_id),
                    new_schema.find_field(&rel.parent_table_id, &rel.parent_field_id),
                    new_schema.find_field(&rel.child_table_id, &rel.child_field_id),
                ) {
                    let fk_name = format!("fk_{}_{}", child_tbl.raw_name, child_fld.name);
                    let stmt = format!(
                        "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE CASCADE ON UPDATE CASCADE;",
                        quote_ident(&child_tbl.raw_name, dialect),
                        quote_ident(&fk_name, dialect),
                        quote_ident(&child_fld.name, dialect),
                        quote_ident(&parent_tbl.raw_name, dialect),
                        quote_ident(&parent_fld.name, dialect)
                    );
                    script.push_str(&format!("{}\n", stmt));
                    stmts.push(stmt);
                }
            }
            script.push('\n');
        }
    }

    // 4. Dropped Tables
    if !diff.dropped_tables.is_empty() {
        script.push_str("-- 5. Dropped Tables\n");
        for t in &diff.dropped_tables {
            let q_tbl = quote_ident(&t.raw_name, dialect);
            let stmt = match dialect {
                SqlDialect::Postgres => format!("DROP TABLE IF EXISTS {} CASCADE;", q_tbl),
                _ => format!("DROP TABLE IF EXISTS {};", q_tbl),
            };
            script.push_str(&format!("{}\n", stmt));
            stmts.push(stmt);
        }
        script.push('\n');
    }

    (script, stmts)
}

/// Generates a full baseline schema DDL for a schema model in the given dialect.
pub fn generate_baseline_sql(schema: &SchemaModel, dialect: SqlDialect) -> (String, Vec<String>) {
    let mut script = String::new();
    let mut stmts = Vec::new();

    script.push_str("-- -----------------------------------------------------------------------------\n");
    script.push_str("-- Baseline Schema DDL Generated by MigraDB\n");
    script.push_str(&format!("-- Dialect: {:?}\n", dialect));
    script.push_str("-- -----------------------------------------------------------------------------\n\n");

    if dialect == SqlDialect::Postgres {
        let ext = "CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";".to_string();
        script.push_str(&format!("{}\n\n", ext));
        stmts.push(ext);
    } else if dialect == SqlDialect::Sqlite {
        let pragma = "PRAGMA foreign_keys = ON;".to_string();
        script.push_str(&format!("{}\n\n", pragma));
        stmts.push(pragma);
    }

    for t in &schema.tables {
        let (_, table_stmts) = generate_table_sql(t, schema, dialect);
        for s in table_stmts {
            script.push_str(&format!("{}\n\n", s));
            stmts.push(s);
        }
    }

    (script, stmts)
}

/// Prepares a dynamic execution plan against DrawDB for application runners.
pub fn plan_sync_drawdb<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    schema_path: P,
    migrations_dir: Q,
    schema_dir: R,
    dialect_str: Option<&str>,
    force_full: bool,
) -> Result<SyncPlan> {
    let schema_ref = schema_path.as_ref();
    let mig_ref = migrations_dir.as_ref();
    let sch_ref = schema_dir.as_ref();

    if !schema_ref.exists() {
        return Err(MigrationError::Codegen(format!(
            "DrawDB schema file not found: {}",
            schema_ref.display()
        )));
    }

    let dialect = SqlDialect::parse(dialect_str.unwrap_or("postgres"));
    let new_schema = parse_drawdb_file(schema_ref)?;
    let snapshot_file = find_snapshot(mig_ref, sch_ref);

    if force_full || snapshot_file.is_none() {
        let (sql, stmts) = generate_baseline_sql(&new_schema, dialect);
        let summary = format!("Initial baseline schema with {} table(s).", new_schema.tables.len());
        return Ok(SyncPlan {
            success: true,
            is_empty: stmts.is_empty(),
            diff_summary: summary,
            statements: stmts,
            sql,
            dialect: format!("{:?}", dialect).to_lowercase(),
            error: None,
        });
    }

    let snap_path = snapshot_file.unwrap();
    let old_schema = parse_drawdb_file(&snap_path)?;
    let diff = diff_schemas(&old_schema, &new_schema);

    if diff.is_empty() {
        return Ok(SyncPlan {
            success: true,
            is_empty: true,
            diff_summary: "No schema changes detected.".to_string(),
            statements: Vec::new(),
            sql: "-- No schema changes detected.\n".to_string(),
            dialect: format!("{:?}", dialect).to_lowercase(),
            error: None,
        });
    }

    let (sql, stmts) = generate_diff_sql(&diff, Some(&old_schema), &new_schema, dialect);
    Ok(SyncPlan {
        success: true,
        is_empty: stmts.is_empty(),
        diff_summary: diff.summary(),
        statements: stmts,
        sql,
        dialect: format!("{:?}", dialect).to_lowercase(),
        error: None,
    })
}

/// Calculates diff, creates migration file, and saves snapshot JSON to both directories.
pub fn diff_drawdb<P: AsRef<Path>, Q: AsRef<Path>, R: AsRef<Path>>(
    schema_path: P,
    migrations_dir: Q,
    schema_dir: R,
    migration_name: &str,
    dialect_str: Option<&str>,
    force_full: bool,
) -> Result<DiffResult> {
    let schema_ref = schema_path.as_ref();
    let mig_ref = migrations_dir.as_ref();
    let sch_ref = schema_dir.as_ref();

    let plan = plan_sync_drawdb(schema_ref, mig_ref, sch_ref, dialect_str, force_full)?;

    if plan.is_empty {
        return Ok(DiffResult {
            success: true,
            migration_path: None,
            diff_summary: plan.diff_summary,
            is_empty: true,
            statements: Vec::new(),
            error: None,
        });
    }

    // Auto-create directories if missing
    if !mig_ref.exists() {
        fs::create_dir_all(mig_ref)?;
    }
    if !sch_ref.exists() {
        fs::create_dir_all(sch_ref)?;
    }

    let clean_name = if migration_name.trim().is_empty() {
        "sync_drawdb"
    } else {
        migration_name.trim()
    };

    let mig_file = MigrationParser::create_migration_file(mig_ref, clean_name, &plan.sql)?;

    // Read the new schema JSON content and persist snapshot to schema/ and migrations/
    let new_content = fs::read_to_string(schema_ref)?;
    save_snapshots(mig_ref, sch_ref, &new_content)?;

    Ok(DiffResult {
        success: true,
        migration_path: Some(mig_file.to_string_lossy().to_string()),
        diff_summary: plan.diff_summary,
        is_empty: false,
        statements: plan.statements,
        error: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codegen::schema::*;

    fn make_test_table(name: &str, cols: &[(&str, &str, bool, bool)]) -> TableModel {
        TableModel {
            id: format!("tbl_{}", name),
            raw_name: name.to_string(),
            entity_name: name.to_string(),
            fields: cols
                .iter()
                .enumerate()
                .map(|(i, &(c_name, c_type, is_pk, not_null))| FieldModel {
                    id: format!("f_{}_{}", name, i),
                    name: c_name.to_string(),
                    db_type: c_type.to_string(),
                    size: if c_type == "VARCHAR" { Some("255".to_string()) } else { None },
                    primary: is_pk,
                    not_null,
                    unique: false,
                    default_value: None,
                    comment: None,
                })
                .collect(),
            comment: None,
        }
    }

    #[test]
    fn test_diff_empty() {
        let t1 = make_test_table("users", &[("id", "BIGINT", true, true), ("email", "VARCHAR", false, true)]);
        let s1 = SchemaModel {
            tables: vec![t1.clone()],
            relationships: vec![],
        };
        let s2 = SchemaModel {
            tables: vec![t1],
            relationships: vec![],
        };

        let diff = diff_schemas(&s1, &s2);
        assert!(diff.is_empty());
    }

    #[test]
    fn test_diff_added_column_postgres() {
        let t1 = make_test_table("users", &[("id", "BIGINT", true, true)]);
        let t2 = make_test_table(
            "users",
            &[("id", "BIGINT", true, true), ("email", "VARCHAR", false, true)],
        );

        let s1 = SchemaModel { tables: vec![t1], relationships: vec![] };
        let s2 = SchemaModel { tables: vec![t2], relationships: vec![] };

        let diff = diff_schemas(&s1, &s2);
        assert!(!diff.is_empty());
        assert_eq!(diff.table_diffs.len(), 1);
        assert_eq!(diff.table_diffs[0].added_fields.len(), 1);
        assert_eq!(diff.table_diffs[0].added_fields[0].name, "email");

        let (sql, stmts) = generate_diff_sql(&diff, Some(&s1), &s2, SqlDialect::Postgres);
        assert!(sql.contains("ALTER TABLE \"users\" ADD COLUMN \"email\" VARCHAR(255) NOT NULL;"));
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn test_diff_added_column_mysql() {
        let t1 = make_test_table("users", &[("id", "BIGINT", true, true)]);
        let t2 = make_test_table(
            "users",
            &[("id", "BIGINT", true, true), ("email", "VARCHAR", false, true)],
        );

        let s1 = SchemaModel { tables: vec![t1], relationships: vec![] };
        let s2 = SchemaModel { tables: vec![t2], relationships: vec![] };

        let diff = diff_schemas(&s1, &s2);
        let (sql, stmts) = generate_diff_sql(&diff, Some(&s1), &s2, SqlDialect::MySql);
        assert!(sql.contains("ALTER TABLE `users` ADD COLUMN `email` VARCHAR(255) NOT NULL;"));
        assert_eq!(stmts.len(), 1);
    }

    #[test]
    fn test_diff_modify_column_mysql() {
        let t1 = make_test_table("users", &[("id", "BIGINT", true, true), ("status", "VARCHAR", false, false)]);
        let mut t2 = make_test_table("users", &[("id", "BIGINT", true, true), ("status", "VARCHAR", false, true)]);
        t2.fields[1].size = Some("50".to_string());

        let s1 = SchemaModel { tables: vec![t1], relationships: vec![] };
        let s2 = SchemaModel { tables: vec![t2], relationships: vec![] };

        let diff = diff_schemas(&s1, &s2);
        assert!(!diff.is_empty());
        assert_eq!(diff.table_diffs[0].modified_fields.len(), 1);

        let (sql, stmts) = generate_diff_sql(&diff, Some(&s1), &s2, SqlDialect::MySql);
        assert!(sql.contains("ALTER TABLE `users` MODIFY COLUMN `status` VARCHAR(50) NOT NULL;"));
        assert_eq!(stmts.len(), 1);
    }
}
