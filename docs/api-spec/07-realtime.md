# 07 - 实时通信（WebSocket）

## 概述

管理 WebSocket 连接生命周期：认证握手、心跳保活、消息路由（双向）、事件广播。作为所有模块的实时通信基础设施，所有 bridge 事件通过此通道推送给客户端。

**源码位置**：`process/webserver/websocket/WebSocketManager.ts`、`process/webserver/adapter.ts`、`common/adapter/registry.ts`、`common/adapter/standalone.ts`

> **定位**：WebSocket 层本身不包含业务逻辑——它是一个通用的**双向消息总线**。上行消息（客户端 → 服务端）路由到 bridge handler 执行业务逻辑；下行消息（服务端 → 客户端）是各模块 bridge 的事件广播。本文档描述 WebSocket 层自身的连接管理、认证、心跳、消息格式和路由机制，以及完整的下行事件目录。

## 架构设计

### 消息流转

```
客户端 (WebUI / Electron Renderer)
  │                          ▲
  │  ws.send({name, data})   │  ws.send({name, data})
  ▼                          │
┌──────────────────────────────────────────┐
│           WebSocketManager               │
│  - 认证（token 验证）                      │
│  - 心跳（ping/pong）                       │
│  - JSON 解析 / 序列化                      │
│  - 内置消息处理（pong、subscribe-show-open） │
└──────────┬───────────────────┬───────────┘
           │ 上行               │ 下行
           ▼                   │
┌──────────────────┐  ┌──────────────────────┐
│  Bridge Emitter  │  │  broadcastToAll()     │
│  (路由到具体      │  │  (所有 bridge.emit()  │
│   bridge handler) │  │   触发广播)            │
└──────────────────┘  └──────────────────────┘
```

### 部署模式

| 模式 | 适配器 | 消息路径 |
|------|--------|---------|
| Electron（桌面端） | `adapter/main.ts` | bridge.emit() → IPC(BrowserWindow) + broadcastToAll(WebSocket) |
| Standalone（纯后端） | `adapter/standalone.ts` | bridge.emit() → broadcastToAll(WebSocket) |

> Standalone 模式是原项目为解耦业务逻辑与 Electron 主进程而引入的中间产物，使后端可脱离 Electron 独立运行。但该模式未经充分验证，可能存在未暴露的问题。Rust 重写时是全新实现，天然只有 WebSocket 通道，不存在 IPC 适配层——梳理接口时以 Electron 模式的完整实现为准，Standalone 仅作架构参考。

### 服务器挂载

WebSocket 服务器与 HTTP 服务器共享同一端口，通过协议升级（HTTP Upgrade）建立连接：

```
HTTP Server (Express)
  ├── REST API routes (/api/*)
  └── WebSocket Server (ws://)
      └── WebSocketManager
```

## 连接生命周期

### 建立连接

```
客户端                          服务端
  │                              │
  │─── HTTP Upgrade (ws://) ────→│
  │    + Token (3 种方式之一)      │
  │                              │── extractWebSocketToken()
  │                              │── validateWebSocketToken()
  │                              │
  │←── [认证失败] auth-expired ──│── ws.close(1008)
  │                              │
  │←── [认证成功] 连接建立 ──────│── addClient(ws, token)
  │                              │── setupMessageHandler()
  │                              │── replay buffered messages
  │                              │
```

### Token 提取（优先级递减）

| 来源 | 格式 | 说明 |
|------|------|------|
| `Authorization` header | `Bearer <token>` | 标准方式 |
| `Cookie` | `aionui-session=<token>` | 浏览器自动携带 |
| `Sec-WebSocket-Protocol` header | `<token>` | 不支持 Cookie 的客户端的备用方案 |

### 认证验证

- 调用 `AuthService.verifyWebSocketToken(token)` 验证 JWT
- JWT 参数：`audience: 'aionui-webui'`，`issuer: 'aionui'`
- 验证 token 黑名单（已登出的 token）
- **当前实现**：WebSocket 复用 Web 登录 session token（`SESSION_EXPIRY: '24h'`），独立的 WebSocket token（`WEBSOCKET_EXPIRY: '5m'`）配置已预留但未启用

### 消息缓冲

连接建立时异步认证（`await validateConnection`）期间，可能有消息先于认证完成到达。实现使用消息缓冲机制：

1. 注册临时 `bufferMessage` 监听器缓存到 `pendingMessages[]`
2. 认证完成后卸载临时监听器，注册正式 handler
3. 重放 `pendingMessages` 中的缓冲消息

