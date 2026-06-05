// SimpleProxy 前端逻辑

const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

let isConnected = false;
let isConnecting = false;
let currentMode = 'rule';

// DOM 元素
const statusBar = document.getElementById('statusBar');
const statusText = document.getElementById('statusText');
const dropZone = document.getElementById('dropZone');
const configLoaded = document.getElementById('configLoaded');
const smartModeBtn = document.getElementById('smartMode');
const globalModeBtn = document.getElementById('globalMode');
const connectBtn = document.getElementById('connectBtn');
const btnText = document.getElementById('btnText');
const configName = document.getElementById('configName');
const configNameText = document.getElementById('configNameText');

// 初始化
async function init() {
    // 检查初始状态
    try {
        const status = await invoke('get_status');
        updateUI(status);
    } catch (e) {
        console.error('获取状态失败:', e);
    }

    // 监听托盘事件
    await listen('tray-toggle', () => toggleConnection());
    await listen('tray-mode', (event) => setMode(event.payload));

    // 设置拖拽
    setupDragDrop();
}

// 设置拖拽功能
function setupDragDrop() {
    const dropZoneEl = document.getElementById('dropZone');

    // 阻止默认行为
    ['dragenter', 'dragover', 'dragleave', 'drop'].forEach(eventName => {
        dropZoneEl.addEventListener(eventName, preventDefaults, false);
        document.body.addEventListener(eventName, preventDefaults, false);
    });

    // 拖拽高亮
    ['dragenter', 'dragover'].forEach(eventName => {
        dropZoneEl.addEventListener(eventName, () => {
            dropZoneEl.classList.add('drag-over');
        });
    });

    ['dragleave', 'drop'].forEach(eventName => {
        dropZoneEl.addEventListener(eventName, () => {
            dropZoneEl.classList.remove('drag-over');
        });
    });

    // 处理拖放
    dropZoneEl.addEventListener('drop', handleDrop);
}

function preventDefaults(e) {
    e.preventDefault();
    e.stopPropagation();
}

// 处理文件拖放
async function handleDrop(e) {
    const files = e.dataTransfer.files;
    if (files.length === 0) return;

    const file = files[0];
    const name = file.name.toLowerCase();

    // 检查文件类型
    if (!name.endsWith('.yaml') && !name.endsWith('.yml')) {
        showError('请拖入 .yaml 文件');
        return;
    }

    try {
        // 读取文件内容
        const content = await readFile(file);

        // 调用后端导入
        const result = await invoke('import_config', { yamlContent: content });

        // 更新 UI
        configNameText.textContent = file.name;
        dropZone.style.display = 'none';
        configLoaded.style.display = 'flex';
        configName.style.display = 'block';

        updateStatusBar('ready');
    } catch (error) {
        showError(error.toString());
    }
}

// 读取文件
function readFile(file) {
    return new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(reader.result);
        reader.onerror = () => reject('读取文件失败');
        reader.readAsText(file);
    });
}

// 切换连接状态
async function toggleConnection() {
    if (isConnecting) return;

    isConnecting = true;
    connectBtn.classList.add('connecting');
    connectBtn.disabled = true;
    btnText.innerHTML = '<span class="spinner"></span>连接中';

    try {
        if (isConnected) {
            await invoke('disconnect');
            isConnected = false;
            updateStatusBar('ready');
        } else {
            await invoke('connect');
            isConnected = true;
            updateStatusBar('connected');
        }
    } catch (error) {
        showError(error.toString());
        updateStatusBar('error');
    } finally {
        isConnecting = false;
        connectBtn.classList.remove('connecting');
        connectBtn.disabled = false;
    }
}

// 设置模式
async function setMode(mode) {
    currentMode = mode;

    // 更新按钮样式
    smartModeBtn.classList.toggle('active', mode === 'rule');
    globalModeBtn.classList.toggle('active', mode === 'global');

    try {
        await invoke('set_mode', { mode });
    } catch (error) {
        console.error('切换模式失败:', error);
    }
}

// 更新状态栏
function updateStatusBar(status) {
    statusBar.className = 'status-bar';
    connectBtn.className = 'connect-btn';

    switch (status) {
        case 'connected':
            statusBar.classList.add('status-connected');
            statusText.textContent = '已连接';
            connectBtn.classList.add('connected');
            btnText.textContent = '关 闭';
            isConnected = true;
            break;
        case 'ready':
            statusBar.classList.add('status-off');
            statusText.textContent = '未连接';
            btnText.textContent = '打 开';
            isConnected = false;
            break;
        case 'error':
            statusBar.classList.add('status-error');
            statusText.textContent = '连接失败';
            btnText.textContent = '打 开';
            isConnected = false;
            break;
        case 'no_config':
        default:
            statusBar.classList.add('status-off');
            statusText.textContent = '未连接';
            btnText.textContent = '打 开';
            isConnected = false;
            break;
    }
}

// 更新 UI 状态
function updateUI(status) {
    updateStatusBar(status);

    if (status === 'connected' || status === 'ready') {
        // 有配置时隐藏拖拽区
        dropZone.style.display = 'none';
        configLoaded.style.display = 'flex';
    }
}

// 显示错误
function showError(message) {
    // 简单的错误提示
    const errorDiv = document.createElement('div');
    errorDiv.style.cssText = `
        position: fixed;
        top: 20px;
        left: 50%;
        transform: translateX(-50%);
        background: #ffebee;
        color: #c62828;
        padding: 12px 24px;
        border-radius: 8px;
        font-size: 16px;
        z-index: 1000;
        box-shadow: 0 2px 8px rgba(0,0,0,0.15);
    `;
    errorDiv.textContent = message;
    document.body.appendChild(errorDiv);

    setTimeout(() => {
        errorDiv.remove();
    }, 3000);
}

// 页面加载后初始化
document.addEventListener('DOMContentLoaded', init);
