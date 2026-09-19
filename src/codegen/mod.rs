pub mod csharp;
pub mod drawdb_parser;
pub mod golang;
pub mod naming;
pub mod node_ts;
pub mod python;
pub mod schema;

use crate::error::{MigrationError, Result};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetLanguage {
    Go,
    Node,
    Python,
    CSharp,
}

impl TargetLanguage {
    pub fn parse(s: &str) -> Result<Self> {
        match s.to_lowercase().trim() {
            "go" | "golang" => Ok(Self::Go),
            "node" | "nodejs" | "ts" | "typescript" | "js" | "javascript" => Ok(Self::Node),
            "python" | "py" => Ok(Self::Python),
            "csharp" | "cs" | "dotnet" | "efcore" => Ok(Self::CSharp),
            other => Err(MigrationError::Codegen(format!(
                "Unsupported target language `{}`. Supported: go, node, python, csharp",
                other
            ))),
        }
    }
}

pub fn generate_models<P: AsRef<Path>, Q: AsRef<Path>>(
    schema_path: P,
    target_lang: TargetLanguage,
    output_dir: Q,
    pkg_or_namespace: Option<&str>,
) -> Result<Vec<PathBuf>> {
    let schema = drawdb_parser::parse_drawdb_file(schema_path)?;
    let out_dir = output_dir.as_ref();

    if !out_dir.exists() {
        fs::create_dir_all(out_dir)?;
    }

    let files: Vec<(String, String)> = match target_lang {
        TargetLanguage::Go => {
            let pkg = pkg_or_namespace.unwrap_or("models");
            golang::generate_go_models(&schema, pkg)
        }
        TargetLanguage::Node => node_ts::generate_node_ts_models(&schema),
        TargetLanguage::Python => python::generate_python_models(&schema),
        TargetLanguage::CSharp => {
            let ns = pkg_or_namespace.unwrap_or("Centra.Models");
            csharp::generate_csharp_models(&schema, ns)
        }
    };

    let mut written = Vec::new();
    for (filename, content) in files {
        let dest = out_dir.join(&filename);
        fs::write(&dest, content)?;
        written.push(dest);
    }

    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const SAMPLE_DRAWDB: &str = r#"{
      "tables": [
        {
          "id": "t_company",
          "name": "m_companies",
          "fields": [
            { "id": "f_c_id", "name": "id", "type": "UUID", "primary": true, "notNull": true },
            { "id": "f_c_name", "name": "name", "type": "VARCHAR", "size": "200", "notNull": true }
          ]
        },
        {
          "id": "t_branch",
          "name": "m_branches",
          "fields": [
            { "id": "f_b_id", "name": "id", "type": "UUID", "primary": true, "notNull": true },
            { "id": "f_b_cid", "name": "company_id", "type": "UUID", "notNull": true },
            { "id": "f_b_name", "name": "name", "type": "VARCHAR", "size": "200", "notNull": true }
          ]
        }
      ],
      "relationships": [
        {
          "id": "rel_1",
          "name": "fk_branch_company",
          "startTableId": "t_company",
          "endTableId": "t_branch",
          "startFieldId": "f_c_id",
          "endFieldId": "f_b_cid",
          "cardinality": "one_to_many"
        }
      ]
    }"#;

    #[test]
    fn test_generate_go() {
        let dir = tempdir().unwrap();
        let schema_file = dir.path().join("schema.json");
        fs::write(&schema_file, SAMPLE_DRAWDB).unwrap();

        let out_dir = dir.path().join("out_go");
        let result = generate_models(&schema_file, TargetLanguage::Go, &out_dir, Some("models"));
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
        let branch_content = fs::read_to_string(out_dir.join("branch.go")).unwrap();
        assert!(branch_content.contains("type Branch struct"));
        assert!(branch_content.contains("*Company"));
        assert!(branch_content.contains("`json:\"company,omitempty\" db:\"-\"`"));
    }

    #[test]
    fn test_generate_ts() {
        let dir = tempdir().unwrap();
        let schema_file = dir.path().join("schema.json");
        fs::write(&schema_file, SAMPLE_DRAWDB).unwrap();

        let out_dir = dir.path().join("out_ts");
        let result = generate_models(&schema_file, TargetLanguage::Node, &out_dir, None);
        assert!(result.is_ok());
        let branch_content = fs::read_to_string(out_dir.join("branch.ts")).unwrap();
        assert!(branch_content.contains("export interface Branch"));
        assert!(branch_content.contains("company?: Company"));
    }

    #[test]
    fn test_generate_python() {
        let dir = tempdir().unwrap();
        let schema_file = dir.path().join("schema.json");
        fs::write(&schema_file, SAMPLE_DRAWDB).unwrap();

        let out_dir = dir.path().join("out_py");
        let result = generate_models(&schema_file, TargetLanguage::Python, &out_dir, None);
        assert!(result.is_ok());
        let py_content = fs::read_to_string(out_dir.join("models.py")).unwrap();
        assert!(py_content.contains("class Branch(Base):"));
        assert!(py_content.contains("class Company(Base):"));
    }

    #[test]
    fn test_generate_csharp() {
        let dir = tempdir().unwrap();
        let schema_file = dir.path().join("schema.json");
        fs::write(&schema_file, SAMPLE_DRAWDB).unwrap();

        let out_dir = dir.path().join("out_cs");
        let result = generate_models(&schema_file, TargetLanguage::CSharp, &out_dir, Some("Centra.Models"));
        assert!(result.is_ok());
        let cs_content = fs::read_to_string(out_dir.join("Branch.cs")).unwrap();
        assert!(cs_content.contains("public partial class Branch"));
        assert!(cs_content.contains("public virtual Company? Company { get; set; }"));
    }
}