### 心跳保活

| 参数 | 值 | 说明 |
|------|-----|------|
| `HEARTBEAT_INTERVAL` | 30 秒 | 服务端发送 ping 的间隔 |
| `HEARTBEAT_TIMEOUT` | 60 秒 | 客户端无 pong 响应则断开 |

**心跳检查流程**（每 30 秒执行一次）：

1. 遍历所有客户端
2. **超时检测**：`now - lastPing > 60s` → 关闭连接（code 1008）
3. **Token 过期检测**：`validateWebSocketToken(token)` → 发送 `auth-expired` 事件 → 关闭连接（code 1008）
4. **发送 ping**：`{ name: "ping", data: { timestamp } }`
5. 客户端回复 `{ name: "pong" }` → 更新 `lastPing`

### 连接关闭

| 场景 | Close Code | 说明 |
|------|-----------|------|
| 正常关闭 | 1000 (`NORMAL_CLOSURE`) | 服务器关闭 |
| 无 Token | 1008 (`POLICY_VIOLATION`) | 连接时未提供 token |
| Token 无效 | 1008 (`POLICY_VIOLATION`) | JWT 验证失败 |
| Token 过期 | 1008 (`POLICY_VIOLATION`) | 心跳检查发现过期 |
| 心跳超时 | 1008 (`POLICY_VIOLATION`) | 60 秒无 pong 响应 |

> Token 过期时先发送 `auth-expired` 事件再关闭，使客户端能跳转登录页而非进入无限重连循环。

## 消息协议

### 消息格式

所有 WebSocket 消息均为 JSON，统一格式：

```json
{
  "name": "event-name",
  "data": { /* 事件数据 */ }
}
```

### 错误响应

客户端发送的消息无法解析时：

```json
{
  "error": "Invalid message format",
  "expected": "{ \"name\": \"event-name\", \"data\": {...} }"
}
```

### 上行消息路由（客户端 → 服务端）

客户端发送的消息按以下优先级处理：

| 优先级 | name | 处理方式 | 说明 |
|--------|------|---------|------|
| 1 | `pong` | WebSocketManager 内部处理 | 心跳响应，更新 `lastPing` |
| 2 | `subscribe-show-open` | WebSocketManager 内部处理 | 文件选择请求，见下文 |
| 3 | 其他所有 | 转发到 Bridge Emitter | 路由到对应 bridge handler |

**Bridge Emitter 路由**：上行消息的 `name` 字段对应 bridge handler 的注册名。例如客户端发送 `{ name: "conversation.send-message", data: {...} }` 会路由到 `conversationBridge` 中注册的 `conversation.send-message` handler。

### 文件选择请求（`subscribe-show-open`）

WebUI 模式下替代 Electron 原生文件对话框。客户端发送文件选择订阅后，服务端解析参数并回发请求：

**客户端 → 服务端**：
```json
{
  "name": "subscribe-show-open",
  "data": {
    "properties": ["openFile"]
  }
}
```

**服务端 → 该客户端**（非广播，单播）：
```json
{
  "name": "show-open-request",
  "data": {
    "properties": ["openFile"],
    "isFileMode": true
  }
}
```

`isFileMode` 判断逻辑：`properties` 包含 `openFile` 且不包含 `openDirectory` 时为 `true`。

> **设计决策**：此功能是 Electron `dialog.showOpenDialog` 在 WebUI 中的替代方案。Rust 重写中保留此机制——后端通知前端"需要用户选择文件"，前端通过自己的文件选择 UI 完成后回传路径。

## 下行事件目录（服务端 → 客户端广播）

所有 bridge 模块通过 `bridge.xxx.yyy.emit(data)` 触发的事件都会广播给**所有** WebSocket 客户端（无过滤）。

### 系统事件（WebSocket 层自身）

| 事件名 | 方向 | 数据 | 说明 |
|--------|------|------|------|
| `ping` | 服务端 → 客户端 | `{ timestamp: number }` | 心跳探测 |
| `pong` | 客户端 → 服务端 | `{}` | 心跳响应 |
| `auth-expired` | 服务端 → 客户端 | `{ message: string }` | Token 过期通知（关闭前发送） |
| `show-open-request` | 服务端 → 单个客户端 | `{ properties, isFileMode }` | 文件选择请求 |

### 会话与聊天事件

