use crate::types::{Request, Response};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

pub async fn process_request(request: Request) -> Response {
    info!("Processing request: command={}, filesystem={:?}",
          request.command, request.filesystem);

    match request.command.as_str() {
        "ping" => Response::success(),
        "clone" => handle_clone(request).await,
        "snapshot" => handle_snapshot(request).await,
        "delete" => handle_delete(request).await,
        _ => Response::error(format!("Unknown command: {}", request.command)),
    }
}

async fn handle_clone(request: Request) -> Response {
    let filesystem = request.filesystem.clone().unwrap_or_else(|| "zfs".to_string());

    match filesystem.as_str() {
        "zfs" => handle_zfs_clone(request).await,
        "btrfs" => handle_btrfs_clone(request).await,
        _ => Response::error(format!("Unsupported filesystem: {}", filesystem)),
    }
}

async fn handle_snapshot(request: Request) -> Response {
    let filesystem = request.filesystem.clone().unwrap_or_else(|| "zfs".to_string());

    match filesystem.as_str() {
        "zfs" => handle_zfs_snapshot(request).await,
        "btrfs" => handle_btrfs_snapshot(request).await,
        _ => Response::error(format!("Unsupported filesystem: {}", filesystem)),
    }
}

async fn handle_delete(request: Request) -> Response {
    let filesystem = request.filesystem.clone().unwrap_or_else(|| "zfs".to_string());

    match filesystem.as_str() {
        "zfs" => handle_zfs_delete(request).await,
        "btrfs" => handle_btrfs_delete(request).await,
        _ => Response::error(format!("Unsupported filesystem: {}", filesystem)),
    }
}

async fn handle_zfs_clone(request: Request) -> Response {
    let snapshot = match request.snapshot {
        Some(s) => s,
        None => return Response::error("Missing snapshot parameter".to_string()),
    };

    let clone = match request.clone {
        Some(c) => c,
        None => return Response::error("Missing clone parameter".to_string()),
    };

    debug!("Creating ZFS clone {} from {}", clone, snapshot);

    // Validate that the snapshot exists
    if !zfs_snapshot_exists(&snapshot).await {
        return Response::error(format!("ZFS snapshot {} does not exist", snapshot));
    }

    // Validate that the clone dataset doesn't already exist
    if zfs_dataset_exists(&clone).await {
        return Response::error(format!("ZFS dataset {} already exists", clone));
    }

    // Execute zfs clone with sudo
    match run_command("sudo", &["zfs", "clone", &snapshot, &clone]).await {
        Ok(_) => {
            // Get the mountpoint of the cloned dataset
            match get_zfs_mountpoint(&clone).await {
                Ok(mountpoint) => {
                    if mountpoint != "none" && mountpoint != "legacy" {
                        // Set ownership to the user who started the daemon
                        if let Some(user) = get_sudo_user() {
                            let _ = run_command("sudo", &["chown", "-R", &user, &mountpoint]).await;
                        }
                        Response::success_with_mountpoint(mountpoint)
                    } else {
                        Response::success()
                    }
                }
                Err(e) => {
                    warn!("Failed to get mountpoint for clone {}: {}", clone, e);
                    Response::success() // Clone succeeded but mountpoint unknown
                }
            }
        }
        Err(e) => {
            error!("Failed to create ZFS clone {} from {}: {}", clone, snapshot, e);
            Response::error(format!("Failed to create ZFS clone {} from {}: {}", clone, snapshot, e))
        }
    }
}

async fn handle_zfs_snapshot(request: Request) -> Response {
    let source = match request.source {
        Some(s) => s,
        None => return Response::error("Missing source parameter".to_string()),
    };

    let snapshot = match request.snapshot {
        Some(s) => s,
        None => return Response::error("Missing snapshot parameter".to_string()),
    };

    debug!("Creating ZFS snapshot {} from {}", snapshot, source);

    // Validate that the source dataset exists
    if !zfs_dataset_exists(&source).await {
        return Response::error(format!("ZFS dataset {} does not exist", source));
    }

    // Validate that the snapshot doesn't already exist
    if zfs_snapshot_exists(&snapshot).await {
        return Response::error(format!("ZFS snapshot {} already exists", snapshot));
    }

    // Execute zfs snapshot with sudo
    match run_command("sudo", &["zfs", "snapshot", &snapshot]).await {
        Ok(_) => Response::success(),
        Err(e) => {
            error!("Failed to create ZFS snapshot {}: {}", snapshot, e);
            Response::error(format!("Failed to create ZFS snapshot {}: {}", snapshot, e))
        }
    }
}

