# 03 - 认证与用户管理

## 概述

提供 WebUI 的用户注册、登录、登出、Token 管理和访问控制。支持密码认证和 QR 码免密登录两种方式。

**源码位置**：`process/webserver/auth/`、`process/webserver/routes/authRoutes.ts`、`process/bridge/authBridge.ts`

## 架构设计

### 分层结构

```
middleware/   → 请求拦截：Token 提取、验证、安全头、输入校验
service/      → 业务逻辑：密码哈希、JWT 签发/验证、Token 黑名单
repository/   → 数据访问：用户 CRUD、速率限制存储
```

### 认证流程

```
客户端请求
  → 提取 Token（Authorization Header / Cookie）
  → 验证 JWT（签名 + 过期 + 黑名单检查）
  → 查找用户（数据库）
  → 注入 req.user
  → 业务路由
```

### JWT 策略

| 配置项 | 值 | 说明 |
|--------|-----|------|
| 算法 | HS256（jsonwebtoken 默认） | — |
| 过期时间 | 24h | 会话 JWT |
| Cookie 存活 | 30 天 | `aionui-session` Cookie |
| issuer | `aionui` | — |
| audience | `aionui-webui` | HTTP 和 WebSocket 共用 |

**JWT Secret 管理**：

1. 优先使用环境变量 `JWT_SECRET`
2. 其次从数据库读取主用户的 `jwt_secret` 字段
3. 若均不存在，生成 64 字节随机密钥并持久化到数据库

**Token 失效机制**：

- **单 Token 失效**（登出）：将 Token 的 SHA-256 哈希加入内存黑名单，按过期时间自动清理
- **全局失效**（修改密码）：轮换 JWT Secret，使所有已签发 Token 一次性失效

## REST API

### POST /login

用户登录，获取会话 Token。

**中间件**：`authRateLimiter` → `validateLoginInput`

**请求体**：

```json
{
  "username": "string",
  "password": "string"
}
```

**输入校验**：
- `username`、`password` 均为必填、字符串类型
- `username` 最长 32 字符，`password` 最长 128 字符

**成功响应** `200`：

```json
{
  "success": true,
  "message": "Login successful",
  "user": {
    "id": "auth_1712345678_abc",
    "username": "admin"
  },
  "token": "eyJhbGciOiJI..."
}
```

同时设置 `Set-Cookie: aionui-session=<token>`。

**错误响应**：

| 状态码 | 场景 |
|--------|------|
| 400 | 缺少必填字段 / 类型错误 / 长度超限 |
| 401 | 用户名或密码错误 |
| 429 | 触发速率限制（15 分钟内 5 次失败） |
| 500 | 服务器内部错误 |

**安全措施**：
- 用户不存在时执行伪 bcrypt 校验，防止用户名枚举时序攻击
- 密码验证使用常量时间比较 + 最低 50ms 延迟

---

### POST /logout

用户登出，清除会话。

**中间件**：`apiRateLimiter` → `authenticateToken` → `authenticatedActionLimiter`

**需要认证**：是

**成功响应** `200`：

```json
{
  "success": true,
  "message": "Logged out successfully"
}
```

同时清除 `aionui-session` Cookie 并将当前 Token 加入黑名单。

**错误响应**：

| 状态码 | 场景 |
|--------|------|
| 403 | 未认证 / Token 无效 |
| 429 | 触发速率限制 |
| 500 | 服务器内部错误 |

---

### GET /api/auth/status

获取系统认证状态（是否需要初始设置）。

**中间件**：`apiRateLimiter`

**需要认证**：否

**成功响应** `200`：

```json
{
  "success": true,
  "needsSetup": true,
  "userCount": 0,
  "isAuthenticated": false
}
```

- `needsSetup`：系统中不存在已设置密码的用户时为 `true`，前端据此展示注册/设置页面

**错误响应**：

| 状态码 | 场景 |
|--------|------|
| 429 | 触发速率限制 |
| 500 | 服务器内部错误 |

---

### GET /api/auth/user

获取当前登录用户信息。

**中间件**：`apiRateLimiter` → `authenticateToken` → `authenticatedActionLimiter`

**需要认证**：是

**成功响应** `200`：

```json
{
  "success": true,
  "user": {
    "id": "auth_1712345678_abc",
    "username": "admin"
  }
}
```