| 事件名 | 来源模块 | 数据 | 说明 |
|--------|---------|------|------|
| `chat.response.stream` | conversation | `{ type, data, msg_id, conversation_id, hidden? }` | AI 响应流式消息 |
| `conversation.turn.completed` | conversation | `IConversationTurnCompletedEvent` | 会话回合完成 |
| `conversation.list-changed` | conversation | `{ conversationId, action, source? }` | 会话列表变更 |
| `confirmation.add` | conversation.confirmation | `IConfirmation & { conversation_id }` | 新确认请求 |
| `confirmation.update` | conversation.confirmation | `IConfirmation & { conversation_id }` | 确认请求更新 |
| `confirmation.remove` | conversation.confirmation | `{ conversation_id, id }` | 确认请求移除 |
| `openclaw.response.stream` | openclawConversation | `IResponseMessage` | OpenClaw 响应流 |

### 文件与工作区事件

| 事件名 | 来源模块 | 数据 | 说明 |
|--------|---------|------|------|
| `file-changed` | fileWatch | `{ filePath, eventType }` | 文件系统变更 |
| `file-stream-content-update` | fileStream | `{ filePath, content, workspace, relativePath, operation }` | 文件内容实时更新 |
| `workspace-office-file-added` | workspaceOfficeWatch | `{ filePath, workspace }` | 工作区新增 Office 文件 |

### 文档预览事件

| 事件名 | 来源模块 | 数据 | 说明 |
|--------|---------|------|------|
| `preview.open` | preview | `{ content, contentType, metadata? }` | 打开预览面板 |
| `ppt-preview.status` | pptPreview | `{ state, message? }` | PPT 预览状态 |
| `word-preview.status` | wordPreview | `{ state, message? }` | Word 预览状态 |
| `excel-preview.status` | excelPreview | `{ state, message? }` | Excel 预览状态 |

### 定时任务事件

| 事件名 | 来源模块 | 数据 | 说明 |
|--------|---------|------|------|
| `cron.job-created` | cron | `ICronJob` | 定时任务创建 |
| `cron.job-updated` | cron | `ICronJob` | 定时任务更新 |
| `cron.job-removed` | cron | `{ jobId }` | 定时任务删除 |
| `cron.job-executed` | cron | `{ jobId, status, error? }` | 定时任务执行结果 |

### 扩展与集成事件

| 事件名 | 来源模块 | 数据 | 说明 |
|--------|---------|------|------|
| `extensions.state-changed` | extensions | `{ name, enabled, reason? }` | 扩展启用/禁用 |
| `hub.state-changed` | hub | `{ name, status, error? }` | Hub 扩展安装/更新状态 |
| `channel.pairing-requested` | channel | `IChannelPairingRequest` | 通道配对请求 |
| `channel.plugin-status-changed` | channel | `{ pluginId, status }` | 通道插件状态变更 |
| `channel.user-authorized` | channel | `IChannelUser` | 通道用户授权 |

### 团队协作事件

| 事件名 | 来源模块 | 数据 | 说明 |
|--------|---------|------|------|
| `team.agent.spawned` | team | `ITeamAgentSpawnedEvent` | 团队成员生成 |
| `team.agent.status` | team | `ITeamAgentStatusEvent` | 团队成员状态变更 |
| `team.agent.removed` | team | `ITeamAgentRemovedEvent` | 团队成员移除 |
| `team.agent.renamed` | team | `ITeamAgentRenamedEvent` | 团队成员重命名 |

### 系统管理事件

| 事件名 | 来源模块 | 数据 | 说明 |
|--------|---------|------|------|
| `system-settings:language-changed` | systemSettings | `{ language }` | 语言偏好变更 |
| `webui.status-changed` | webui | `{ running, port?, localUrl?, networkUrl? }` | WebUI 服务状态 |
| `webui.reset-password-result` | webui | `{ success, newPassword?, msg? }` | 密码重置结果 |
| `update.open` | update | `{ source? }` | 打开更新 UI |
| `update.download.progress` | update | `UpdateDownloadProgressEvent` | 更新下载进度 |
| `auto-update.status` | autoUpdate | `AutoUpdateStatus` | 自动更新状态 |
| `notification.clicked` | notification | `{ conversationId? }` | 通知被点击 |
| `deep-link.received` | deepLink | `{ action, params }` | 深度链接接收 |
| `app.devtools-state-changed` | application | `{ isOpen }` | DevTools 状态 |
| `app.log-stream` | application | `{ level, tag, message, data? }` | 日志流 |

