use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsCatalog {
    pub files: Vec<UserSettingsFileSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsFileSummary {
    pub id: &'static str,
    pub file_name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsFile {
    pub id: &'static str,
    pub file_name: &'static str,
    pub path: String,
    pub content: String,
    pub size_bytes: usize,
    pub sections: Vec<IniSection>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IniSection {
    pub name: String,
    pub entries: Vec<IniEntry>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IniEntry {
    pub key: String,
    pub value: String,
    pub line: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsUpdateRequest {
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsUpdateResponse {
    pub file: UserSettingsFile,
    pub restart_recommended: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsBackupSummary {
    pub id: String,
    pub file_name: String,
    pub size_bytes: usize,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsBackupsResponse {
    pub file: String,
    pub backups: Vec<UserSettingsBackupSummary>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsBackupCreateResponse {
    pub backup: UserSettingsBackupSummary,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettingsRestoreResponse {
    pub file: UserSettingsFile,
    pub restored_from: String,
    pub restart_recommended: bool,
}
