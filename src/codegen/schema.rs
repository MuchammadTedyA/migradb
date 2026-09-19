use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Cardinality {
    OneToOne,
    OneToMany,
    ManyToMany,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldModel {
    pub id: String,
    pub name: String,
    pub db_type: String,
    pub size: Option<String>,
    pub primary: bool,
    pub not_null: bool,
    pub unique: bool,
    pub default_value: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableModel {
    pub id: String,
    pub raw_name: String,
    pub entity_name: String,
    pub fields: Vec<FieldModel>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipModel {
    pub id: String,
    pub name: String,
    pub parent_table_id: String,
    pub child_table_id: String,
    pub parent_field_id: String,
    pub child_field_id: String,
    pub cardinality: Cardinality,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaModel {
    pub tables: Vec<TableModel>,
    pub relationships: Vec<RelationshipModel>,
}

impl SchemaModel {
    pub fn find_table_by_id(&self, id: &str) -> Option<&TableModel> {
        self.tables.iter().find(|t| t.id == id)
    }

    pub fn find_table_by_raw_name(&self, name: &str) -> Option<&TableModel> {
        self.tables.iter().find(|t| t.raw_name == name)
    }

    pub fn find_field<'a>(&'a self, table_id: &str, field_id: &str) -> Option<&'a FieldModel> {
        self.find_table_by_id(table_id)
            .and_then(|t| t.fields.iter().find(|f| f.id == field_id))
    }

    /// Relationships where this table is the child (ManyToOne / belongs to parent)
    pub fn incoming_relationships(&self, table_id: &str) -> Vec<&RelationshipModel> {
        self.relationships
            .iter()
            .filter(|r| r.child_table_id == table_id)
            .collect()
    }

    /// Relationships where this table is the parent (OneToMany / has children)
    pub fn outgoing_relationships(&self, table_id: &str) -> Vec<&RelationshipModel> {
        self.relationships
            .iter()
            .filter(|r| r.parent_table_id == table_id)
            .collect()
    }
}

