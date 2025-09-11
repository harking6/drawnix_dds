#!/bin/bash

echo "🚀 DDS 发布-订阅测试"
echo "==================="

# 设置颜色
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}📦 编译程序...${NC}"

# 编译订阅者
echo "编译 DDS 订阅者..."
cd dds_subscriber
if cargo build --release; then
    echo -e "${GREEN}✅ 订阅者编译成功${NC}"
else
    echo -e "${RED}❌ 订阅者编译失败${NC}"
    exit 1
fi
cd ..

# 编译发布者
echo "编译 DDS 发布者..."
cd test_publisher
if cargo build --release; then
    echo -e "${GREEN}✅ 发布者编译成功${NC}"
else
    echo -e "${RED}❌ 发布者编译失败${NC}"
    exit 1
fi
cd ..

echo ""
echo -e "${GREEN}✅ 编译完成！${NC}"
echo ""

echo -e "${BLUE}🧪 使用说明:${NC}"
echo "1. 在一个终端启动订阅者:"
echo -e "   ${YELLOW}cd dds_subscriber && ./target/release/drawnix-dds-subscriber listen --verbose${NC}"
echo ""
echo "2. 在另一个终端启动发布者:"
echo -e "   ${YELLOW}cd test_publisher && ./target/release/test_publisher${NC}"
echo ""
echo "3. 观察订阅者终端的实时消息接收"

echo ""
echo -e "${BLUE}📋 项目结构:${NC}"
echo "drawnix-dds/"
echo "├── dds_subscriber/          # DDS 订阅者程序"
echo "│   ├── src/main.rs         # 订阅者源码"
echo "│   ├── Cargo.toml          # 订阅者依赖"
echo "│   └── ZRDDS_QOS_PROFILES.xml"
echo "├── test_publisher/          # DDS 发布者程序"
echo "│   ├── src/main.rs         # 发布者源码"
echo "│   ├── Cargo.toml          # 发布者依赖"
echo "│   └── ZRDDS_QOS_PROFILES.xml"
echo "└── test_dds.sh             # 本测试脚本"

echo ""
echo -e "${BLUE}🎯 快速测试:${NC}"
echo "如果想要快速验证，可以运行："
echo -e "${YELLOW}timeout 30s ./dds_subscriber/target/release/drawnix-dds-subscriber listen --verbose --timeout 20 &${NC}"
echo -e "${YELLOW}sleep 3 && ./test_publisher/target/release/test_publisher${NC}"
