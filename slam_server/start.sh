#!/bin/bash

echo "正在启动Slam Server服务..."
echo "构建项目中..."

# 构建项目
cargo build

if [ $? -eq 0 ]; then
    echo "构建成功，正在启动服务..."
    # 运行服务
    cargo run
else
    echo "构建失败，请检查错误信息"
    exit 1
fi