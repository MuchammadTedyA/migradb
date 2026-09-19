use crate::codegen::naming::clean_entity_name;
use crate::codegen::schema::{Cardinality, FieldModel, RelationshipModel, SchemaModel, TableModel};
use crate::error::{MigrationError, Result};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct DrawDBField {
    id: String,
    name: String,
    #[serde(rename = "type")]
    field_type: String,
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    primary: bool,
    #[serde(default, rename = "notNull")]
    not_null: bool,
    #[serde(default)]
    unique: bool,
    #[serde(default, rename = "default")]
    default_val: Option<String>,
    #[serde(default)]
    comment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DrawDBTable {
    id: String,
    name: String,
    #[serde(default)]
    fields: Vec<DrawDBField>,
    #[serde(default)]
    comment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DrawDBRelationship {
    id: String,
    #[serde(default)]
    name: String,
    #[serde(rename = "startTableId")]
    start_table_id: String,
    #[serde(rename = "endTableId")]
    end_table_id: String,
    #[serde(default, rename = "startFieldId")]
    start_field_id: Option<String>,
    #[serde(default, rename = "endFieldId")]
    end_field_id: Option<String>,
    #[serde(default)]
    cardinality: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DrawDBExport {
    #[serde(default)]
    tables: Vec<DrawDBTable>,
    #[serde(default)]
    relationships: Vec<DrawDBRelationship>,
}

pub fn parse_drawdb_file<P: AsRef<Path>>(path: P) -> Result<SchemaModel> {
    let path_ref = path.as_ref();
    if !path_ref.exists() {
        return Err(MigrationError::Codegen(format!(
            "DrawDB file not found: {}",
            path_ref.display()
        )));
    }

    let content = fs::read_to_string(path_ref)?;
    parse_drawdb_json(&content)
}

pub fn parse_drawdb_json(json_str: &str) -> Result<SchemaModel> {
    let export: DrawDBExport = serde_json::from_str(json_str).map_err(|e| {
        MigrationError::Codegen(format!("Failed to parse DrawDB JSON: {}", e))
    })?;

    let mut tables = Vec::new();

    for t in export.tables {
        let entity_name = clean_entity_name(&t.name);
        let fields = t
            .fields
            .into_iter()
            .map(|f| FieldModel {
                id: f.id,
                name: f.name,
                db_type: f.field_type.to_uppercase(),
                size: f.size.filter(|s| !s.is_empty()),
                primary: f.primary,
                // Primary keys are always considered not null
                not_null: f.not_null || f.primary,
                unique: f.unique,
                default_value: f.default_val.filter(|d| !d.is_empty()),
                comment: f.comment.filter(|c| !c.is_empty()),
            })
            .collect();

        tables.push(TableModel {
            id: t.id,
            raw_name: t.name,
            entity_name,
            fields,
            comment: t.comment.filter(|c| !c.is_empty()),
        });
    }

    let mut relationships = Vec::new();

    for r in export.relationships {
        let cardinality = match r.cardinality.as_deref() {
            Some("one_to_one") => Cardinality::OneToOne,
            Some("many_to_many") => Cardinality::ManyToMany,
            _ => Cardinality::OneToMany,
        };

        let start_field = r.start_field_id.unwrap_or_default();
        let end_field = r.end_field_id.unwrap_or_default();

        if !start_field.is_empty() && !end_field.is_empty() {
            relationships.push(RelationshipModel {
                id: r.id,
                name: r.name,
                parent_table_id: r.start_table_id,
                child_table_id: r.end_table_id,
                parent_field_id: start_field,
                child_field_id: end_field,
                cardinality,
            });
        }
    }

    Ok(SchemaModel {
        tables,
        relationships,
    })
}