**错误响应**：

| 状态码 | 场景 |
|--------|------|
| 403 | 未认证 / Token 无效 |
| 429 | 触发速率限制 |
| 500 | 服务器内部错误 |

---

### POST /api/auth/change-password

修改当前用户密码。

**中间件**：`apiRateLimiter` → `authenticateToken` → `authenticatedActionLimiter`

**需要认证**：是

**请求体**：

```json
{
  "currentPassword": "string",
  "newPassword": "string"
}
```

**新密码校验规则**：
- 长度 8~128 字符
- 不得为常见弱密码（`password`、`12345678` 等）

**成功响应** `200`：

```json
{
  "success": true,
  "message": "Password changed successfully"
}
```

修改成功后轮换 JWT Secret，使所有现有会话失效（强制重新登录）。

**错误响应**：

| 状态码 | 场景 |
|--------|------|
| 400 | 缺少字段 / 新密码不满足强度要求 |
| 401 | 当前密码错误 |
| 404 | 用户不存在 |
| 500 | 服务器内部错误 |

---

### POST /api/auth/refresh

刷新会话 Token。

**中间件**：`apiRateLimiter` → `authenticatedActionLimiter`

**请求体**：

```json
{
  "token": "string"
}
```

**成功响应** `200`：

```json
{
  "success": true,
  "token": "eyJhbGciOiJI..."
}
```

**错误响应**：

| 状态码 | 场景 |
|--------|------|
| 400 | 缺少 token |
| 401 | Token 无效或已过期 |
| 500 | 服务器内部错误 |

---

### GET /api/ws-token

获取 WebSocket 连接所需的 Token。

**中间件**：`apiRateLimiter` → `authenticatedActionLimiter`

**需要认证**：是（通过 Header/Cookie 中的 session token 验证）

**成功响应** `200`：

```json
{
  "success": true,
  "wsToken": "eyJhbGciOiJI...",
  "expiresIn": 2592000000
}
```

当前直接复用主会话 Token，不生成独立的 WebSocket Token。

**错误响应**：

| 状态码 | 场景 |
|--------|------|
| 401 | 未认证 / Token 无效 / 用户不存在 |
| 429 | 触发速率限制 |
| 500 | 服务器内部错误 |

---

### POST /api/auth/qr-login

二维码扫码登录验证。

**中间件**：`authRateLimiter`

**需要认证**：否

**请求体**：

```json
{
  "qrToken": "string"
}
```

**流程**：
1. 服务端启动时生成 QR Token 并存入内存，生成包含 Token 的 URL
2. 用户扫描二维码打开 `/qr-login?token=xxx` 页面
3. 页面 JS 调用本接口提交 `qrToken`
4. 服务端验证 Token（存在性、过期、是否已使用、本地网络限制）
5. 验证通过后生成 session Token，设置 Cookie

**成功响应** `200`：

```json
{
  "success": true,
  "user": { "username": "admin" },
  "token": "eyJhbGciOiJI..."
}
```

**QR Token 特性**：
- 有效期 5 分钟
- 一次性使用
- 本地模式下限制只能从局域网访问

**错误响应**：

| 状态码 | 场景 |
|--------|------|
| 400 | 缺少 qrToken |
| 401 | QR Token 无效 / 已过期 / 已使用 |
| 429 | 触发速率限制 |
| 500 | 服务器内部错误 |

---

### GET /qr-login

二维码登录页面（静态 HTML）。

返回内嵌 JavaScript 的 HTML 页面，JS 从 URL 参数读取 `token` 并调用 `POST /api/auth/qr-login`。

## IPC 接口（Electron → 后端）

### googleAuth.status

| 属性 | 值 |
|------|-----|
| 通道 | `googleAuth.status` |
| 目标协议 | 不迁移（Google OAuth 桌面端专属） |
| 参数 | `{ proxy?: string }` |
| 返回 | `{ success: boolean, data?: { account: string }, msg?: string }` |

检查 Google OAuth 登录状态。从缓存/凭证文件读取 OAuth 信息。

### googleAuth.login

| 属性 | 值 |
|------|-----|
| 通道 | `googleAuth.login` |
| 目标协议 | 不迁移（Google OAuth 桌面端专属） |
| 参数 | `{ proxy?: string }` |
| 返回 | `{ success: boolean, data?: { account: string }, msg?: string }` |

