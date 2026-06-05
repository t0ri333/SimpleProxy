// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod proxy;
mod tray;

use std::sync::Mutex;
use tauri::State;

/// 应用状态
struct AppState {
    proxy_manager: Mutex<proxy::ProxyManager>,
    current_config: Mutex<Option<String>>,
    current_mode: Mutex<String>,
}

/// 导入配置文件
#[tauri::command]
fn import_config(state: State<AppState>, yaml_content: String) -> Result<String, String> {
    // 校验 YAML 格式和必要字段
    config::validate_config(&yaml_content)?;

    // 注入默认设置
    let processed = config::inject_defaults(&yaml_content)?;

    // 保存配置
    let app_dir = dirs::data_local_dir()
        .ok_or("无法获取应用目录")?
        .join("SimpleProxy");
    std::fs::create_dir_all(&app_dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let config_path = app_dir.join("config.yaml");
    std::fs::write(&config_path, &processed).map_err(|e| format!("保存配置失败: {}", e))?;

    // 备份原始文件
    let backup_dir = app_dir.join("configs");
    std::fs::create_dir_all(&backup_dir).ok();
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("config_{}.yaml", timestamp));
    std::fs::write(backup_path, &yaml_content).ok();

    *state.current_config.lock().unwrap() = Some(config_path.to_string_lossy().to_string());

    Ok("配置已就绪".to_string())
}

/// 连接代理
#[tauri::command]
fn connect(state: State<AppState>) -> Result<String, String> {
    let config_path = state
        .current_config
        .lock()
        .unwrap()
        .clone()
        .ok_or("请先拖入配置文件")?;

    let mode = state.current_mode.lock().unwrap().clone();

    let mut manager = state.proxy_manager.lock().unwrap();
    manager.start(&config_path, &mode)?;

    Ok("已连接".to_string())
}

/// 断开代理
#[tauri::command]
fn disconnect(state: State<AppState>) -> Result<String, String> {
    let mut manager = state.proxy_manager.lock().unwrap();
    manager.stop()?;
    Ok("已断开".to_string())
}

/// 切换模式
#[tauri::command]
fn set_mode(state: State<AppState>, mode: String) -> Result<String, String> {
    if mode != "rule" && mode != "global" {
        return Err("无效模式".to_string());
    }

    *state.current_mode.lock().unwrap() = mode.clone();

    // 如果已连接，热重载
    let manager = state.proxy_manager.lock().unwrap();
    if manager.is_running() {
        manager.switch_mode(&mode)?;
    }

    Ok(format!("已切换到{}模式", if mode == "rule" { "智能" } else { "全局" }))
}

/// 获取连接状态
#[tauri::command]
fn get_status(state: State<AppState>) -> String {
    let manager = state.proxy_manager.lock().unwrap();
    let has_config = state.current_config.lock().unwrap().is_some();

    if manager.is_running() {
        "connected".to_string()
    } else if has_config {
        "ready".to_string()
    } else {
        "no_config".to_string()
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 设置系统托盘
            tray::setup_tray(app)?;
            Ok(())
        })
        .manage(AppState {
            proxy_manager: Mutex::new(proxy::ProxyManager::new()),
            current_config: Mutex::new(None),
            current_mode: Mutex::new("rule".to_string()),
        })
        .invoke_handler(tauri::generate_handler![
            import_config,
            connect,
            disconnect,
            set_mode,
            get_status,
        ])
        .run(tauri::generate_context!())
        .expect("启动 SimpleProxy 失败");
}
