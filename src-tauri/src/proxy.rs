use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};

/// mihomo 进程管理器
pub struct ProxyManager {
    process: Option<Child>,
    running: AtomicBool,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            process: None,
            running: AtomicBool::new(false),
        }
    }

    /// 启动 mihomo 进程
    pub fn start(&mut self, config_path: &str, _mode: &str) -> Result<(), String> {
        if self.running.load(Ordering::Relaxed) {
            return Err("代理已在运行".to_string());
        }

        // 获取 mihomo 二进制路径（打包在 resources 中）
        let mihomo_path = Self::get_mihomo_path()?;

        // 启动 mihomo
        let child = Command::new(&mihomo_path)
            .args(["-f", config_path])
            .args(["-d", &Self::get_config_dir()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("启动 mihomo 失败: {}", e))?;

        self.process = Some(child);
        self.running.store(true, Ordering::Relaxed);

        // 设置系统代理
        Self::set_system_proxy(true)?;

        Ok(())
    }

    /// 停止 mihomo 进程
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            child.kill().map_err(|e| format!("停止进程失败: {}", e))?;
            child.wait().ok();
        }

        self.running.store(false, Ordering::Relaxed);

        // 清除系统代理
        Self::set_system_proxy(false)?;

        Ok(())
    }

    /// 检查是否运行中
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// 切换代理模式
    pub fn switch_mode(&self, mode: &str) -> Result<(), String> {
        // 通过 mihomo RESTful API 切换模式
        let url = format!("http://127.0.0.1:9090/configs");
        let body = format!(r#"{{"mode":"{}"}}"#, mode);

        // 使用 reqwest 或 curl 切换
        // 这里简化处理，实际应调用 mihomo API
        let _ = (url, body);

        Ok(())
    }

    /// 获取 mihomo 二进制路径
    fn get_mihomo_path() -> Result<String, String> {
        // Tauri 打包后的资源路径
        #[cfg(target_os = "windows")]
        {
            let exe_dir = std::env::current_exe()
                .map_err(|e| format!("获取路径失败: {}", e))?
                .parent()
                .ok_or("无法获取目录")?
                .to_path_buf();
            Ok(exe_dir.join("mihomo.exe").to_string_lossy().to_string())
        }

        #[cfg(target_os = "macos")]
        {
            let exe_dir = std::env::current_exe()
                .map_err(|e| format!("获取路径失败: {}", e))?
                .parent()
                .ok_or("无法获取目录")?
                .to_path_buf();
            Ok(exe_dir.join("mihomo").to_string_lossy().to_string())
        }
    }

    /// 获取配置目录
    fn get_config_dir() -> String {
        dirs::data_local_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("SimpleProxy")
            .to_string_lossy()
            .to_string()
    }

    /// 设置/清除系统代理
    fn set_system_proxy(enable: bool) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            // Windows: 通过注册表设置系统代理
            if enable {
                // 设置代理: 127.0.0.1:7890
                std::process::Command::new("reg")
                    .args([
                        "add",
                        r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings"#,
                        "/v", "ProxyEnable",
                        "/t", "REG_DWORD",
                        "/d", "1",
                        "/f",
                    ])
                    .output()
                    .ok();

                std::process::Command::new("reg")
                    .args([
                        "add",
                        r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings"#,
                        "/v", "ProxyServer",
                        "/t", "REG_SZ",
                        "/d", "127.0.0.1:7890",
                        "/f",
                    ])
                    .output()
                    .ok();
            } else {
                std::process::Command::new("reg")
                    .args([
                        "add",
                        r#"HKCU\Software\Microsoft\Windows\CurrentVersion\Internet Settings"#,
                        "/v", "ProxyEnable",
                        "/t", "REG_DWORD",
                        "/d", "0",
                        "/f",
                    ])
                    .output()
                    .ok();
            }
        }

        #[cfg(target_os = "macos")]
        {
            let services = ["Wi-Fi", "Ethernet"];
            for service in services {
                if enable {
                    std::process::Command::new("networksetup")
                        .args([
                            "-setwebproxy", service, "127.0.0.1", "7890",
                        ])
                        .output()
                        .ok();
                    std::process::Command::new("networksetup")
                        .args([
                            "-setsecurewebproxy", service, "127.0.0.1", "7890",
                        ])
                        .output()
                        .ok();
                } else {
                    std::process::Command::new("networksetup")
                        .args(["-setwebproxystate", service, "off"])
                        .output()
                        .ok();
                    std::process::Command::new("networksetup")
                        .args(["-setsecurewebproxystate", service, "off"])
                        .output()
                        .ok();
                }
            }
        }

        Ok(())
    }
}

impl Drop for ProxyManager {
    fn drop(&mut self) {
        // 确保进程被清理
        if let Some(mut child) = self.process.take() {
            child.kill().ok();
            child.wait().ok();
        }
        Self::set_system_proxy(false).ok();
    }
}
