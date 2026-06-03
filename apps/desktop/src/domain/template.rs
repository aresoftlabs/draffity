//! Template schema (v1). Loaded from JSON, applied at project creation.
//!
//! The loader trait can swap in user/cloud templates without changing this
//! domain type. All built-in templates are visible to every user.

use serde::{Deserialize, Serialize};

use crate::domain::DocumentType;
use crate::error::{AppError, AppResult};

pub const TEMPLATE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateKind {
    Novel,
    Paper,
    Manga,
    Screenplay,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    /// Single-line text input.
    String,
    /// Multi-line text input.
    Text,
    Number,
    Date,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateNode {
    pub title: String,
    pub doc_type: DocumentType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub synopsis: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<TemplateNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Template {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub kind: TemplateKind,
    pub locale: String,
    #[serde(default)]
    pub structure: Vec<TemplateNode>,
    #[serde(default, rename = "metadataFields")]
    pub metadata_fields: Vec<MetadataField>,
}

impl Template {
    /// Reject malformed templates with a precise error message. Schema-level
    /// validation only — not concerned with whether the user picked it.
    pub fn validate(&self) -> AppResult<()> {
        if self.schema_version != TEMPLATE_SCHEMA_VERSION {
            return Err(AppError::Invariant(format!(
                "template '{}': unsupported schemaVersion {} (expected {})",
                self.id, self.schema_version, TEMPLATE_SCHEMA_VERSION
            )));
        }
        if self.id.trim().is_empty() {
            return Err(AppError::Invariant("template id is empty".into()));
        }
        if self.name.trim().is_empty() {
            return Err(AppError::Invariant(format!(
                "template '{}': name is empty",
                self.id
            )));
        }
        if self.locale.trim().is_empty() {
            return Err(AppError::Invariant(format!(
                "template '{}': locale is empty",
                self.id
            )));
        }
        for f in &self.metadata_fields {
            if f.key.trim().is_empty() {
                return Err(AppError::Invariant(format!(
                    "template '{}': metadata field with empty key",
                    self.id
                )));
            }
        }
        for node in &self.structure {
            validate_node(&self.id, node)?;
        }
        Ok(())
    }
}

fn validate_node(template_id: &str, node: &TemplateNode) -> AppResult<()> {
    if node.title.trim().is_empty() {
        return Err(AppError::Invariant(format!(
            "template '{template_id}': empty node title"
        )));
    }
    for child in &node.children {
        validate_node(template_id, child)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> Template {
        Template {
            schema_version: 1,
            id: "x".into(),
            name: "X".into(),
            description: None,
            kind: TemplateKind::Generic,
            locale: "es".into(),
            structure: vec![],
            metadata_fields: vec![],
        }
    }

    #[test]
    fn minimal_validates() {
        assert!(minimal().validate().is_ok());
    }

    #[test]
    fn rejects_wrong_schema_version() {
        let mut t = minimal();
        t.schema_version = 2;
        assert!(t.validate().is_err());
    }

    #[test]
    fn rejects_empty_id_or_name() {
        let mut t = minimal();
        t.id = "".into();
        assert!(t.validate().is_err());

        let mut t = minimal();
        t.name = " ".into();
        assert!(t.validate().is_err());
    }

    #[test]
    fn rejects_metadata_field_empty_key() {
        let mut t = minimal();
        t.metadata_fields.push(MetadataField {
            key: "".into(),
            label: "Author".into(),
            field_type: FieldType::String,
            required: true,
            default: None,
        });
        assert!(t.validate().is_err());
    }

    #[test]
    fn rejects_node_with_empty_title() {
        let mut t = minimal();
        t.structure.push(TemplateNode {
            title: "".into(),
            doc_type: DocumentType::Chapter,
            synopsis: None,
            children: vec![],
        });
        assert!(t.validate().is_err());
    }

    #[test]
    fn nested_node_validates_recursively() {
        let mut t = minimal();
        t.structure.push(TemplateNode {
            title: "Acto 1".into(),
            doc_type: DocumentType::Folder,
            synopsis: None,
            children: vec![TemplateNode {
                title: "".into(),
                doc_type: DocumentType::Chapter,
                synopsis: None,
                children: vec![],
            }],
        });
        let err = t.validate().unwrap_err();
        assert!(matches!(err, AppError::Invariant(_)));
    }

    #[test]
    fn deserializes_from_json() {
        let raw = r#"{
            "schemaVersion": 1,
            "id": "demo",
            "name": "Demo",
            "kind": "novel",
            "locale": "en",
            "structure": [
              { "title": "Act 1", "docType": "folder",
                "children": [ { "title": "Ch 1", "docType": "chapter" } ] }
            ],
            "metadataFields": [
              { "key": "author", "label": "Author", "type": "string", "required": true }
            ]
        }"#;
        let t: Template = serde_json::from_str(raw).unwrap();
        assert!(t.validate().is_ok());
        assert_eq!(t.structure.len(), 1);
        assert_eq!(t.structure[0].children.len(), 1);
        assert_eq!(t.metadata_fields[0].key, "author");
    }
}
