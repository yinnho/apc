# Agent Protocol CLI

Agent Protocol 客户端和服务端实现，支持 JSON-RPC 2.0 协议。

## 组件

| 组件 | 说明 |
|------|------|
| `apc` | Agent Protocol Client - 命令行客户端 |
| `agentd` | Agent Daemon - Agent 服务端 |
| `registry` | Registry - Agent 注册中心 |
| `registry-mirror` | Registry Mirror - 注册中心镜像 |
| `shared` | 共享库 |

## 协议版本

- **v0.2**: JSON-RPC 2.0 over TCP (端口 86)
- **v0.1**: 自定义文本协议 (已废弃)

## 快速开始

### 构建

```bash
cargo build --workspace --release
```

### 运行 agentd

```bash
./target/release/agentd
```

### 使用 apc 客户端

```bash
# 查看服务器信息
./target/release/apc --url agent://localhost:86 info

# 列出 agents
./target/release/apc --url agent://localhost:86 list

# 发送消息
./target/release/apc --url agent://localhost:86 send echo "Hello World"
```

## 协议示例

### JSON-RPC 请求

```json
{"jsonrpc":"2.0","id":1,"method":"hello","params":{"clientName":"apc","clientVersion":"0.2.0"}}
```

### JSON-RPC 响应

```json
{"jsonrpc":"2.0","id":1,"result":{"serverName":"agentd","serverVersion":"0.2.0"}}
```

### 流式事件

```json
{"type":"taskStatus","taskId":"task-123","status":"running"}
{"type":"taskCompleted","taskId":"task-123","result":{"status":"ok"}}
```

## 文档

- [协议规范](docs/PROTOCOL.md) - JSON-RPC 2.0 方法定义
- [开发计划](.claude/plans/ancient-splashing-anchor.md) - A2A 融合改造计划

## 测试

```bash
# 单元测试
cargo test --workspace

# 集成测试
./tests/integration_test.sh
```

## License

MIT
