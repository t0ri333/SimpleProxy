# SimpleProxy

**一键翻墙，给爸妈用的 VPN 客户端**

拖进去、按一下、就能上网的 VPN 工具。

## 给子女的使用说明

### 安装（给爸妈）

1. 从 GitHub Releases 下载最新的 `SimpleProxy` 安装包
2. 双击安装（会自动创建桌面快捷方式）
3. 把配置文件（`.yaml`）发给爸妈
4. 爸妈把配置文件拖进 SimpleProxy → 点"打开" → 完事

### 本地开发

```bash
# 安装依赖
npm install

# 开发模式运行
npm run dev

# 构建安装包
npm run build
```

### 配置文件

配置文件由子女准备，格式为标准的 Clash/mihomo YAML。爸妈只需要拖入使用。

配置文件示例：
```yaml
proxies:
  - name: "线路1"
    type: ss
    server: xxx.xxx.com
    port: 7879
    cipher: aes-256-gcm
    password: your-password

proxy-groups:
  - name: Proxy
    type: select
    proxies:
      - "线路1"
      - DIRECT

rules:
  - DOMAIN-SUFFIX,google.com,Proxy
  - DOMAIN-SUFFIX,youtube.com,Proxy
  - GEOSITE,CN,DIRECT
  - GEOIP,CN,DIRECT
  - MATCH,Proxy
```

### 构建 Windows 安装包

1. 推送代码到 GitHub
2. 创建 tag：`git tag v1.0.0 && git push --tags`
3. GitHub Actions 会自动构建 Windows `.exe` 安装包
4. 从 Releases 下载安装包

## 功能

- 拖拽导入配置文件
- 一键连接/断开
- 智能模式/全局模式切换
- 系统托盘最小化
- 开机自启动

## 技术栈

- Tauri 2 (Rust + WebView)
- mihomo 代理核心
- 原生 HTML/CSS/JS
