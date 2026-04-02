// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use winreg::enums::*;
use winreg::RegKey;
use std::process::Command;
use std::path::PathBuf;
use sysinfo::System;
use std::thread;
use std::time::Duration;
use std::env;
use std::fs;

mod process_killer;
mod auto_typer;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
fn enable_bnet_multi_instance() -> Result<(), String> {
    if let Ok(app_data) = env::var("APPDATA") {
        let config_path = PathBuf::from(app_data).join("Battle.net").join("Battle.net.config");
        
        if config_path.exists() {
            if let Ok(config_str) = fs::read_to_string(&config_path) {
                if let Ok(mut config) = serde_json::from_str::<serde_json::Value>(&config_str) {
                    let mut changed = false;
                    
                    if let Some(client) = config.get_mut("Client") {
                        if let Some(client_obj) = client.as_object_mut() {
                            if client_obj.get("AllowMultipleInstances").and_then(|v| v.as_str()) != Some("true") {
                                client_obj.insert("AllowMultipleInstances".to_string(), serde_json::json!("true"));
                                changed = true;
                            }
                        }
                    } else {
                        let mut client_obj = serde_json::Map::new();
                        client_obj.insert("AllowMultipleInstances".to_string(), serde_json::json!("true"));
                        if let Some(root) = config.as_object_mut() {
                            root.insert("Client".to_string(), serde_json::Value::Object(client_obj));
                            changed = true;
                        }
                    }
                    
                    if changed {
                        if let Ok(updated_config_str) = serde_json::to_string_pretty(&config) {
                            let _ = fs::write(&config_path, updated_config_str);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

#[tauri::command]
fn get_d2r_path() -> Result<String, String> {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_path = r#"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Diablo II Resurrected"#;
    
    if let Ok(key) = hklm.open_subkey(key_path) {
        if let Ok(install_location) = key.get_value::<String, _>("InstallLocation") {
            let path = PathBuf::from(&install_location).join("D2R.exe");
            if path.exists() {
                return Ok(install_location);
            }
        }
    }
    
    Err("Could not find Diablo II Resurrected installation path in registry.".into())
}

#[tauri::command]
fn kill_d2r_mutex() -> Result<String, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut found = false;
    let mut total_killed = 0;

    for (pid, process) in sys.processes() {
            if process.name().to_string().to_lowercase() == "d2r.exe" {
                found = true;
                println!("Found D2R process with PID: {}", pid);
                
                // 调用 process_killer 模块去遍历和查杀这个进程下的 Mutex
                match process_killer::kill_d2r_mutexes_for_pid(pid.as_u32()) {
                    Ok(killed) => {
                        println!("Successfully killed {} mutexes for PID {}", killed, pid);
                        total_killed += killed;
                    },
                    Err(e) => {
                        println!("Error killing mutexes for PID {}: {}", pid, e);
                    }
                }
            }
        }
    
    if found {
        Ok(format!("Successfully processed D2R mutexes. Killed: {}", total_killed))
    } else {
        Ok("No running D2R process found, ready to launch.".into())
    }
}

// 核心功能：启动 D2R
#[tauri::command]
fn launch_d2r(path: String, args: Vec<String>, username: Option<String>, password: Option<String>) -> Result<String, String> {
    let executable = PathBuf::from(&path).join("D2R.exe");
    
    if !executable.exists() {
        return Err(format!("D2R.exe not found in {}", path));
    }

    match Command::new(&executable)
        .current_dir(path)
        .args(args)
        .spawn() {
        Ok(child) => {
            let pid = child.id();
            
            // 如果提供了账号密码，启动一个新线程在几秒后尝试自动输入
            if let (Some(u), Some(p)) = (username, password) {
                thread::spawn(move || {
                    // 等待游戏启动到登录界面（根据电脑性能调整，暂定 10 秒）
                    println!("Waiting 10s for D2R to reach login screen...");
                    thread::sleep(Duration::from_secs(10));
                    
                    if let Err(e) = auto_typer::type_credentials(&u, &p) {
                        println!("Auto-type failed: {}", e);
                    } else {
                        println!("Auto-type finished for PID {}", pid);
                    }
                });
            }

            Ok(format!("Successfully launched D2R with PID: {}", pid))
        },
        Err(e) => Err(format!("Failed to launch D2R: {}", e)),
    }
}

#[tauri::command]
fn launch_bnet() -> Result<String, String> {
    // 首先自动开启战网允许多开选项
    let _ = enable_bnet_multi_instance();

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let key_path = r#"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Battle.net"#;
    
    let mut bnet_path = PathBuf::from(r#"C:\Program Files (x86)\Battle.net\Battle.net.exe"#);
    
    if let Ok(key) = hklm.open_subkey(key_path) {
        if let Ok(install_location) = key.get_value::<String, _>("InstallLocation") {
            let path = PathBuf::from(&install_location).join("Battle.net.exe");
            if path.exists() {
                bnet_path = path;
            }
        }
    }

    if !bnet_path.exists() {
            return Err("Could not find Battle.net.exe. Please ensure Battle.net is installed.".into());
        }

        // 不传递 --exec="launch OSI"，而是直接启动战网。
        // 配合之前修改的 AllowMultipleInstances="true"，这会弹出一个全新的战网登录窗口！
        match Command::new(&bnet_path)
            .spawn() {
            Ok(_) => Ok("Successfully launched Battle.net to start D2R.".into()),
            Err(e) => Err(format!("Failed to launch Battle.net: {}", e)),
        }
    }

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            get_d2r_path,
            kill_d2r_mutex,
            launch_d2r,
            launch_bnet
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
