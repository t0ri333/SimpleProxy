use serde_yaml::Value;

/// 校验配置文件
pub fn validate_config(yaml_content: &str) -> Result<(), String> {
    let value: Value =
        serde_yaml::from_str(yaml_content).map_err(|_| "文件不是有效的 YAML 格式")?;

    // 检查必须是映射
    let map = value.as_mapping().ok_or("配置文件格式错误")?;

    // 检查 proxies 字段
    let has_proxies = map
        .keys()
        .any(|k| k.as_str() == Some("proxies"));

    if !has_proxies {
        return Err("配置文件缺少 proxies 字段".to_string());
    }

    // 检查 proxies 不为空
    let proxies = &map[&Value::String("proxies".to_string())];
    if let Some(list) = proxies.as_sequence() {
        if list.is_empty() {
            return Err("配置文件中没有服务器节点".to_string());
        }
    } else {
        return Err("proxies 格式错误，应为列表".to_string());
    }

    Ok(())
}

/// 注入应用默认设置
pub fn inject_defaults(yaml_content: &str) -> Result<String, String> {
    let mut value: Value =
        serde_yaml::from_str(yaml_content).map_err(|e| format!("解析 YAML 失败: {}", e))?;

    let map = value.as_mapping_mut().ok_or("配置格式错误")?;

    // 注入/覆盖必要字段
    let key = |s: &str| Value::String(s.to_string());

    // mixed-port: 7890
    map.insert(key("mixed-port"), Value::Number(7890.into()));

    // allow-lan: false（安全起见）
    map.insert(key("allow-lan"), Value::Bool(false));

    // bind-address: "*"
    map.insert(key("bind-address"), Value::String("*".to_string()));

    // mode: rule
    map.insert(key("mode"), Value::String("rule".to_string()));

    // log-level: warning
    map.insert(key("log-level"), Value::String("warning".to_string()));

    // ipv6: false
    map.insert(key("ipv6"), Value::Bool(false));

    // geodata-mode: true
    map.insert(key("geodata-mode"), Value::Bool(true));

    // DNS 设置
    let dns = serde_yaml::from_str::<Value>(
        r#"
enable: true
enhanced-mode: fake-ip
fake-ip-range: 198.18.0.1/16
default-nameserver:
  - 223.5.5.5
  - 119.29.29.29
nameserver:
  - https://doh.pub/dns-query
  - https://dns.alidns.com/dns-query
fallback:
  - https://1.1.1.1/dns-query
  - https://dns.google/dns-query
fallback-filter:
  geoip: true
  geoip-code: CN
"#,
    )
    .unwrap();

    map.insert(key("dns"), dns);

    // TUN 设置（Windows 需要管理员权限）
    let tun = serde_yaml::from_str::<Value>(
        r#"
enable: true
stack: gvisor
dns-hijack:
  - any:53
auto-route: true
auto-detect-interface: true
"#,
    )
    .unwrap();

    map.insert(key("tun"), tun);

    // geox-url
    let geox = serde_yaml::from_str::<Value>(
        r#"
geoip: "https://cdn.jsdelivr.net/gh/MetaCubeX/meta-rules-dat@release/geoip.dat"
geosite: "https://cdn.jsdelivr.net/gh/MetaCubeX/meta-rules-dat@release/geosite.dat"
mmdb: "https://cdn.jsdelivr.net/gh/MetaCubeX/meta-rules-dat@release/country.mmdb"
"#,
    )
    .unwrap();

    map.insert(key("geox-url"), geox);

    // 序列化回 YAML
    serde_yaml::to_string(&value).map_err(|e| format!("序列化 YAML 失败: {}", e))
}
