# Bili2MP4 项目开发计划

基于需求文档 [Bili2MP4需求文档.md](./Bili2MP4需求文档.md)，本文档制定详细的项目开发计划。架构为**轻量化本地单体应用**，采用 **Rust + Tauri**。

---

## 1. 项目概览

| 项目属性 | 说明 |
|----------|------|
| 项目名称 | Bili2MP4 |
| 架构 | 单体应用，GUI + 业务逻辑同一进程 |
| 目标平台 | macOS 11.0+（Intel / Apple Silicon） |
| 开发语言 | Rust |
| GUI | Tauri（Rust + Web 前端） |
| 核心依赖 | GPAC MP4Box |
| 预计总工期 | 6 周 |

---

## 2. 技术架构与目录规划

### 2.1 架构原则

- **单体**：一个进程、一个 `.app`，GUI 与后端逻辑内聚
- **无网络**：仅操作本地文件，无服务端、无 RPC、无 Socket
- **Tauri Commands**：前端通过 `invoke` 调用 Rust 命令，Rust 通过 `emit` 推送进度事件
- **即开即用**：双击启动，关闭即退出

### 2.2 项目结构（Tauri 标准布局）

```
Bili2MP4/
├── src/                       # 前端资源（HTML/JS 或轻量框架）
│   ├── index.html
│   ├── assets/
│   └── ...
├── src-tauri/                  # Rust 后端
│   ├── src/
│   │   ├── main.rs             # 入口
│   │   ├── lib.rs              # Tauri 命令注册
│   │   ├── cache/              # 缓存扫描与 entry.json 解析
│   │   ├── convert/            # MP4Box 转换核心
│   │   ├── filemgr/            # 输出路径、冲突处理
│   │   └── config/             # 偏好与日志
│   ├── Cargo.toml              # 依赖：serde, serde_json, walkdir, tokio 等
│   ├── tauri.conf.json         # 窗口、资源、权限配置
│   └── icons/
├── resources/                  # 捆绑 MP4Box 等
├── .cursor/rules/
└── plan.md
```

### 2.3 模块职责

| 模块 | 对应需求 | 主要文件 |
|------|----------|----------|
| cache | F01, F02, F03, F04 | mod.rs, scanner.rs, parser.rs |
| convert | F06, F07, F08 | mod.rs, mp4box.rs |
| filemgr | F05, F09, F11 | mod.rs, output.rs, conflict.rs |
| config | F12 | mod.rs, preferences.rs |
| GUI | F01-F12, UI | src/ 前端 + lib.rs 命令 |

### 2.4 核心依赖（Cargo.toml）

| 依赖 | 用途 |
|------|------|
| tauri | GUI 框架 |
| serde, serde_json | JSON 解析、配置序列化 |
| walkdir | 遍历缓存目录 |
| tokio | 异步任务、取消支持（可选，或使用 std::process） |
| thiserror, anyhow | 错误处理 |

---

## 3. 阶段规划

### 阶段 1：核心逻辑（2 周）

**目标**：实现缓存扫描、解析、MP4Box 转换，支持进度事件与取消。

#### 1.1 项目初始化（1 天）

- [x] 创建 Tauri 项目结构
- [x] 配置 Cargo.toml 依赖（serde, walkdir, thiserror 等）
- [x] 搭建 src-tauri/src 模块结构（cache, convert, filemgr, config）

#### 1.2 缓存扫描与解析（3 天）

- [x] 默认路径：`~/Movies/Bilibili/`、`~/Library/Containers/com.bilibili.bilibili/Data/Download/`
- [x] 使用 walkdir 遍历子目录，定位 `entry.json`
- [x] serde 反序列化 entry.json，提取标题、清晰度、分 P、音视频 m4s 路径
- [x] 暴露 `scan(dir: &Path) -> Result<Vec<Video>>` 供 Tauri 命令调用

#### 1.3 MP4Box 转换（4 天）

- [x] 调研 m4s 格式，确定 MP4Box 命令（优先 `#video`/`#audio`，备选 `:raw`）
- [x] 实现 `convert(item, out_dir, cancel: &AtomicBool) -> Result<()>`，通过 `tauri::AppHandle::emit` 推送进度
- [x] 使用 `std::process::Command` 启动子进程，捕获 stdout 解析进度
- [x] 文件名安全化（`sanitize_filename` 或手动过滤）
- [x] 错误处理（thiserror 定义错误类型）

#### 1.4 文件与配置（2 天）

- [x] 输出路径、冲突策略（覆盖/跳过/重命名）
- [x] 偏好持久化（serde_json 写入 `~/.config/bili2mp4/config.json`）
- [x] 检测并定位 MP4Box（捆绑或 `which MP4Box`）

**交付物**：Rust 核心库，可被 Tauri 命令调用的 `scan`、`convert`，带进度 emit。

---