执行 Google OAuth 登录流程，带 2 分钟超时。

### googleAuth.logout

| 属性 | 值 |
|------|-----|
| 通道 | `googleAuth.logout` |
| 目标协议 | 不迁移（Google OAuth 桌面端专属） |

清除 OAuth 凭证文件缓存。

> **注意**：`authBridge.ts` 中的三个 Google OAuth IPC 接口依赖 `@office-ai/aioncli-core`（Electron 桌面端的 OAuth 库），属于 Electron 专属功能，Rust 后端**不需要迁移**。Rust 后端的认证完全通过上述 REST API 实现。

## Token 提取策略

### HTTP 请求

按优先级依次尝试：

1. `Authorization: Bearer <token>` Header
2. `aionui-session` Cookie

不再支持 URL query 参数（安全风险：日志泄露、Referrer 泄露）。

### WebSocket 请求

按优先级依次尝试：

1. `Authorization: Bearer <token>` Header
2. `Cookie: aionui-session=<token>`
3. `Sec-WebSocket-Protocol` Header（第一个协议值，用于不支持 Cookie 的客户端）

## 速率限制

| 限制器 | 窗口 | 最大请求数 | 应用范围 | 限流键 |
|--------|------|-----------|---------|-------|
| `authRateLimiter` | 15 分钟 | 5 | 登录/注册/QR 登录 | IP（跳过成功请求） |
| `apiRateLimiter` | 1 分钟 | 60 | 一般 API | IP |
| `authenticatedActionLimiter` | 1 分钟 | 20 | 已认证敏感操作 | 用户 ID（优先）/ IP |

## 安全中间件

### 安全响应头

| Header | 值 | 作用 |
|--------|-----|------|
| `X-Frame-Options` | `DENY` | 防止点击劫持 |
| `X-Content-Type-Options` | `nosniff` | 防止 MIME 嗅探 |
| `X-XSS-Protection` | `1; mode=block` | 启用 XSS 保护 |
| `Referrer-Policy` | `strict-origin-when-cross-origin` | 限制 Referrer 泄露 |
| `Content-Security-Policy` | 见下文 | 内容安全策略 |

CSP 策略在开发和生产环境略有不同（开发环境允许 `unsafe-eval` 以支持 webpack-dev-server）。

### CSRF 防护

- Cookie 名：`aionui-csrf-token`
- Header 名：`x-csrf-token`
- Token 长度：32 字节

### Cookie 配置

| 属性 | 值 | 说明 |
|------|-----|------|
| `httpOnly` | `true` | JS 不可读取 |
| `secure` | 动态 | 仅 HTTPS 环境启用（`AIONUI_HTTPS=true`） |
| `sameSite` | `strict` / `lax` | 远程 HTTP 模式降级为 `lax` |
| `maxAge` | 30 天 | — |

## 数据模型

### AuthUser

认证用户的最小字段集（从完整 `IUser` 中挑选）：

```
AuthUser {
  id: string
  username: string
  password_hash: string
  jwt_secret: string | null
  created_at: number
  updated_at: number
  last_login: number | null
}
```

### TokenPayload

JWT Token 内容：

```
TokenPayload {
  userId: string
  username: string
  iat: number       // 签发时间（JWT 标准字段）
  exp: number       // 过期时间（JWT 标准字段）
}
```

### RateLimitEntry

速率限制条目：

```
RateLimitEntry {
  count: number      // 尝试次数
  resetTime: number  // 重置时间戳
}
```

## UserRepository 接口

| 方法 | 签名 | 说明 |
|------|------|------|
| `hasUsers` | `() → bool` | 系统中是否存在已设置密码的用户 |
| `getSystemUser` | `() → AuthUser?` | 获取系统默认用户 |
| `getPrimaryWebUIUser` | `() → AuthUser?` | 获取主 WebUI 用户（系统用户优先，其次 admin） |
| `setSystemUserCredentials` | `(username, passwordHash) → void` | 设置系统用户凭据（初始引导） |
| `createUser` | `(username, passwordHash) → AuthUser` | 创建新用户 |
| `findByUsername` | `(username) → AuthUser?` | 按用户名查找 |
| `findById` | `(id) → AuthUser?` | 按 ID 查找 |
| `listUsers` | `() → AuthUser[]` | 获取所有用户 |
| `countUsers` | `() → number` | 用户总数 |
| `updatePassword` | `(userId, passwordHash) → void` | 更新密码 |
| `updateUsername` | `(userId, username) → void` | 更新用户名 |
| `updateLastLogin` | `(userId) → void` | 更新最后登录时间 |
| `updateJwtSecret` | `(userId, jwtSecret) → void` | 更新 JWT Secret |

