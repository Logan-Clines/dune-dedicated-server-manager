use std::{
    env,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    errors::failure,
    models::CommandResult,
    shell::{ps_single_quoted, run_powershell},
};

const STEAMCMD_URL: &str = "https://steamcdn-a.akamaihd.net/client/installer/steamcmd.zip";
const OPENSSH_URL: &str =
    "https://github.com/PowerShell/Win32-OpenSSH/releases/latest/download/OpenSSH-Win64.zip";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ManagedTool {
    #[serde(rename = "steamcmd")]
    SteamCmd,
    #[serde(rename = "openssh")]
    OpenSsh,
}

impl ManagedTool {
    pub fn parse(value: &str) -> CommandResult<Self> {
        match value.to_ascii_lowercase().as_str() {
            "steamcmd" | "steam-cmd" => Ok(Self::SteamCmd),
            "openssh" | "open-ssh" | "ssh" => Ok(Self::OpenSsh),
            _ => Err(failure(format!(
                "Unknown managed tool {value}; expected steamcmd or openssh"
            ))),
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Self::SteamCmd => "steamcmd",
            Self::OpenSsh => "openssh",
        }
    }

    pub fn executable_name(self) -> &'static str {
        match self {
            Self::SteamCmd => "steamcmd.exe",
            Self::OpenSsh => "ssh.exe",
        }
    }

    pub fn default_url(self) -> &'static str {
        match self {
            Self::SteamCmd => STEAMCMD_URL,
            Self::OpenSsh => OPENSSH_URL,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub tool: ManagedTool,
    pub installed: bool,
    pub tools_root: PathBuf,
    pub install_dir: PathBuf,
    pub executable: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInstallResult {
    pub status: ToolStatus,
    pub source_url: String,
    pub installed_now: bool,
}

#[derive(Debug, Clone)]
pub struct Toolchain {
    root: PathBuf,
}

impl Toolchain {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn from_default_root() -> CommandResult<Self> {
        Ok(Self::new(default_tools_root()?))
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn status(&self, tool: ManagedTool) -> ToolStatus {
        let install_dir = self.install_dir(tool);
        let executable = install_dir.join(tool.executable_name());
        ToolStatus {
            tool,
            installed: executable.is_file(),
            tools_root: self.root.clone(),
            install_dir,
            executable,
        }
    }

    pub fn status_all(&self) -> Vec<ToolStatus> {
        [ManagedTool::SteamCmd, ManagedTool::OpenSsh]
            .into_iter()
            .map(|tool| self.status(tool))
            .collect()
    }

    pub fn install(
        &self,
        tool: ManagedTool,
        force: bool,
        source_url: Option<String>,
    ) -> CommandResult<ToolInstallResult> {
        let before = self.status(tool);
        let source_url = source_url.unwrap_or_else(|| tool.default_url().to_string());
        if before.installed && !force {
            return Ok(ToolInstallResult {
                status: before,
                source_url,
                installed_now: false,
            });
        }

        let script = install_zip_tool_script(&self.root, tool, &source_url, force);
        run_powershell(&script)?;
        let status = self.status(tool);
        if !status.installed {
            return Err(failure(format!(
                "{} installation did not produce {}",
                tool.id(),
                status.executable.display()
            )));
        }
        Ok(ToolInstallResult {
            status,
            source_url,
            installed_now: true,
        })
    }

    fn install_dir(&self, tool: ManagedTool) -> PathBuf {
        self.root.join("tools").join(tool.id())
    }
}

pub fn default_tools_root() -> CommandResult<PathBuf> {
    if let Ok(value) = env::var("DUNE_MANAGER_HOME") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    if let Ok(value) = env::var("LOCALAPPDATA") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed).join("DuneDedicatedServerManager"));
        }
    }
    Ok(env::current_dir()
        .map_err(|err| failure(format!("Failed to determine current directory: {err}")))?
        .join(".dune-manager"))
}

fn install_zip_tool_script(
    root: &Path,
    tool: ManagedTool,
    source_url: &str,
    force: bool,
) -> String {
    let install_dir = root.join("tools").join(tool.id());
    let downloads_dir = root.join("downloads");
    let staging_dir = root.join("staging").join(tool.id());
    let archive_path = downloads_dir.join(format!("{}.zip", tool.id()));
    let executable = install_dir.join(tool.executable_name());
    format!(
        r#"
$ErrorActionPreference = 'Stop'
$url = {url}
$installDir = {install_dir}
$downloadsDir = {downloads_dir}
$stagingDir = {staging_dir}
$archivePath = {archive_path}
$executable = {executable}
$executableName = {executable_name}
$force = {force}

if ((Test-Path -LiteralPath $executable) -and (-not $force)) {{
  [Console]::Out.WriteLine("already-installed")
  exit 0
}}

New-Item -ItemType Directory -Force -Path $downloadsDir | Out-Null
New-Item -ItemType Directory -Force -Path (Split-Path -Parent $stagingDir) | Out-Null
if (Test-Path -LiteralPath $stagingDir) {{ Remove-Item -LiteralPath $stagingDir -Recurse -Force }}
if ((Test-Path -LiteralPath $installDir) -and $force) {{
  Remove-Item -LiteralPath $installDir -Recurse -Force
}}

Invoke-WebRequest -Uri $url -OutFile $archivePath
Expand-Archive -LiteralPath $archivePath -DestinationPath $stagingDir -Force

$found = Get-ChildItem -LiteralPath $stagingDir -Recurse -File -Filter $executableName | Select-Object -First 1
if (-not $found) {{ throw "Archive did not contain $executableName" }}

New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item -Path (Join-Path $found.DirectoryName '*') -Destination $installDir -Recurse -Force
if (-not (Test-Path -LiteralPath $executable)) {{
  throw "Expected executable was not installed: $executable"
}}
[Console]::Out.WriteLine($executable)
"#,
        url = ps_single_quoted(source_url),
        install_dir = ps_single_quoted(&install_dir.to_string_lossy()),
        downloads_dir = ps_single_quoted(&downloads_dir.to_string_lossy()),
        staging_dir = ps_single_quoted(&staging_dir.to_string_lossy()),
        archive_path = ps_single_quoted(&archive_path.to_string_lossy()),
        executable = ps_single_quoted(&executable.to_string_lossy()),
        executable_name = ps_single_quoted(tool.executable_name()),
        force = if force { "$true" } else { "$false" },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tool_paths_are_app_owned() {
        let root = PathBuf::from(r"C:\Users\Example\AppData\Local\DuneDedicatedServerManager");
        let toolchain = Toolchain::new(root.clone());
        let status = toolchain.status(ManagedTool::SteamCmd);
        assert_eq!(status.tools_root, root);
        assert!(status.executable.ends_with(r"tools\steamcmd\steamcmd.exe"));
    }

    #[test]
    fn install_script_downloads_and_expands_without_global_path_changes() {
        let script = install_zip_tool_script(
            Path::new(r"C:\DuneTools"),
            ManagedTool::OpenSsh,
            ManagedTool::OpenSsh.default_url(),
            false,
        );
        assert!(script.contains("Invoke-WebRequest"));
        assert!(script.contains("Expand-Archive"));
        assert!(script.contains("OpenSSH-Win64.zip"));
        assert!(!script.contains("setx"));
        assert!(!script.contains("$env:Path"));
    }
}
