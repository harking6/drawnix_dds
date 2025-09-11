#!/bin/bash

# 测试白板同步功能的脚本

echo "🧪 开始测试白板DDS同步功能..."

# 检查必要的组件
echo "📋 检查组件状态..."

# 1. 检查DDS订阅者
echo "1️⃣ 启动DDS订阅者（后台）..."
cd dds_subscriber
cargo run -- listen --verbose &
SUBSCRIBER_PID=$!
cd ..

# 等待订阅者启动
sleep 2

# 2. 启动测试发布者
echo "2️⃣ 启动测试发布者（后台）..."
cd test_publisher
cargo run &
PUBLISHER_PID=$!
cd ..

# 等待发布者启动
sleep 2

# 3. 启动主应用
echo "3️⃣ 启动主应用..."
echo "请在浏览器中打开应用，进行以下测试："
echo "   - 创建几何图形（矩形、圆形等）"
echo "   - 拖动和调整图形"
echo "   - 添加文本"
echo "   - 删除元素"
echo "   - 观察右侧日志面板的DDS同步信息"
echo ""
echo "💡 提示：可以同时打开多个浏览器窗口测试多用户协作"
echo ""
echo "按 Ctrl+C 停止所有测试进程"

# 启动主应用
npm run dev

# 清理函数
cleanup() {
    echo ""
    echo "🧹 清理测试进程..."
    kill $SUBSCRIBER_PID 2>/dev/null
    kill $PUBLISHER_PID 2>/dev/null
    echo "✅ 测试完成"
    exit 0
}

# 捕获中断信号
trap cleanup SIGINT SIGTERM

# 等待用户中断
wait