### 阶段 2：GUI 实现（2.5 周）✅ 已完成

**目标**：Tauri 单进程 GUI，完成列表、选择、转换、偏好闭环。

#### 2.1 布局与命令绑定（3 天）

- [x] 主窗口布局：工具栏、列表、底部输出路径与进度条
- [x] 在 lib.rs 注册 Tauri 命令：`scan`、`convert`、`cancel_convert`、`get_config`、`set_config` 等
- [x] 前端通过 `@tauri-apps/api` 的 `invoke` 调用命令
- [x] 前端通过 `listen` 订阅 `convert-progress` 事件
- [x] 遵循 macOS HIG，支持深色模式（prefers-color-scheme）

#### 2.2 视频列表（4 天）

- [x] 启动时调用 `scan`（默认或用户选择目录）
- [x] 列表展示：标题、清晰度、大小、分 P 数、缓存日期
- [x] 全选、按清晰度筛选（动态选项，匹配解析出的 1080P+、720P60 等）
- [x] 搜索：按标题关键字过滤，200ms 防抖
- [ ] 拖拽文件夹指定缓存目录（待实现）

#### 2.3 转换与进度（3 天）

- [x] 调用 `convert`，监听 `convert-progress` 事件
- [x] 实时更新进度条与当前文件名
- [x] 转换在 `spawn_blocking` 工作线程执行，避免阻塞主线程
- [x] 转换期间禁用操作，支持取消
- [x] 完成后：打开文件夹（按偏好，shell/open）

#### 2.4 转换兼容性增强

- [x] MP4Box 标准模式失败时自动尝试 `:raw` 模式
- [x] `:raw` 仍失败时使用 ffmpeg 兜底（`ffmpeg -i video.m4s -i audio.m4s -c copy -movflags +faststart`）

**交付物**：可用的 macOS 风格 GUI 小工具，完整转换流程。

---

### 阶段 3：增强与打包（1.5 周）

**目标**：冲突处理、可选弹幕、测试、打包。

#### 3.1 增强功能（3 天）

- [ ] 文件冲突：覆盖、跳过、自动重命名
- [ ] 弹幕（可选）：识别 danmaku.xml，转换 ASS 嵌入
- [ ] MP4Box 捆绑：嵌入静态版到 resources，LGPL 合规

#### 3.2 测试与打包（4 天）

- [ ] 单元测试：`cargo test`，cache/convert/filemgr 模块（覆盖率 ≥ 70%）
- [ ] 端到端：真实 B 站缓存样本，双架构验证
- [ ] `cargo tauri build` 打包 `.app`，体积 ≤ 50MB

#### 3.3 文档（1 天）

- [ ] 用户帮助：安装、使用、常见问题

**交付物**：双架构 `.app` 发布，文档就绪。

---

## 4. 风险与应对

| 风险 | 影响 | 应对措施 |
|------|------|----------|
| m4s 格式与 MP4Box 不兼容 | 无法无损转换 | 提前实测，准备 `:raw` 或流提取备选 |
| MP4Box 体积过大 | 安装包超 50MB | 评估精简版或提示用户自行安装 |
| Tauri 构建 / 权限问题 | 开发受阻 | 查阅 Tauri 文档，按需配置 tauri.conf.json 权限 |
| Rust 异步与取消 | 实现复杂 | 优先使用同步 `std::process::Command` + `AtomicBool` 取消 |

---

## 5. 里程碑检查表

- [x] **M1**：核心逻辑完成，scan、convert 可被 Tauri 命令调用
- [x] **M2**：GUI 完成列表、选择、转换、偏好闭环
- [ ] **M3**：双架构 `.app` 打包，文档完成

---

## 6. 阶段二建设总结

### 6.1 已实现功能

| 功能 | 说明 |
|------|------|
| GUI 主窗口 | 工具栏、视频列表、底部输出区、进度条 |
| 缓存扫描 | 默认路径 + 手动选择，支持 entry.json / videoInfo.json 双格式 |
| 视频列表 | 全选、清晰度筛选（动态选项）、标题搜索 |
| 转换流程 | 勾选→输出目录→开始转换，实时进度、取消 |
| 转换兼容 | MP4Box → :raw → ffmpeg 三级兜底 |
| 异步转换 | `spawn_blocking` 避免主线程阻塞 |
| 权限配置 | Tauri 2 capabilities + 应用命令权限 |
| CLI 测试 | `cargo run --bin cli_test` 命令行验证 |

### 6.2 技术要点

- **Tauri 2**：capabilities 需显式声明 `allow-app-commands` 等
- **转换阻塞**：长时间 MP4Box 需放在 `spawn_blocking` 中执行
- **m4s 兼容**：部分 B 站缓存 MP4Box 报错，ffmpeg 作为兜底

---

*计划版本：1.4*  
*制定日期：2026-02-19*
