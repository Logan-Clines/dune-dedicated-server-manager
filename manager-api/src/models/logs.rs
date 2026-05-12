use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogExportResponse {
    pub namespace: String,
    pub generated_at_unix_ms: u128,
    pub tail_lines: i64,
    pub pods: Vec<PodLogExport>,
    pub errors: Vec<LogExportError>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PodLogExport {
    pub name: String,
    pub phase: String,
    pub containers: Vec<ContainerLogExport>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerLogExport {
    pub name: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogExportError {
    pub pod: String,
    pub container: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagerLogResponse {
    pub path: &'static str,
    pub available: bool,
    pub truncated: bool,
    pub tail_lines: usize,
    pub lines: Vec<String>,
}
