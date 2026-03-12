# Agent Protocol v0.2

JSON-RPC 2.0 协议规范

## 概述

Agent Protocol v0.2 基于 JSON-RPC 2.0 标准，通过 TCP (端口 86) 传输。

## 协议格式

所有消息均为 JSON 格式，每条消息以换行符 (`\n`) 结束。

### 请求格式

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "methodName",
  "params": { ... }
}
```

### 响应格式 (成功)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { ... }
}
```

### 响应格式 (错误)

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32601,
    "message": "Method not found"
  }
}
```

### 通知格式 (无响应)

```json
{
  "jsonrpc": "2.0",
  "method": "bye"
}
```

## JSON-RPC 错误码

| 代码 | 含义 |
|------|------|
| -32700 | Parse error - 无效 JSON |
| -32600 | Invalid Request - 无效请求 |
| -32601 | Method not found - 方法不存在 |
| -32602 | Invalid params - 无效参数 |
| -32603 | Internal error - 内部错误 |

## 方法列表

### hello

客户端连接后的握手。

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "hello",
  "params": {
    "clientName": "apc",
    "clientVersion": "0.2.0"
  }
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "serverName": "agentd",
    "serverVersion": "0.2.0"
  }
}
```

---

### sendMessage

向指定 agent 发送消息。

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "sendMessage",
  "params": {
    "agentId": "hotel",
    "message": "查询房间"
  }
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "response": "查询结果..."
  }
}
```

---

### getAgentCard

获取指定 agent 的能力卡片 (Agent Card)。

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "getAgentCard",
  "params": {
    "agentId": "hotel"
  }
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "name": "北京酒店",
    "capabilities": ["room_booking", "room_query", "room_service"]
  }
}
```

---

### listAgents

列出服务器上的所有 agent。

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "listAgents"
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "agents": ["hotel", "restaurant", "taxi"]
  }
}
```

---

### getServerInfo

获取服务器信息。

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "getServerInfo"
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "version": "0.2.0",
    "agentCount": 3
  }
}
```

---

### findAgents

在 registry 中搜索 agent (仅 registry 支持)。

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "findAgents",
  "params": {
    "query": "hotel Beijing"
  }
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "results": [
      {
        "id": "beijing-hotel",
        "name": "北京大酒店",
        "type": "hotel",
        "address": "agent://hotel.beijing.example.com:86",
        "capabilities": ["booking", "query"],
        "rating": 4.8
      }
    ]
  }
}
```

---

### registerAgent

向 registry 注册新 agent (仅 registry 支持)。

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "registerAgent",
  "params": {
    "id": "my-agent",
    "name": "My Agent",
    "type": "assistant",
    "version": "1.0.0",
    "address": {
      "host": "example.com",
      "port": 86,
      "url": "agent://example.com:86"
    },
    "owner": {
      "id": "owner-1",
      "name": "Owner Name",
      "verified": true
    },
    "capabilities": [
      {"id": "chat", "name": "Chat", "description": "Chat capability"}
    ],
    "metadata": {},
    "publicKey": "..."
  }
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "agentId": "my-agent"
  }
}
```

---

### syncChanges

同步 registry 变更 (仅 registry-mirror 支持)。

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "method": "syncChanges",
  "params": {
    "sinceVersion": 100
  }
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 8,
  "result": {
    "currentVersion": 150,
    "changes": [
      {"action": "Add", "path": "/agents/new-agent", "content": "..."},
      {"action": "Update", "path": "/agents/existing-agent", "content": "..."}
    ]
  }
}
```

---

### bye

关闭连接 (通知，无响应)。

```json
{
  "jsonrpc": "2.0",
  "method": "bye"
}
```

---

## 流式事件 (Streaming Events)

用于任务进度推送。事件以 JSON 行格式发送 (非 JSON-RPC)。

### 事件格式

```json
{"type":"taskStatus","taskId":"task-123","status":"running"}
{"type":"artifact","taskId":"task-123","artifact":{"id":"a1","mimeType":"text/plain","content":{"type":"text","Text":"..."}}}
{"type":"taskCompleted","taskId":"task-123","result":{"status":"ok"}}
{"type":"taskError","taskId":"task-123","error":{"code":500,"message":"Internal error"}}
{"type":"heartbeat","timestamp":1234567890}
```

### 任务状态

| 状态 | 说明 |
|------|------|
| `pending` | 等待执行 |
| `running` | 执行中 |
| `paused` | 暂停 |
| `completed` | 完成 |
| `failed` | 失败 |
| `cancelled` | 已取消 |

### 订阅任务

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "method": "subscribeTask",
  "params": {
    "taskId": "task-123"
  }
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 9,
  "result": {
    "taskId": "task-123",
    "subscribed": true
  }
}
```

### 取消任务

**请求:**
```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "method": "cancelTask",
  "params": {
    "taskId": "task-123"
  }
}
```

**响应:**
```json
{
  "jsonrpc": "2.0",
  "id": 10,
  "result": {
    "taskId": "task-123",
    "cancelled": true
  }
}
```

---

## Agent Card 格式

Agent Card 描述 agent 的能力和安全要求。

```json
{
  "name": "Hotel Agent",
  "description": "北京酒店预订服务",
  "url": "agent://hotel.beijing.example.com:86",
  "version": "1.0.0",
  "type": "hotel",
  "capabilities": [
    {
      "id": "room_booking",
      "name": "房间预订",
      "description": "在线预订房间",
      "inputSchema": {...},
      "outputSchema": {...}
    }
  ],
  "securitySchemes": {
    "mtls": {
      "type": "mutualTLS",
      "description": "双向 TLS 认证"
    }
  },
  "security": [
    {"mtls": []}
  ],
  "provider": {
    "organization": "北京酒店集团",
    "url": "https://beijing-hotel.example.com",
    "email": "contact@beijing-hotel.example.com"
  },
  "metadata": {...}
}
```

---

## 连接示例

### 使用 netcat

```bash
# 连接 agentd
nc localhost 86

# 发送 hello
{"jsonrpc":"2.0","id":1,"method":"hello","params":{"clientName":"test","clientVersion":"1.0"}}

# 发送消息
{"jsonrpc":"2.0","id":2,"method":"sendMessage","params":{"agentId":"echo","message":"Hello"}}
```

### 使用 apc CLI

```bash
# 查看服务器信息
apc --url agent://localhost:86 info

# 列出 agents
apc --url agent://localhost:86 list

# 发送消息
apc --url agent://localhost:86 send echo "Hello World"
```

---

## 从 v0.1 迁移

| v0.1 命令 | v0.2 JSON-RPC 方法 |
|-----------|-------------------|
| `HELLO name version` | `hello` |
| `CAPA @agent` | `getAgentCard` |
| `CAPA @all` | `listAgents` |
| `SEND @agent msg` | `sendMessage` |
| `FIND query` | `findAgents` |
| `REGISTER agent` | `registerAgent` |
| `SYNC since=N` | `syncChanges` |
| `BYE` | `bye` (notification) |

---

## 参考

- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [A2A Protocol](https://github.com/a2aproject/a2a)
