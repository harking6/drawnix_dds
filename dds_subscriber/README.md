# Drawnix DDS 订阅者

一个独立的命令行程序，用于监听 Drawnix 白板应用通过 DDS 发布的数据变化。

## 🚀 功能特性

- 🎯 **实时监听** - 监听白板变化和元素变化消息
- 📊 **详细统计** - 显示接收消息的统计信息
- 🎨 **彩色输出** - 使用颜色区分不同类型的消息
- ⏱️ **灵活控制** - 支持超时设置和优雅退出
- 🧪 **测试模式** - 内置测试功能验证 DDS 通信

## 📦 编译

```bash
cd dds_subscriber
cargo build --release
```

## 🎯 使用方法

### 基础监听
```bash
./target/release/drawnix-dds-subscriber listen
```

### 详细模式监听
```bash
./target/release/drawnix-dds-subscriber listen --verbose
```

### 指定域 ID 和超时
```bash
./target/release/drawnix-dds-subscriber listen --domain-id 150 --timeout 60
```

### 运行测试
```bash
./target/release/drawnix-dds-subscriber test
```

## 📋 命令行选项

### `listen` 命令

- `-d, --domain-id <DOMAIN_ID>` - DDS 域 ID（默认: 150）
- `-v, --verbose` - 显示详细消息内容
- `-T, --timeout <SECONDS>` - 监听超时时间，0 表示无限期（默认: 0）

### `test` 命令

- `-d, --domain-id <DOMAIN_ID>` - DDS 域 ID（默认: 150）

## 📊 监听的 DDS 主题

### DrawnixBoardChanges
白板整体变化数据，包含：
- 元素数量统计
- 操作列表
- 变化详情
- 时间戳

### DrawnixElementChanges  
元素级别变化数据，包含：
- 新增元素
- 删除元素
- 修改元素
- 时间戳

## 🎨 输出格式

### 基础模式
```
[14:30:15.123] 📨 #1 📋 白板变化
   📊 元素数量: 3
   🔧 操作数量: 1

[14:30:15.456] 📨 #2 🔧 元素变化
   ➕ 新增: 1
   ➖ 删除: 0
   🔄 修改: 2
```

### 详细模式
在基础模式基础上，额外显示：
- 完整的操作数据 JSON
- 详细的元素变化内容
- 原始消息内容（错误时）

## 🧪 测试验证

### 1. 自动化测试
运行完整的发布-订阅测试：
```bash
cd ..
./test_pubsub.sh
```

### 2. 手动测试
1. 启动订阅者：
   ```bash
   ./target/release/drawnix-dds-subscriber listen --verbose
   ```

2. 在另一个终端启动 Tauri 应用：
   ```bash
   cd ../src-tauri
   cargo tauri dev
   ```

3. 在 Tauri 应用中调用 DDS 测试命令

4. 观察订阅者终端的实时输出

## 🔧 故障排除

### 未接收到消息
1. **检查域 ID** - 确保发布者和订阅者使用相同的域 ID
2. **检查 ZRDDS 安装** - 确保 ZRDDS 正确安装在 `/usr/ZRDDS/`
3. **检查权限** - 确保有访问 DDS 域的权限
4. **检查防火墙** - 确保 DDS 通信端口未被阻塞

### 编译错误
1. **检查依赖** - 确保 zrdds-safe 库路径正确
2. **检查 Rust 版本** - 需要 Rust 1.70.0+
3. **检查 ZRDDS_ROOT** - 确保环境变量设置正确

## 📚 示例场景

### 监听 5 分钟并保存日志
```bash
./target/release/drawnix-dds-subscriber listen --verbose --timeout 300 > dds_messages.log 2>&1
```

### 测试连接状态
```bash
./target/release/drawnix-dds-subscriber test
```

### 快速验证是否有消息
```bash
timeout 10s ./target/release/drawnix-dds-subscriber listen
```

## 🎯 与主应用集成

这个订阅者可以用于：
- **开发调试** - 验证 DDS 消息是否正确发布
- **系统监控** - 监控白板活动和性能
- **数据分析** - 记录和分析用户行为
- **故障诊断** - 排查 DDS 通信问题

订阅者与 Drawnix Tauri 应用完全独立，可以在不同的机器上运行，实现分布式监控。
