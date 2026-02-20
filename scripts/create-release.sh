#!/bin/bash
# 在 GitHub 创建 v1.0.0 Release 并上传 dmg
# 需先安装 GitHub CLI: brew install gh && gh auth login

set -e
cd "$(dirname "$0")/.."
DMG="src-tauri/target/release/bundle/dmg/Bili2MP4_1.0.0_aarch64.dmg"
if [ ! -f "$DMG" ]; then
  echo "请先执行: npm run build"
  exit 1
fi
gh release create v1.0.0 \
  "$DMG" \
  --title "Bili2MP4 v1.0.0" \
  --notes "## 首个正式版本

### 功能
- 缓存扫描、视频列表、清晰度筛选、标题搜索
- MP4Box 无损转换（含 :raw 备选、ffmpeg 兜底）
- 文件冲突：覆盖 / 跳过 / 自动重命名
- 单元测试、用户文档

### 系统要求
- macOS 11.0+（Apple Silicon）
- 需安装 \`brew install gpac\`（ffmpeg 可选）

### 安装
下载 \`Bili2MP4_1.0.0_aarch64.dmg\`，打开后拖入应用程序文件夹。"
echo "Release 已创建: https://github.com/insanetoto/bilibili2mp4/releases/tag/v1.0.0"