> **注意**：`window-controls:maximized-changed` 为 Electron 专属事件，Rust 后端不需要实现。`update.*`、`auto-update.*`、`app.devtools-state-changed`、`notification.clicked`、`deep-link.received` 也需要根据 Rust 后端的实际功能决定是否保留。

## 数据模型

### ClientInfo

WebSocket 客户端连接信息（服务端内部维护）：

```
ClientInfo {
  token: string       // 认证 JWT
  lastPing: number    // 最后心跳时间戳 (ms)
}
```

### WebSocket 消息

```
WebSocketMessage {
  name: string        // 事件名称
  data: any           // 事件数据
}
```

### WebSocket 错误消息

```
WebSocketError {
  error: string       // 错误描述
  expected: string    // 期望的消息格式
}
```

## 配置常量

| 常量 | 值 | 说明 |
|------|-----|------|
| `HEARTBEAT_INTERVAL` | 30000 ms | 心跳发送间隔 |
| `HEARTBEAT_TIMEOUT` | 60000 ms | 心跳超时时间 |
| `CLOSE_CODES.POLICY_VIOLATION` | 1008 | 策略违规关闭码（认证失败、超时） |
| `CLOSE_CODES.NORMAL_CLOSURE` | 1000 | 正常关闭码（服务器关闭） |
| `MAX_IPC_PAYLOAD_SIZE` | 50 MB | 单条消息大小上限（Electron 适配器限制，Rust 可调整） |

## 模块依赖

- **依赖**：
  - `03-auth`：Token 验证（`AuthService.verifyWebSocketToken`）

- **被依赖**：
  - 所有模块：任何调用 `bridge.xxx.emit()` 的模块都通过此通道推送事件
  - `05-conversation`：AI 响应流、确认请求、会话列表变更
  - `08-file-workspace`：文件变更通知
  - `09-channel`：通道配对和状态
  - `10-team`：团队协作事件
  - `11-cron`：定时任务生命周期事件
  - `13-extension`：扩展状态变更
  - `14-app-lifecycle`：WebUI 状态、更新进度
  - `16-office-preview`：文档预览状态

## 候选公共类型

| 类型 | 说明 |
|------|------|
| `WebSocketMessage { name, data }` | 统一消息格式，所有 WebSocket 通信共用 |
| `WebSocketCloseCode` | 关闭码枚举（`NormalClosure=1000`, `PolicyViolation=1008`） |

## Rust 迁移备注

### 技术选型

| 组件 | 建议 | 说明 |
|------|------|------|
| WebSocket 服务端 | `axum` 内置 WebSocket | 与 HTTP 服务共享同一端口，协议升级自动处理 |
| 连接管理 | `DashMap<ConnectionId, ClientInfo>` | 并发安全的客户端列表 |
| 心跳定时器 | `tokio::time::interval` | 30 秒周期检查 |
| 消息序列化 | `serde_json` | JSON 消息编解码 |
| 广播通道 | `tokio::sync::broadcast` | 事件广播给所有客户端 |

### 架构建议

1. **统一消息总线**：使用 `tokio::sync::broadcast` 通道替代原实现中的回调注册模式。各业务模块通过 `broadcast::Sender::send()` 发布事件，WebSocket handler 持有 `broadcast::Receiver` 自动推送给客户端

2. **连接管理**：
   - `DashMap<ConnectionId, ClientInfo>` 存储活跃连接
   - `ConnectionId` 为递增整数或 UUID
   - `ClientInfo` 包含 `token`、`lastPing`、`tx: mpsc::Sender<Message>`（每连接的发送通道）

3. **心跳与清理**：单独的 `tokio::spawn` 任务运行心跳检查循环。使用 `DashMap::retain()` 原子清理超时/过期连接

4. **消息路由**：上行消息通过 `match name { ... }` 分发到对应的 service handler。不再需要 EventEmitter 模式——直接调用 service 方法

5. **背压控制**：原实现的 `broadcastToAll` 无背压控制（慢客户端可能阻塞）。Rust 中每个连接使用 bounded `mpsc` 通道，满时丢弃消息并记录警告

6. **安全增强**：
   - 原实现所有事件广播给所有客户端，无权限过滤。可考虑按用户/会话过滤敏感事件
   - 消息大小限制：原实现 50MB 上限来自 Electron IPC，Rust 中可设置更合理的限制（如 1MB，大文件走 HTTP）
