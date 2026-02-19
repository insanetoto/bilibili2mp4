# Bili2MP4

B 站缓存视频无损转换工具（macOS），基于 GPAC MP4Box 将 m4s 无损封装为标准 MP4。

## 环境要求

- Rust (>= 1.70)
- Node.js (>= 18) — 用于 Tauri 构建
- macOS 11.0+

## 安装依赖

```bash
# 安装 Rust（若未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 MP4Box（转换核心）
brew install gpac

# 安装 Node 依赖
npm install
```

## 构建与运行

```bash
# 开发模式
npm run dev

# 生产构建
npm run build
```

## 图标

首次构建前需准备应用图标，可执行：

```bash
# 使用 Tauri 从图片生成图标
npx tauri icon path/to/your-icon.png
```

或手动将图标放入 `src-tauri/icons/` 目录。

## 项目结构

- `src/` — 前端（阶段 2 实现完整 UI）
- `src-tauri/src/` — Rust 后端
  - `cache/` — 缓存扫描与 entry.json 解析
  - `convert/` — MP4Box 转换
  - `filemgr/` — 输出路径、冲突处理
  - `config/` — 偏好配置

## 阶段 1 完成项

- [x] 项目初始化
- [x] 缓存扫描与解析
- [x] MP4Box 转换（标准 + :raw 备选）
- [x] 文件管理与配置
