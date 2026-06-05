use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Emitter, Manager,
};

/// 设置系统托盘
pub fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    // 创建菜单项
    let open_item = MenuItem::with_id(app, "open", "显示主界面", true, None::<&str>)?;
    let connect_item = MenuItem::with_id(app, "connect", "打开/关闭", true, None::<&str>)?;
    let smart_mode = MenuItem::with_id(app, "smart", "智能模式", true, None::<&str>)?;
    let global_mode = MenuItem::with_id(app, "global", "全局模式", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[&open_item, &connect_item, &smart_mode, &global_mode, &quit_item],
    )?;

    // 创建托盘图标
    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("SimpleProxy")
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
            "connect" => {
                // 触发前端的连接/断开逻辑
                if let Some(window) = app.get_webview_window("main") {
                    window.emit("tray-toggle", ()).ok();
                }
            }
            "smart" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.emit("tray-mode", "rule").ok();
                }
            }
            "global" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.emit("tray-mode", "global").ok();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}
