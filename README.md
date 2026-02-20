# Bili2MP4

B 站缓存视频无损转换工具（macOS / Windows），基于 GPAC MP4Box 将 m4s 无损封装为标准 MP4。

## 安装

### 方式一：下载 Release

从 [Releases](https://github.com/insanetoto/bilibili2mp4/releases) 下载对应平台安装包：

| 平台 | 文件 | 说明 |
|------|------|------|
| macOS (Apple Silicon) | `Bili2MP4_*_aarch64.dmg` | 拖入「应用程序」文件夹 |
| macOS (Intel) | `Bili2MP4_*_x86_64.dmg` | 同上 |
| Windows x64 | `Bili2MP4_*_x64_*.msi` 或 `*.exe` | 运行安装程序 |

若 Releases 未更新，可前往 [Actions](https://github.com/insanetoto/bilibili2mp4/actions) 中对应版本的工作流，在 Artifacts 处下载构建产物。

### 方式二：从源码构建

**环境要求**
- Rust ≥ 1.70
- Node.js ≥ 18
- macOS 11.0+ 或 Windows 10/11 x64

**依赖安装**

macOS：
```bash
brew install gpac        # 必须：转换核心
brew install ffmpeg      # 可选：兜底转换
```

Windows：需将 MP4Box（GPAC）和 ffmpeg 加入 PATH，或安装至 `C:\Program Files\GPAC`、`C:\ffmpeg` 等常见路径。

**构建**

```bash
git clone https://github.com/insanetoto/bilibili2mp4.git
cd bilibili2mp4
npm install
npm run build
```

- **macOS**：产物在 `src-tauri/target/release/bundle/macos/Bili2MP4.app` 或 `bundle/dmg/*.dmg`
- **Windows**：产物在 `src-tauri/target/release/bundle/` 下的 `.msi` / `.exe`

## 使用说明

1. **选择缓存目录**：点击「选择缓存目录」，选择 B 站客户端的下载目录
   - macOS：`~/Movies/bilibili/` 或 `~/Library/Containers/com.bilibili.bilibili/Data/Download/`
   - Windows：`%LOCALAPPDATA%\bilibili\download\` 或 UWP 版对应 `Packages\Microsoft.48666Bilibili.*\LocalState\download\`
2. **扫描**：点击「扫描」或「刷新」加载视频列表
3. **筛选与搜索**：可按清晰度筛选、按标题搜索
4. **选择输出路径**：点击「浏览」选择 MP4 输出目录
5. **文件冲突**：选择「自动重命名」「覆盖」或「跳过」处理已存在文件
6. **开始转换**：勾选视频，点击「开始转换」

转换完成后，可选择打开输出文件夹。

## 常见问题

**Q: 提示「MP4Box 未找到」**
- macOS：执行 `brew install gpac`
- Windows：安装 [GPAC](https://gpac.wp.imt.fr/downloads/)，或手动指定 MP4Box 路径（若有配置项）

**Q: 转换失败，提示「Cannot find track ID」等**
- 本工具会自动尝试 MP4Box `:raw` 模式和 ffmpeg 兜底。macOS 执行 `brew install ffmpeg`；Windows 需安装 ffmpeg 并加入 PATH

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
