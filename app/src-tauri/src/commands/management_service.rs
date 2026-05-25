use std::path::PathBuf;

use base64::Engine as _;
use dune_manager_core::orchestration::{RemoteCommandRunner, RusshRunner, RusshTarget};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::commands::shared::{command_error_message, sh_single_quoted};

const REMOTE_BINARY_PATH: &str = "/opt/dune-server-service/dune-server-service";
const REMOTE_SYSTEMD_UNIT_PATH: &str = "/etc/systemd/system/dune-server-service.service";
const REMOTE_OPENRC_PATH: &str = "/etc/init.d/dune-server-service";
const REMOTE_TOKEN_PATH: &str = "/home/dune/.dune/state/command-auth-token";

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementInstallRequest {
    pub host: String,
    pub user: String,
    pub key_path: Option<String>,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    /// Optional command-auth token. If None, install only refreshes the binary.
    pub command_auth_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementConnRequest {
    pub host: String,
    pub user: String,
    pub key_path: Option<String>,
    #[serde(default = "default_ssh_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementInstallResult {
    pub installed: bool,
    pub started: bool,
    pub init_system: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagementServiceStatus {
    pub installed: bool,
    pub active: bool,
    pub init_system: String,
    pub journal_tail: String,
}

fn default_ssh_port() -> u16 {
    22
}

fn target_from_conn(req: &ManagementConnRequest) -> Result<RusshTarget, String> {
    let mut target = RusshTarget::new(
        PathBuf::from(req.key_path.as_deref().unwrap_or_default().trim().to_string()),
        req.user.trim().to_string(),
        req.host.trim().to_string(),
    );
    if req.port != 0 {
        target.port = req.port;
    }
    target.validate().map_err(|err| err.message)?;
    Ok(target)
}

fn target_from_install(req: &ManagementInstallRequest) -> Result<RusshTarget, String> {
    let conn = ManagementConnRequest {
        host: req.host.clone(),
        user: req.user.clone(),
        key_path: req.key_path.clone(),
        port: req.port,
    };
    target_from_conn(&conn)
}

fn resolve_resource(app: &tauri::AppHandle, path: &str) -> Result<PathBuf, String> {
    let resource = app
        .path()
        .resolve(path, tauri::path::BaseDirectory::Resource)
        .map_err(|err| format!("resolving bundled {path}: {err}"))?;
    if !resource.exists() {
        return Err(format!("bundled {path} missing at {}", resource.display()));
    }
    Ok(resource)
}

#[tauri::command]
pub async fn install_management_service(
    app: tauri::AppHandle,
    request: ManagementInstallRequest,
) -> Result<ManagementInstallResult, String> {
    let binary_path = resolve_resource(&app, "binaries/dune-server-service")?;
    let unit_path = resolve_resource(&app, "binaries/dune-server-service.service")?;
    let openrc_path = resolve_resource(&app, "binaries/dune-server-service.openrc")?;
    let target = target_from_install(&request)?;
    let token = request.command_auth_token.clone();

    tauri::async_runtime::spawn_blocking(move || {
        install_inner(&target, &binary_path, &unit_path, &openrc_path, token.as_deref())
    })
    .await
    .map_err(|err| format!("install worker failed: {err}"))?
}

#[tauri::command]
pub async fn uninstall_management_service(
    request: ManagementConnRequest,
) -> Result<(), String> {
    let target = target_from_conn(&request)?;
    tauri::async_runtime::spawn_blocking(move || uninstall_inner(&target))
        .await
        .map_err(|err| format!("uninstall worker failed: {err}"))?
}

#[tauri::command]
pub async fn management_service_status(
    request: ManagementConnRequest,
) -> Result<ManagementServiceStatus, String> {
    let target = target_from_conn(&request)?;
    tauri::async_runtime::spawn_blocking(move || status_inner(&target))
        .await
        .map_err(|err| format!("status worker failed: {err}"))?
}

fn install_inner(
    target: &RusshTarget,
    binary_path: &std::path::Path,
    unit_path: &std::path::Path,
    openrc_path: &std::path::Path,
    token: Option<&str>,
) -> Result<ManagementInstallResult, String> {
    let binary_b64 = read_b64(binary_path)?;
    let unit_b64 = read_b64(unit_path)?;
    let openrc_b64 = read_b64(openrc_path)?;

    let token_segment = match token {
        Some(t) => {
            let token_b64 = base64::engine::general_purpose::STANDARD.encode(t.as_bytes());
            format!(
                "sudo install -d -m 0700 -o dune -g dune /home/dune/.dune/state && \
                 echo {b64} | base64 -d | sudo install -m 0600 -o dune -g dune /dev/stdin {dest} && ",
                b64 = sh_single_quoted(&token_b64),
                dest = sh_single_quoted(REMOTE_TOKEN_PATH),
            )
        }
        None => String::new(),
    };

    let stop_old = format!(
        "{{ sudo systemctl disable --now server-management-service.service >/dev/null 2>&1 || true; \
          sudo systemctl stop dune-server-service.service >/dev/null 2>&1 || true; \
          sudo rc-service dune-server-service stop >/dev/null 2>&1 || true; }} && "
    );

    let install_binary = format!(
        "sudo install -d -m 0755 /opt/dune-server-service && \
         echo {b64} | base64 -d | sudo install -m 0755 -o root -g root /dev/stdin {dest} && ",
        b64 = sh_single_quoted(&binary_b64),
        dest = sh_single_quoted(REMOTE_BINARY_PATH),
    );

    let init_install_and_start = format!(
        "if command -v systemctl >/dev/null 2>&1; then \
            echo SYSTEMD; \
            echo {unit_b64} | base64 -d | sudo install -m 0644 -o root -g root /dev/stdin {unit_dest} && \
            sudo systemctl daemon-reload && \
            sudo systemctl enable --now dune-server-service.service && \
            sudo systemctl is-active dune-server-service.service; \
         elif command -v rc-service >/dev/null 2>&1; then \
            echo OPENRC; \
            echo {openrc_b64} | base64 -d | sudo install -m 0755 -o root -g root /dev/stdin {openrc_dest} && \
            sudo rc-update add dune-server-service default >/dev/null 2>&1 || true; \
            sudo rc-service dune-server-service restart >/dev/null 2>&1 || sudo rc-service dune-server-service start; \
            sleep 1; \
            sudo rc-service dune-server-service status >/dev/null 2>&1 && echo active || echo inactive; \
         else \
            echo NONE; \
            echo \"no supported init system found (need systemd or openrc)\" >&2; \
            exit 1; \
         fi",
        unit_b64 = sh_single_quoted(&unit_b64),
        unit_dest = sh_single_quoted(REMOTE_SYSTEMD_UNIT_PATH),
        openrc_b64 = sh_single_quoted(&openrc_b64),
        openrc_dest = sh_single_quoted(REMOTE_OPENRC_PATH),
    );

    let script = format!(
        "set -eu\n\
         export PATH=/sbin:/usr/sbin:/usr/local/sbin:$PATH\n\
         {stop_old}\n{install_binary}\n{token_segment}\n{init_install_and_start}\n\
         exit 0\n",
    );

    let runner = RusshRunner::new(target.clone());
    let stdout = runner.run_script(&script).map_err(command_error_message)?;

    let mut init_system = String::from("unknown");
    let mut active_state = String::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        match trimmed {
            "SYSTEMD" => init_system = "systemd".to_string(),
            "OPENRC" => init_system = "openrc".to_string(),
            "active" | "inactive" => active_state = trimmed.to_string(),
            _ => {}
        }
    }
    let started = active_state == "active";

    Ok(ManagementInstallResult {
        installed: true,
        started,
        init_system: init_system.clone(),
        message: format!("installed via {init_system}; active={active_state}"),
    })
}

fn uninstall_inner(target: &RusshTarget) -> Result<(), String> {
    let script = "set -eu\n\
         export PATH=/sbin:/usr/sbin:/usr/local/sbin:$PATH\n\
         if command -v systemctl >/dev/null 2>&1; then\n  \
             sudo systemctl disable --now dune-server-service.service >/dev/null 2>&1 || true\n  \
             sudo rm -f /etc/systemd/system/dune-server-service.service\n  \
             sudo systemctl daemon-reload\n\
         fi\n\
         if command -v rc-service >/dev/null 2>&1; then\n  \
             sudo rc-service dune-server-service stop >/dev/null 2>&1 || true\n  \
             sudo rc-update del dune-server-service default >/dev/null 2>&1 || true\n  \
             sudo rm -f /etc/init.d/dune-server-service\n\
         fi\n\
         sudo rm -rf /opt/dune-server-service\n\
         exit 0\n";
    let runner = RusshRunner::new(target.clone());
    runner
        .run_script(script)
        .map_err(command_error_message)
        .map(|_| ())
}

fn status_inner(target: &RusshTarget) -> Result<ManagementServiceStatus, String> {
    let script = "set +e\n\
         export PATH=/sbin:/usr/sbin:/usr/local/sbin:$PATH\n\
         [ -x /opt/dune-server-service/dune-server-service ] && echo INSTALLED=yes || echo INSTALLED=no\n\
         if command -v systemctl >/dev/null 2>&1; then\n  \
             echo INIT=systemd\n  \
             sudo systemctl is-active dune-server-service.service\n  \
             sudo journalctl -u dune-server-service.service -n 25 --no-pager 2>/dev/null | tail -n 25\n\
         elif command -v rc-service >/dev/null 2>&1; then\n  \
             echo INIT=openrc\n  \
             sudo rc-service dune-server-service status >/dev/null 2>&1 && echo active || echo inactive\n  \
             sudo tail -n 25 /var/log/dune-server-service.log 2>/dev/null || true\n\
         else\n  \
             echo INIT=none\n\
         fi\n\
         exit 0\n";
    let runner = RusshRunner::new(target.clone());
    let stdout = runner.run_script(script).map_err(command_error_message)?;
    let mut installed = false;
    let mut active = false;
    let mut init_system = String::from("unknown");
    let mut journal_lines: Vec<&str> = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        match trimmed {
            "INSTALLED=yes" => installed = true,
            "INSTALLED=no" => installed = false,
            "INIT=systemd" => init_system = "systemd".to_string(),
            "INIT=openrc" => init_system = "openrc".to_string(),
            "INIT=none" => init_system = "none".to_string(),
            "active" => active = true,
            "inactive" => active = false,
            other if !other.is_empty() => journal_lines.push(other),
            _ => {}
        }
    }
    Ok(ManagementServiceStatus {
        installed,
        active,
        init_system,
        journal_tail: journal_lines.join("\n"),
    })
}

fn read_b64(path: &std::path::Path) -> Result<String, String> {
    let bytes = std::fs::read(path)
        .map_err(|err| format!("reading resource {}: {err}", path.display()))?;
    Ok(base64::engine::general_purpose::STANDARD.encode(&bytes))
}
