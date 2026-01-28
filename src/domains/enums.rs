use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq)]
#[schemars(inline)]
pub enum IssueType {
    Epic,
    Story,
    Subtask,
    Task,
    Feature,
    Request,
    Bug,
}

impl fmt::Display for IssueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IssueType::Story => write!(f, "Story"),
            IssueType::Bug => write!(f, "Bug"),
            IssueType::Epic => write!(f, "Epic"),
            IssueType::Task => write!(f, "Task"),
            IssueType::Subtask => write!(f, "Subtask"),
            IssueType::Feature => write!(f, "Feature"),
            IssueType::Request => write!(f, "Request"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq)]
#[schemars(inline)]
pub enum Priority {
    Highest,
    High,
    Medium,
    Low,
    Lowest,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Highest => write!(f, "Highest"),
            Priority::High => write!(f, "High"),
            Priority::Medium => write!(f, "Medium"),
            Priority::Low => write!(f, "Low"),
            Priority::Lowest => write!(f, "Lowest"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq)]
#[schemars(inline)]
pub enum Status {
    #[serde(rename = "To Do")]
    ToDo,
    #[serde(rename = "In Progress")]
    InProgress,
    #[serde(rename = "Done")]
    Done,
    #[serde(rename = "In Review")]
    InReview,
    #[serde(rename = "Blocked")]
    Blocked,
    #[serde(rename = "Cancelled")]
    Cancelled,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::ToDo => write!(f, "To Do"),
            Status::InProgress => write!(f, "In Progress"),
            Status::Done => write!(f, "Done"),
            Status::InReview => write!(f, "In Review"),
            Status::Blocked => write!(f, "Blocked"),
            Status::Cancelled => write!(f, "Cancelled"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq)]
#[schemars(inline)]
pub enum LinkType {
    Blocks,
    #[serde(rename = "Is blocked by")]
    IsBlockedBy,
    Clones,
    Relates,
    Duplicates,
}

impl fmt::Display for LinkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkType::Blocks => write!(f, "Blocks"),
            LinkType::IsBlockedBy => write!(f, "Is blocked by"),
            LinkType::Clones => write!(f, "Clones"),
            LinkType::Relates => write!(f, "Relates"),
            LinkType::Duplicates => write!(f, "Duplicates"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[schemars(inline)]
pub enum SprintState {
    Active,
    Future,
    Closed,
}

impl fmt::Display for SprintState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SprintState::Active => write!(f, "active"),
            SprintState::Future => write!(f, "future"),
            SprintState::Closed => write!(f, "closed"),
        }
    }
}

/// Presets de filtros de campos para reducir contexto del LLM.
/// Cubre ~80% de casos comunes sin necesidad de discovery.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum FieldPreset {
    /// Solo identificadores básicos: id, key
    /// Reducción: ~95% del tamaño
    Minimal,

    /// Vista rápida: id, key, summary, status
    /// Reducción: ~90% del tamaño
    Basic,

    /// Campos más comunes: id, key, summary, status, assignee, priority, created, updated
    /// Reducción: ~85% del tamaño
    Standard,

    /// Vista detallada con campos principales: standard + description, labels, components
    /// Reducción: ~70% del tamaño
    Detailed,

    /// Sin filtro (comportamiento actual)
    Full,
}

impl FieldPreset {
    /// Convierte el preset a lista de campos para Jira API
    pub fn to_field_list(&self) -> String {
        match self {
            Self::Minimal => "id,key".to_string(),
            Self::Basic => "id,key,summary,status".to_string(),
            Self::Standard => "id,key,summary,status,assignee,priority,created,updated".to_string(),
            Self::Detailed => "id,key,summary,status,assignee,priority,created,updated,description,labels,components,issuelinks".to_string(),
            Self::Full => "*all".to_string(),
        }
    }
}

impl fmt::Display for FieldPreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minimal => write!(f, "minimal"),
            Self::Basic => write!(f, "basic"),
            Self::Standard => write!(f, "standard"),
            Self::Detailed => write!(f, "detailed"),
            Self::Full => write!(f, "full"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_preset_minimal() {
        assert_eq!(FieldPreset::Minimal.to_field_list(), "id,key");
    }

    #[test]
    fn test_field_preset_basic() {
        let fields = FieldPreset::Basic.to_field_list();
        assert!(fields.contains("summary"));
        assert!(fields.contains("status"));
    }

    #[test]
    fn test_field_preset_standard() {
        let fields = FieldPreset::Standard.to_field_list();
        assert!(fields.contains("assignee"));
        assert!(fields.contains("priority"));
    }
}