## 初始引导流程

1. 系统启动时，数据库自动创建 `system_default_user` 占位用户（空密码）
2. 前端调用 `GET /api/auth/status`，返回 `needsSetup: true`
3. 前端展示初始设置页面
4. 用户通过以下方式之一完成设置：
   - 调用 `setSystemUserCredentials` 为系统用户设置用户名和密码
   - 通过 QR 码扫码登录（本地模式下）
5. 设置完成后 `needsSetup` 变为 `false`

`AuthService.generateUserCredentials()` 可生成随机用户名（6-8 位字母数字）和强密码（12-17 位，含大小写、数字、特殊字符），用于自动引导场景。

## 密码策略

### 校验规则

- 最小长度：8 字符
- 最大长度：128 字符
- 禁止常见弱密码：`password`、`12345678`、`123456789`、`qwertyui`、`abcdefgh`

### 用户名校验

- 长度：3~32 字符
- 允许字符：`[a-zA-Z0-9_-]`
- 不能以 `-` 或 `_` 开头/结尾

### 哈希算法

- bcrypt，salt rounds = 12

## 模块依赖

- **依赖**：
  - `02-database`：用户数据存储（users 表 CRUD）
  - `process/bridge/webuiQR`：QR 码生成和验证
  - `process/webserver/config/constants`：认证配置常量
  - `process/webserver/middleware/security`：速率限制中间件

- **被依赖**：
  - 几乎所有需要认证的 API 路由（通过 `AuthMiddleware.authenticateToken`）
  - `07-realtime`（WebSocket）：WebSocket Token 验证
  - `14-app-lifecycle`：WebUI 服务器启动时的 QR 登录

## 候选公共类型

| 类型 | 来源 | 说明 |
|------|------|------|
| `ApiResponse<T>` | 全局 | 统一 API 响应信封（成功 + 错误），所有端点共用 |
| `ApiError` | 全局 | 统一错误响应格式（error code + message），替代原实现中 `message`/`error` 字段混用的问题 |
| `TokenPayload` | auth/middleware | JWT 负载结构，WebSocket 模块也使用 |
| 速率限制配置 | constants | 可复用的速率限制参数 |
| Cookie 配置 | constants | 统一的 Cookie 策略 |

> **设计决策**：原实现的错误响应格式不一致（有的用 `message`，有的用 `error`，有的带 `details`）。Rust 重写时统一为标准错误格式，在 `01-common-types.md` 中定义。各端点文档仅列出错误状态码和触发条件，不重复定义响应体结构。

## Rust 迁移备注

1. **JWT 库**：使用 `jsonwebtoken` crate，支持 HS256 签名和标准 claims（iss、aud、exp）
2. **密码哈希**：使用 `bcrypt` crate，保持 salt rounds = 12
3. **Token 黑名单**：可用 `DashMap` 或 `tokio::sync::RwLock<HashMap>` 实现内存黑名单，配合定时清理任务
4. **速率限制**：可使用 `tower` 的 `RateLimit` 中间件或 `governor` crate
5. **Cookie 管理**：使用 `axum-extra` 的 `CookieJar` 或 `tower-cookies`
6. **常量时间比较**：使用 `subtle` crate 的 `ConstantTimeEq` trait
7. **QR 登录**：`webuiQR` 的 Token 内存存储可使用 `DashMap<String, QrTokenData>` + TTL 清理
8. **Google OAuth IPC**：不迁移，Rust 后端不需要桌面端 OAuth 流程
9. **安全响应头**：使用 `tower-http` 的 `SetResponseHeaderLayer` 或自定义 middleware
10. **CSRF**：可使用 `axum-csrf` 或自行实现 Double Submit Cookie 模式
