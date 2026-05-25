use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

pub mod build;
pub mod specs;

pub use build::{build, validate_and_build, BuildKind};
pub use specs::SPECS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldKind {
    String,
    Int,
    Float,
    Bool,
    Select,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Items,
    Movement,
    Broadcast,
    Progression,
    Player,
    Journey,
    Exec,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectOption {
    pub value: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct FieldSpec {
    pub key: &'static str,
    pub label: &'static str,
    pub kind: FieldKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub helper: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<&'static [SelectOption]>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub category: Category,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destructive: Option<bool>,
    #[serde(rename = "needsPlayer")]
    pub needs_player: bool,
    #[serde(rename = "allowAllPlayers")]
    pub allow_all_players: bool,
    pub describe: &'static str,
    pub fields: &'static [FieldSpec],
    #[serde(skip)]
    pub build: BuildKind,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("unknown command: {0}")]
    UnknownCommand(String),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("field {0} must be {1}")]
    WrongType(String, &'static str),
    #[error("Generic broadcast requires Title and Body")]
    BroadcastNeedsTitleAndBody,
}

pub fn find_command(id: &str) -> Option<&'static CommandSpec> {
    SPECS.iter().find(|s| s.id == id)
}