async fn handle_zfs_delete(request: Request) -> Response {
    let target = match request.target {
        Some(t) => t,
        None => return Response::error("Missing target parameter".to_string()),
    };

    debug!("Deleting ZFS dataset {}", target);

    // Validate that the target dataset exists
    if !zfs_dataset_exists(&target).await {
        return Response::error(format!("ZFS dataset {} does not exist", target));
    }

    // Execute zfs destroy with sudo
    match run_command("sudo", &["zfs", "destroy", "-r", &target]).await {
        Ok(_) => Response::success(),
        Err(e) => {
            error!("Failed to delete ZFS dataset {}: {}", target, e);
            Response::error(format!("Failed to delete ZFS dataset {}: {}", target, e))
        }
    }
}

async fn handle_btrfs_clone(request: Request) -> Response {
    let source = match request.source {
        Some(s) => s,
        None => return Response::error("Missing source parameter".to_string()),
    };

    let destination = match request.destination {
        Some(d) => d,
        None => return Response::error("Missing destination parameter".to_string()),
    };

    debug!("Creating Btrfs subvolume snapshot {} from {}", destination, source);

    // Validate that the source subvolume exists
    if !btrfs_subvolume_exists(&source).await {
        return Response::error(format!("Btrfs subvolume {} does not exist", source));
    }

    // Validate that the destination doesn't already exist
    if std::path::Path::new(&destination).exists() {
        return Response::error(format!("Destination {} already exists", destination));
    }

    // Execute btrfs subvolume snapshot with sudo
    match run_command("sudo", &["btrfs", "subvolume", "snapshot", &source, &destination]).await {
        Ok(_) => {
            // Set ownership to the user who started the daemon
            if let Some(user) = get_sudo_user() {
                let _ = run_command("sudo", &["chown", "-R", &user, &destination]).await;
            }
            Response::success_with_path(destination)
        }
        Err(e) => {
            error!("Failed to create Btrfs snapshot {} from {}: {}", destination, source, e);
            Response::error(format!("Failed to create Btrfs snapshot {} from {}: {}", destination, source, e))
        }
    }
}

async fn handle_btrfs_snapshot(request: Request) -> Response {
    // For Btrfs, clone and snapshot are the same operation (subvolume snapshot)
    handle_btrfs_clone(request).await
}

async fn handle_btrfs_delete(request: Request) -> Response {
    let target = match request.target {
        Some(t) => t,
        None => return Response::error("Missing target parameter".to_string()),
    };

    debug!("Deleting Btrfs subvolume {}", target);

    // Validate that the target subvolume exists
    if !btrfs_subvolume_exists(&target).await {
        return Response::error(format!("Btrfs subvolume {} does not exist", target));
    }

    // Execute btrfs subvolume delete with sudo
    match run_command("sudo", &["btrfs", "subvolume", "delete", "-R", &target]).await {
        Ok(_) => Response::success(),
        Err(e) => {
            error!("Failed to delete Btrfs subvolume {}: {}", target, e);
            Response::error(format!("Failed to delete Btrfs subvolume {}: {}", target, e))
        }
    }
}

async fn run_command(program: &str, args: &[&str]) -> Result<(), String> {
    debug!("Running command: {} {}", program, args.join(" "));

    let output = Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

async fn get_zfs_mountpoint(dataset: &str) -> Result<String, String> {
    let output = Command::new("sudo")
        .args(&["zfs", "get", "-H", "-o", "value", "mountpoint", dataset])
        .output()
        .await
        .map_err(|e| format!("Failed to get mountpoint: {}", e))?;

    if output.status.success() {
        let mountpoint = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(mountpoint)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.trim().to_string())
    }
}

fn get_sudo_user() -> Option<String> {
    std::env::var("SUDO_USER").ok().or_else(|| std::env::var("USER").ok())
}

async fn zfs_dataset_exists(dataset: &str) -> bool {
    run_command("zfs", &["list", dataset]).await.is_ok()
}

async fn zfs_snapshot_exists(snapshot: &str) -> bool {
    run_command("zfs", &["list", "-t", "snapshot", snapshot]).await.is_ok()
}

async fn btrfs_subvolume_exists(path: &str) -> bool {
    // Check if path exists and is a btrfs subvolume
    match run_command("btrfs", &["subvolume", "show", path]).await {
        Ok(_) => true,
        Err(_) => false,
    }
}
