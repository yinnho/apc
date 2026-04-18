# agc：像 curl 一样访问 AI Agent

## 一个命令，访问任何 Agent

1998 年，curl 诞生。从那以后，访问任何网站只需要一行命令：

```bash
curl https://example.com
```

二十七年后，AI Agent 已经无处不在——Claude、Copilot、Gemini、Cursor……但它们各自为政，没有统一的访问方式。你想调用一个 Agent，要先装它的客户端、学它的 SDK、调它的 API。

今天，这件事变得和 curl 一样简单：

```bash
agc agent://abc123.relay.example.com/claude "hello"
```

**agc**——Agent Protocol Client，一行命令访问任何 Agent。

## 怎么用

安装：

```bash
cargo install aginx-cli
```

访问一个 Agent：

```bash
agc agent://abc123.relay.example.com/copilot "帮我写一个快速排序"
```

指定工作目录：

```bash
agc --cwd /my/project agent://abc123.relay.example.com/claude "修复这个 bug"
```

管道输入：

```bash
cat error.log | agc agent://abc123.relay.example.com/claude
```

就这么简单。不需要 SDK，不需要 API Key，不需要装任何客户端。只要有一个 agent:// 地址，一行命令就能对话。

## agent:// 是什么

你可能注意到了，agc 使用的不是 http://，而是 **agent://**。

这不是噱头。就像 http:// 定义了访问网站的方式，agent:// 定义了访问 Agent 的方式：

```
agent://<id>.relay.example.com/<agent>    → 通过中继访问（TLS 加密）
agent://192.168.1.100:86/<agent>          → 直连访问
```

URL 即协议。看到地址就知道怎么连、连到哪、用哪个 Agent。不需要文档，不需要配置文件。

## 为什么这件事重要

### 1. Agent 需要一个互联网

今天的 AI Agent 像上世纪 90 年代初的网站——每个都独立存在，彼此不互通。你想用 Claude，要打开 Claude；想用 Copilot，要打开 VS Code。

但 Agent 不应该只是给人用的工具。Agent 之间需要互相调用、互相协作。一个 Agent 可以调用另一个 Agent 完成子任务，就像一个网站调用另一个网站的 API。

这需要一个基础设施——**Agent 互联网**。

### 2. 统一协议的意义

回顾互联网的历史，真正让网络爆发的是统一协议：

- **TCP/IP** 统一了网络层——任何设备都可以互联
- **HTTP** 统一了应用层——任何网站都可以被访问
- **HTML** 统一了表现层——任何内容都可以被渲染

Agent 时代需要同样的统一。Aginx 定义的 ACP（Agent Communication Protocol）就是这个意图——任何 Agent 都说同一种语言，就像任何网站都说 HTTP。

### 3. 从工具到生态

curl 之所以伟大，不是因为它能发 HTTP 请求，而是因为它让任何人都能在一行命令里触达整个 Web。它降低了访问的门槛，让 Web API 真正成为基础设施。

agc 想做同样的事。当访问任何 Agent 只需要一行命令时：

- **开发者**可以用脚本编排 Agent 工作流
- **Agent** 可以调用其他 Agent，形成协作网络
- **企业**可以部署私有 Agent，像内网网站一样被访问
- **任何人**都可以发布自己的 Agent，让全世界都能用到

### 4. 去中心化的 Agent 网络

Aginx 的架构 deliberately 去中心化：

- 每个 aginx 实例是一个"服务器"，上面的 Agent 是"网站"
- aginx-relay 做中继穿透，但只转发消息，不存储数据
- Agent 可以在本地运行，也可以在云上运行
- 任何人都可以部署自己的 aginx，发布自己的 Agent

这不是一个平台，是一个协议。不是一个 App Store，是一个开放网络。

## agc 是第一步

agc 是 Aginx 生态的第一个用户端工具。它很简洁——几百行 Rust 代码，只做一件事：连接 agent://，发消息，收回复。

但这一步的意义在于：**它证明了 Agent 可以像网站一样被访问。**

```bash
curl https://www.google.com       # 访问网站
agc  agent://abc.example.com/claude  # 访问 Agent
```

当访问 Agent 和访问网站一样简单时，Agent 互联网就真的开始了。

---

*agc 已在 crates.io 发布：`cargo install aginx-cli`*
*开源地址：https://github.com/yinnho/agc*
