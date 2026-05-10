use serde_json::Value;
use std::{fs, time::{SystemTime, UNIX_EPOCH}};
use tauri::AppHandle;

use crate::{
    config_store::app_data_dir,
    errors::{failure, parse_json},
    models::{
        BattleGroupDetail, BattleGroupSummary, CommandResult, ConfigSnapshot, ServerSetSummary,
        WorkloadList,
    },
    resolve_connection,
    security::redact_json,
    ssh::run_ssh,
    validation::validate_kube_arg,
};

fn get_bg_json(
    app: &AppHandle,
    install_path: &str,
    ip: &str,
    ssh_user: &str,
) -> CommandResult<Value> {
    let command = "sudo kubectl get battlegroup -A -o json";
    let raw = run_ssh(app, install_path, ip, ssh_user, command)?;
    parse_json(&raw, "battlegroup list")
}

fn summarize_server_sets(item: &Value) -> Vec<ServerSetSummary> {
    item["spec"]["serverGroup"]["template"]["spec"]["sets"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|set| ServerSetSummary {
            map: set["map"].as_str().unwrap_or_default().to_string(),
            replicas: set["replicas"].as_u64().unwrap_or_default(),
            memory_limit: set["resources"]["limits"]["memory"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            dedicated_scaling: set["dedicatedScaling"].as_bool().unwrap_or(false),
            image: set["image"].as_str().unwrap_or_default().to_string(),
        })
        .collect()
}

fn unique_strings(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut output = Vec::new();
    for value in values {
        if !value.is_empty() && !output.contains(&value) {
            output.push(value);
        }
    }
    output
}

fn detail_from_battlegroup(item: &Value) -> BattleGroupDetail {
    let namespace = item["metadata"]["namespace"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let name = item["metadata"]["name"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let server_sets = summarize_server_sets(item);
    let server_image = server_sets
        .first()
        .map(|set| set.image.clone())
        .unwrap_or_default();

    let mut utility_images = Vec::new();
    for path in [
        &item["spec"]["utilities"]["director"]["spec"]["image"],
        &item["spec"]["utilities"]["serverGateway"]["spec"]["image"],
        &item["spec"]["utilities"]["textRouter"]["spec"]["image"],
        &item["spec"]["utilities"]["fileBrowser"]["spec"]["image"],
    ] {
        if let Some(image) = path.as_str() {
            utility_images.push(image.to_string());
        }
    }
    for template in item["spec"]["utilities"]["messageQueues"]["templates"]
        .as_array()
        .cloned()
        .unwrap_or_default()
    {
        if let Some(image) = template["spec"]["image"].as_str() {
            utility_images.push(image.to_string());
        }
    }

    BattleGroupDetail {
        namespace,
        name,
        title: item["spec"]["title"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        phase: item["status"]["phase"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        stop: item["spec"]["stop"].as_bool().unwrap_or(false),
        database_phase: item["status"]["database"]["phase"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        server_group_phase: item["status"]["serverGroup"]["phase"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        gateway_phase: item["status"]["serverGateway"]["phase"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        director_phase: item["status"]["director"]["phase"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        server_image,
        utility_images: unique_strings(utility_images.into_iter()),
        server_sets,
    }
}

#[tauri::command]
pub fn get_battlegroups(
    app: AppHandle,
    install_path: Option<String>,
    ip: Option<String>,
    ssh_user: Option<String>,
) -> CommandResult<Vec<BattleGroupSummary>> {
    let (install_path, ip, ssh_user) = resolve_connection(&app, install_path, ip, ssh_user)?;

    let value = get_bg_json(&app, &install_path, &ip, &ssh_user)?;
    let mut groups = Vec::new();
    for item in value["items"].as_array().cloned().unwrap_or_default() {
        let namespace = item["metadata"]["namespace"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let name = item["metadata"]["name"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let title = item["spec"]["title"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let phase = item["status"]["phase"]
            .as_str()
            .unwrap_or_default()
            .to_string();
        let stop = item["spec"]["stop"].as_bool().unwrap_or(false);
        let server_sets = item["spec"]["serverGroup"]["template"]["spec"]["sets"]
            .as_array()
            .map(|sets| sets.len())
            .unwrap_or_default();
        let server_image = item["spec"]["serverGroup"]["template"]["spec"]["sets"]
            .as_array()
            .and_then(|sets| sets.first())
            .and_then(|set| set["image"].as_str())
            .unwrap_or_default()
            .to_string();

        validate_kube_arg(&namespace, "namespace")?;
        let services_raw = run_ssh(
            &app,
            &install_path,
            &ip,
            &ssh_user,
            &format!("sudo kubectl get svc -n {namespace} -o json"),
        )?;
        let services: Value = parse_json(&services_raw, "services")?;
        let mut file_browser_url = None;
        let mut director_url = None;
        for svc in services["items"].as_array().cloned().unwrap_or_default() {
            let svc_name = svc["metadata"]["name"].as_str().unwrap_or_default();
            for port in svc["spec"]["ports"].as_array().cloned().unwrap_or_default() {
                let port_number = port["port"].as_u64().unwrap_or_default();
                let node_port = port["nodePort"].as_u64();
                if svc_name.ends_with("-fb-svc") || port_number == 18888 {
                    file_browser_url = Some(format!("http://{ip}:18888/"));
                }
                if port_number == 11717 {
                    if let Some(node_port) = node_port {
                        director_url = Some(format!("http://{ip}:{node_port}/"));
                    }
                }
            }
        }

        groups.push(BattleGroupSummary {
            namespace,
            name,
            title,
            phase,
            stop,
            server_image,
            file_browser_url,
            director_url,
            server_sets,
        });
    }
    Ok(groups)
}

#[tauri::command]
pub fn get_battlegroup_detail(
    app: AppHandle,
    namespace: String,
    name: String,
    install_path: Option<String>,
    ip: Option<String>,
    ssh_user: Option<String>,
) -> CommandResult<BattleGroupDetail> {
    validate_kube_arg(&namespace, "namespace")?;
    validate_kube_arg(&name, "name")?;
    let (install_path, ip, ssh_user) = resolve_connection(&app, install_path, ip, ssh_user)?;
    let raw = run_ssh(
        &app,
        &install_path,
        &ip,
        &ssh_user,
        &format!("sudo kubectl get battlegroup {name} -n {namespace} -o json"),
    )?;
    let value: Value = parse_json(&raw, "live BattleGroup")?;
    Ok(detail_from_battlegroup(&value))
}

#[tauri::command]
pub fn get_workloads(
    app: AppHandle,
    namespace: String,
    install_path: Option<String>,
    ip: Option<String>,
    ssh_user: Option<String>,
) -> CommandResult<WorkloadList> {
    validate_kube_arg(&namespace, "namespace")?;
    let (install_path, ip, ssh_user) = resolve_connection(&app, install_path, ip, ssh_user)?;

    let pods = run_ssh(
        &app,
        &install_path,
        &ip,
        &ssh_user,
        &format!("sudo kubectl get pods -n {namespace} -o json"),
    )?;
    let services = run_ssh(
        &app,
        &install_path,
        &ip,
        &ssh_user,
        &format!("sudo kubectl get svc -n {namespace} -o json"),
    )?;

    Ok(WorkloadList {
        pods: parse_json(&pods, "pods")?,
        services: parse_json(&services, "services")?,
    })
}

fn patch_battlegroup_stop(
    app: &AppHandle,
    namespace: &str,
    name: &str,
    stop: bool,
    install_path: &str,
    ip: &str,
    ssh_user: &str,
) -> CommandResult<()> {
    validate_kube_arg(namespace, "namespace")?;
    validate_kube_arg(name, "name")?;
    let patch = if stop { "true" } else { "false" };
    let remote = format!(
        "sudo kubectl patch battlegroup {name} -n {namespace} --type=merge -p '{{\"spec\":{{\"stop\":{patch}}}}}'"
    );
    run_ssh(app, install_path, ip, ssh_user, &remote)?;
    Ok(())
}

#[tauri::command]
pub fn start_battlegroup(
    app: AppHandle,
    namespace: String,
    name: String,
    install_path: Option<String>,
    ip: Option<String>,
    ssh_user: Option<String>,
) -> CommandResult<()> {
    let (install_path, ip, ssh_user) = resolve_connection(&app, install_path, ip, ssh_user)?;
    patch_battlegroup_stop(
        &app,
        &namespace,
        &name,
        false,
        &install_path,
        &ip,
        &ssh_user,
    )
}

#[tauri::command]
pub fn stop_battlegroup(
    app: AppHandle,
    namespace: String,
    name: String,
    install_path: Option<String>,
    ip: Option<String>,
    ssh_user: Option<String>,
) -> CommandResult<()> {
    let (install_path, ip, ssh_user) = resolve_connection(&app, install_path, ip, ssh_user)?;
    patch_battlegroup_stop(&app, &namespace, &name, true, &install_path, &ip, &ssh_user)
}

#[tauri::command]
pub fn restart_battlegroup(
    app: AppHandle,
    namespace: String,
    name: String,
    install_path: Option<String>,
    ip: Option<String>,
    ssh_user: Option<String>,
) -> CommandResult<()> {
    let (install_path, ip, ssh_user) = resolve_connection(&app, install_path, ip, ssh_user)?;
    patch_battlegroup_stop(&app, &namespace, &name, true, &install_path, &ip, &ssh_user)?;
    std::thread::sleep(std::time::Duration::from_secs(5));
    patch_battlegroup_stop(
        &app,
        &namespace,
        &name,
        false,
        &install_path,
        &ip,
        &ssh_user,
    )
}

#[tauri::command]
pub fn export_live_config(
    app: AppHandle,
    namespace: String,
    name: String,
    install_path: Option<String>,
    ip: Option<String>,
    ssh_user: Option<String>,
) -> CommandResult<ConfigSnapshot> {
    validate_kube_arg(&namespace, "namespace")?;
    validate_kube_arg(&name, "name")?;
    let (install_path, ip, ssh_user) = resolve_connection(&app, install_path, ip, ssh_user)?;
    let raw = run_ssh(
        &app,
        &install_path,
        &ip,
        &ssh_user,
        &format!("sudo kubectl get battlegroup {name} -n {namespace} -o json"),
    )?;

    let snapshots = app_data_dir(&app)?.join("snapshots");
    fs::create_dir_all(&snapshots)
        .map_err(|err| failure(format!("Failed to create snapshots directory: {err}")))?;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let file_name = format!("{name}-live-{timestamp}.json");
    let path = snapshots.join(file_name);
    let mut value: Value = parse_json(&raw, "live BattleGroup")?;
    redact_json(&mut value);
    let snapshot = serde_json::to_string_pretty(&value)
        .map_err(|err| failure(format!("Failed to serialize snapshot: {err}")))?;
    fs::write(&path, snapshot)
        .map_err(|err| failure(format!("Failed to write snapshot: {err}")))?;

    Ok(ConfigSnapshot {
        file_path: path.to_string_lossy().to_string(),
    })
}
