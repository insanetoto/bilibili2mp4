#!/bin/bash
# 测试转换脚本
# 用法: ./scripts/test-convert.sh
# 需要先安装: brew install gpac

CACHE_DIR="/Users/xinz/Movies/bilibili/1318051900"
OUT_DIR="/Users/xinz/Downloads"

cd "$(dirname "$0")/.."
cargo run --example convert_test 2>&1
