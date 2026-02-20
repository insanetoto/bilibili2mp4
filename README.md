# Bili2MP4

B 站缓存视频无损转换工具（macOS），基于 GPAC MP4Box 将 m4s 无损封装为标准 MP4。

## 安装

### 方式一：从源码构建

**环境要求**
- Rust ≥ 1.70
- Node.js ≥ 18
- macOS 11.0+（Intel / Apple Silicon）

**依赖安装**

```bash
# 安装 MP4Box（转换核心，必须）
brew install gpac

# 可选：部分 m4s 格式需 ffmpeg 兜底
brew install ffmpeg

# 克隆项目并安装
git clone https://github.com/insanetoto/bilibili2mp4.git
cd bilibili2mp4
npm install
```

**构建与运行**

```bash
# 开发模式
npm run dev

# 生产构建（生成 .app）
npm run build
```

构建产物位于 `src-tauri/target/release/bundle/macos/`，双击 `Bili2MP4.app` 即可运行。

### 方式二：下载 Release（若有提供）

从 [Releases](https://github.com/insanetoto/bilibili2mp4/releases) 下载 `.dmg` 或 `.app`，拖入应用程序文件夹。

## 使用说明

1. **选择缓存目录**：点击「选择缓存目录」，选择 B 站 macOS 客户端的下载目录（通常为 `~/Movies/bilibili/` 或 `~/Library/Containers/com.bilibili.bilibili/Data/Download/`）
2. **扫描**：点击「扫描」或「刷新」加载视频列表
3. **筛选与搜索**：可按清晰度筛选、按标题搜索
4. **选择输出路径**：点击「浏览」选择 MP4 输出目录
5. **文件冲突**：选择「自动重命名」「覆盖」或「跳过」处理已存在文件
6. **开始转换**：勾选视频，点击「开始转换」

转换完成后，可选择打开输出文件夹。

## 常见问题

**Q: 提示「MP4Box 未找到」**
- 请执行 `brew install gpac` 安装 MP4Box

**Q: 转换失败，提示「Cannot find track ID」等**
- 本工具会自动尝试 MP4Box `:raw` 模式和 ffmpeg 兜底，请确保已安装 `brew install ffmpeg`

**Q: 默认缓存路径找不到**
- B 站客户端路径可能变更，使用「选择缓存目录」手动指定

**Q: 开发模式下看不到应用图标**
- 自定义图标仅在 `npm run build` 打包后的 `.app` 中生效

## 项目结构

- `src/` — 前端（HTML/JS/CSS）
- `src-tauri/src/` — Rust 后端
  - `cache/` — 缓存扫描、entry.json / videoInfo.json 解析
  - `convert/` — MP4Box 转换（含 ffmpeg 兜底）
  - `filemgr/` — 输出路径、冲突处理
  - `config/` — 偏好配置

## 开发与测试

```bash
# 单元测试
cd src-tauri && cargo test

# CLI 测试（需有真实缓存目录）
cargo run --bin cli_test
```

## 许可证

MIT License — 详见 [LICENSE](./LICENSE)
