#!/bin/bash
# 方式 A：手动 MP4Box 转换
# 用法: ./scripts/manual-convert.sh
# 需要: brew install gpac

CACHE="/Users/xinz/Movies/bilibili/1318051900"
OUT="/Users/xinz/Downloads"
WORK="/tmp/bili2mp4_work"

mkdir -p "$WORK"

# 新版 B 站 macOS 缓存的 m4s 文件头部有 9 字节 0x30 填充，需去除
echo "去除 m4s 头部填充..."
tail -c +10 "$CACHE/1318051900-1-100050.m4s" > "$WORK/video.m4s"
tail -c +10 "$CACHE/1318051900-1-30280.m4s" > "$WORK/audio.m4s"

if ! command -v MP4Box &>/dev/null; then
  echo "请先安装 MP4Box: brew install gpac"
  exit 1
fi

echo "转换: $OUT/【第4版】第六章-项目管理概论.mp4"
MP4Box -add "$WORK/video.m4s"#video -add "$WORK/audio.m4s"#audio \
  -new "$OUT/【第4版】第六章-项目管理概论.mp4" -itags tool=Bili2MP4

if [ $? -eq 0 ]; then
  echo "✓ 完成"
  rm -rf "$WORK"
else
  echo "✗ 转换失败"
  exit 1
fi
