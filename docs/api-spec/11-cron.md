# 11 - 定时任务

## 概述

定时任务调度系统：用户或 AI Agent 创建定时任务（CronJob），按指定计划（一次性、固定间隔、cron 表达式）自动向指定对话发送消息触发 AI 执行。支持任务启停、立即执行、自动重试、系统休眠恢复、Skill 文件管理等能力。

**源码位置**：`process/services/cron/`、`process/bridge/cronBridge.ts`、`process/task/CronCommandDetector.ts`

> **设计决策**：原实现中 AI 可通过文本命令（`[CRON_CREATE]...[/CRON_CREATE]`）创建定时任务，由 `CronCommandDetector` 从消息中解析。这是老技能系统的遗留机制，新系统已改为通过 prompt / Skill 文件方式。Rust 重写时建议废弃文本命令解析，统一使用结构化接口（REST API 或 MCP 工具调用）。

## 子模块划分

| 子模块 | 原始源码 | Rust 归属建议 |
|--------|---------|--------------|
| 核心调度服务 | `CronService.ts` | `aionui-cron` |
| 数据持久化 | `CronStore.ts`、`SqliteCronRepository.ts` | `aionui-db` |
| 忙碌状态守卫 | `CronBusyGuard.ts` | `aionui-cron`（内部模块） |
| 任务执行器 | `WorkerTaskManagerJobExecutor.ts` | `aionui-cron` |
| Skill 文件管理 | `cronSkillFile.ts` | `aionui-cron` |
| Skill 建议监听 | `SkillSuggestWatcher.ts` | `aionui-cron` |
| 事件发射器 | `IpcCronEventEmitter.ts` | `aionui-cron`（HTTP/WS 路由） |
| 文本命令检测 | `CronCommandDetector.ts` | 废弃（见设计决策） |
| IPC 桥接 | `cronBridge.ts` | `aionui-cron`（HTTP 路由） |

---

## IPC 接口

### 任务管理

| 通道名 | 目标协议 | 参数 | 返回值 | 功能语义 |
|--------|---------|------|--------|---------|
| `cron.list-jobs` | HTTP | 无 | `CronJob[]` | 列出所有定时任务 |
| `cron.list-jobs-by-conversation` | HTTP | `{ conversationId: string }` | `CronJob[]` | 按对话 ID 筛选任务列表 |
| `cron.get-job` | HTTP | `{ jobId: string }` | `CronJob \| null` | 获取单个任务详情 |
| `cron.add-job` | HTTP | `CreateCronJobParams` | `CronJob` | 创建定时任务：生成 ID、计算 nextRunAt、持久化到 DB、启动计时器、发射事件 |
| `cron.update-job` | HTTP | `{ jobId: string, updates: Partial<CronJob> }` | `CronJob` | 更新任务：支持修改 name、enabled、schedule、target 等字段。更新 schedule 或 enabled 时重新计算计时器 |
| `cron.remove-job` | HTTP | `{ jobId: string }` | `void` | 删除任务：停止计时器、删除 DB 记录、清理 Skill 文件、发射事件 |
| `cron.run-now` | HTTP | `{ jobId: string }` | `{ conversationId: string }` | 立即异步执行任务：提前创建/获取目标 conversation，异步发送消息触发 AI，返回目标 conversationId |

### Skill 文件管理

| 通道名 | 目标协议 | 参数 | 返回值 | 功能语义 |
|--------|---------|------|--------|---------|
| `cron.save-skill` | HTTP | `{ jobId: string, content: string }` | `void` | 保存 Skill 文件（SKILL.md）：验证内容有效性后写入任务专属目录 |
| `cron.has-skill` | HTTP | `{ jobId: string }` | `boolean` | 查询任务是否已有 Skill 文件 |

### 关联查询（定义在其他模块，Cron 使用）

| 通道名 | 说明 |
|--------|------|
| `conversation.list-by-cron-job` | 按 `cronJobId` 查询关联对话列表（定义在 conversation 模块） |
| `system-settings:get-cron-notification-enabled` | 获取 Cron 通知开关（定义在 system-settings 模块） |
| `system-settings:set-cron-notification-enabled` | 设置 Cron 通知开关（定义在 system-settings 模块） |

### 事件推送

| 通道名 | 方向 | 载荷 | 功能语义 |
|--------|------|------|---------|
| `cron.job-created` | 服务端 → 客户端 | `CronJob` | 任务创建事件 |
| `cron.job-updated` | 服务端 → 客户端 | `CronJob` | 任务更新事件（含状态变更） |
| `cron.job-removed` | 服务端 → 客户端 | `{ jobId: string }` | 任务删除事件 |
| `cron.job-executed` | 服务端 → 客户端 | `CronJobExecutedEvent` | 任务执行结果事件 |

> **协议映射**：事件推送通过 WebSocket 通道实现（复用 `07-realtime.md`）。

### 对话流中的特殊消息类型

| 消息 `type` | 方向 | 载荷 | 功能语义 |
|-------------|------|------|---------|
| `cron_trigger` | 服务端 → 客户端 | `{ cronJobId, cronJobName, triggeredAt }` | 任务触发时插入对话流，通知前端此消息由定时任务触发 |
| `skill_suggest` | 服务端 → 客户端 | `{ cronJobId, name, description, skillContent }` | 检测到 AI 生成的 SKILL_SUGGEST.md 时推送给前端，由用户决定是否保存为 Skill |
| `tips` | 服务端 → 客户端 | warning 文本 | 错过的任务提示消息（系统休眠导致） |

---

## 核心流程

### 任务创建流程

```
用户创建 / AI Agent 请求创建
    ↓
cron.add-job(CreateCronJobParams)
    ↓
CronService.addJob()
    ├─ 生成 ID: "cron_<uuid>"
    ├─ 计算 nextRunAtMs（基于 schedule）
    ├─ 持久化到 DB（insert）
    ├─ 启动计时器（setTimeout / setInterval / Cron 对象）
    ├─ power.preventSleep()（如果是首个启用任务）
    ├─ emitJobCreated(job)
    └─ 返回 CronJob
```

### 任务触发执行流程

```
计时器到期
    ↓
CronService.tick(jobId)
    ├─ 检查 job.enabled === true
    ├─ executor.isConversationBusy(conversationId)?
    │   ├─ 是 → retryCount++，若 < maxRetries 则 30s 后重试
    │   │       若 >= maxRetries 则标记 lastStatus='skipped'
    │   └─ 否 → 继续执行
    ↓
WorkerTaskManagerJobExecutor.executeJob(job)
    ├─ resolveConversationForJob(job)
    │   ├─ executionMode='existing' → 复用原对话
    │   └─ executionMode='new_conversation' → 创建新对话
    │       （agent 或 workspace 变更时也强制创建新对话）
    ├─ setProcessing(conversationId, true)
    ├─ 构建 prompt：
    │   ├─ existing 模式 → buildExistingConvPrompt()
    │   ├─ new 模式无 skill → buildNewConvPrompt()
    │   ├─ new 模式有 skill → buildNewConvWithSkillPrompt()
    │   └─ new 模式 + Gemini → buildNewConvPromptWithSkillSuggest()
    ├─ 插入 cron_trigger 消息到对话（DB + IPC 广播）
    ├─ taskManager.getOrBuildTask(conversationId, { yoloMode: true })
    ├─ task.sendMessage(prompt)
    ├─ 等待 AI 完成
    ├─ setProcessing(conversationId, false)
    ├─ 更新 job.state: lastRunAtMs, lastStatus='ok', runCount++
    └─ emitJobExecuted(jobId, 'ok')
```

### 系统休眠恢复流程

```
系统从休眠唤醒
    ↓
CronService.handleSystemResume()
    ├─ 遍历所有启用的任务
    ├─ 对每个任务检查 nextRunAtMs
    │   ├─ 已错过（nextRunAtMs < now）→ 立即执行 + 插入 tips 消息
    │   └─ 未错过 → 重新计算计时器
    └─ 更新所有计时器
```

### Skill 建议流程

```
定时任务触发 AI 执行（new_conversation 模式）
    ↓
AI 完成首次回复
    ↓
SkillSuggestWatcher.onFinish(conversationId)
    ├─ 检查 workspace 中 SKILL_SUGGEST.md 是否存在且有效
    ├─ SHA-256 去重（避免重复推送相同内容）
    ├─ 验证内容（拒绝占位符）
    └─ 通过 responseStream 广播 skill_suggest 消息
         ↓
前端收到 skill_suggest
    └─ 展示给用户，用户可选择保存为 Skill
         ↓
cron.save-skill({ jobId, content })
    └─ writeCronSkillFile()：验证 + 写入 SKILL.md
```

---

## 数据模型

### 数据库表 `cron_jobs`

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | TEXT | PK | 任务 ID，格式 `cron_<uuid>` |
| `name` | TEXT | NOT NULL | 任务名称 |
| `enabled` | INTEGER | NOT NULL | 启用状态（0/1） |
| `schedule_kind` | TEXT | NOT NULL | 调度类型：`'at'` / `'every'` / `'cron'` |
| `schedule_value` | TEXT | NOT NULL | 调度值：时间戳 / 毫秒间隔 / cron 表达式 |
| `schedule_tz` | TEXT | | 时区（仅 `cron` 类型使用） |
| `schedule_description` | TEXT | | 人类可读的调度描述 |
| `payload_message` | TEXT | NOT NULL | 触发时发送的消息文本 |
| `execution_mode` | TEXT | NOT NULL, DEFAULT `'existing'` | 执行模式：`'existing'`（复用对话）/ `'new_conversation'`（创建新对话） |
| `agent_config` | TEXT | | JSON：Agent 配置（后端类型、模型、workspace 等） |
| `conversation_id` | TEXT | NOT NULL | 关联的对话 ID |
| `conversation_title` | TEXT | | 对话标题 |
| `agent_type` | TEXT | NOT NULL | AI 后端类型 |
| `created_by` | TEXT | NOT NULL | 创建者：`'user'` / `'agent'` |
| `created_at` | INTEGER | NOT NULL | 创建时间戳 |
| `updated_at` | INTEGER | NOT NULL | 更新时间戳 |
| `next_run_at` | INTEGER | | 下次执行时间戳 |
| `last_run_at` | INTEGER | | 上次执行时间戳 |
| `last_status` | TEXT | | 上次状态：`'ok'` / `'error'` / `'skipped'` / `'missed'` |
| `last_error` | TEXT | | 上次错误信息 |
| `run_count` | INTEGER | NOT NULL, DEFAULT 0 | 总执行次数 |
| `retry_count` | INTEGER | NOT NULL, DEFAULT 0 | 当前重试次数 |
| `max_retries` | INTEGER | NOT NULL, DEFAULT 3 | 最大重试次数 |

**索引**：
- `idx_cron_jobs_conversation` ON `conversation_id`
- `idx_cron_jobs_next_run` ON `next_run_at` WHERE `enabled = 1`
- `idx_cron_jobs_agent_type` ON `agent_type`

**关联索引**（`conversations` 表）：
- `idx_conversations_cron_job_id` ON `json_extract(extra, '$.cronJobId')`

**迁移版本**：v9（建表）、v22（添加 `execution_mode`、`agent_config` 列及关联索引）

---

## 共享类型

### 调度类型

```
CronSchedule =
  | { kind: 'at', atMs: number, description: string }         // 一次性：指定时间戳执行
  | { kind: 'every', everyMs: number, description: string }   // 固定间隔：每 N 毫秒执行
  | { kind: 'cron', expr: string, tz?: string, description: string }  // Cron 表达式
```

### 任务主体

```
CronJob {
  id: string                          // 格式 "cron_<uuid>"
  name: string
  enabled: boolean
  schedule: CronSchedule
  target: {
    payload: { kind: 'message', text: string }
    executionMode?: 'existing' | 'new_conversation'
  }
  metadata: {
    conversationId: string
    conversationTitle?: string
    agentType: string                 // AI 后端类型
    createdBy: 'user' | 'agent'
    createdAt: number
    updatedAt: number
    agentConfig?: CronAgentConfig
  }
  state: {
    nextRunAtMs?: number
    lastRunAtMs?: number
    lastStatus?: 'ok' | 'error' | 'skipped' | 'missed'
    lastError?: string
    runCount: number
    retryCount: number
    maxRetries: number
  }
}
```

### Agent 配置

```
CronAgentConfig {
  backend: string                     // AI 后端类型
  name: string                        // Agent 显示名称
  cliPath?: string                    // CLI 路径（如 Claude CLI）
  isPreset?: boolean                  // 是否为预设 agent
  customAgentId?: string              // 自定义 agent ID
  presetAgentType?: string            // 预设 agent 类型
  mode?: string                       // 运行模式
  modelId?: string                    // 模型标识
  configOptions?: Record<string, string>  // 额外配置选项
  workspace?: string                  // 工作区路径
}
```

### 创建参数

```
CreateCronJobParams {
  name: string
  description?: string
  schedule: CronSchedule
  prompt?: string                     // 触发时的 prompt（新系统用）
  message?: string                    // 触发时的消息（老系统用，与 prompt 二选一）
  conversationId: string
  conversationTitle?: string
  agentType: string
  createdBy: 'user' | 'agent'
  executionMode?: 'existing' | 'new_conversation'
  agentConfig?: CronAgentConfig
}
```

### 事件载荷

```
CronJobExecutedEvent {
  jobId: string
  status: 'ok' | 'error' | 'skipped' | 'missed'
  error?: string
}
```

### 内部类型

```
ConversationState {                   // CronBusyGuard 内部
  isProcessing: boolean
  lastActiveAt: number
}
```

---

## Repository 接口

```
ICronRepository {
  insert(job: CronJob) → void
  update(jobId: string, updates: Partial<CronJob>) → void
  delete(jobId: string) → void
  getById(jobId: string) → CronJob | null
  listAll() → CronJob[]
  listEnabled() → CronJob[]
  listByConversation(conversationId: string) → CronJob[]
  deleteByConversation(conversationId: string) → number   // 返回删除数量
}
```

---

## 关键常量

| 常量 | 值 | 说明 |
|------|---|------|
| 重试间隔 | 30000 (30s) | 对话忙碌时重试的等待时间 |
| 最大重试次数 | 3 | 默认 `maxRetries`，超过后标记 `skipped` |
| 空闲清理阈值 | 3600000 (1h) | `CronBusyGuard.cleanup()` 清理超过 1 小时的空闲状态 |
| idle 等待超时 | 60000 (60s) | `waitForIdle()` 默认超时时间 |
| Skill 文件目录 | `{cronSkillsDir}/{jobId}/SKILL.md` | 每个任务独立的 Skill 文件路径 |
| Skill 建议文件 | `{workspace}/SKILL_SUGGEST.md` | AI 建议的 Skill 内容（工作区内） |

---

## 与其他模块的集成

### 依赖

| 模块 | 依赖方式 |
|------|---------|
| `02-database` | 读写 `cron_jobs` 表，查询 `conversations` 表的 `extra.cronJobId` |
| `04-system-settings` | 读取 Cron 通知开关配置 |
| `05-conversation` | 创建/查询对话、插入消息（cron_trigger / tips / skill_suggest 类型） |
| `06-ai-agent` | 通过 `WorkerTaskManager` 获取或创建 AgentTask 并发送消息触发 AI 执行 |
| `07-realtime` | 事件推送（job-created / job-updated / job-removed / job-executed）通过 WebSocket |

### 被依赖

| 模块 | 依赖方式 |
|------|---------|
| `05-conversation` | 删除对话时级联删除关联的定时任务（`deleteByConversation`） |
| `14-app-lifecycle` | 应用启动时调用 `cronService.init()` 初始化调度；系统休眠恢复时调用 `handleSystemResume()` |

---

## Skill 文件系统

定时任务支持关联 Skill 文件，用于在 `new_conversation` 模式下为 AI 提供持久化的上下文和指令。

### Skill 文件结构

```markdown
---
name: <Skill 名称>
description: <一行描述>
---

<Skill 正文：AI 的详细指令>
```

### Skill 文件操作

| 操作 | 说明 |
|------|------|
| `writeCronSkillFile(jobId, name, description, prompt, scheduleDescription?)` | 构建标准格式并写入 |
| `writeRawCronSkillFile(jobId, rawContent)` | 写入原始内容（先验证格式） |
| `readCronSkillContent(jobId)` | 读取 Skill 内容 |
| `hasCronSkillFile(jobId)` | 检查是否存在 |
| `deleteCronSkillFile(jobId)` | 删除整个任务 Skill 目录 |
| `validateSkillContent(content)` | 验证内容有效性：拒绝占位符文本 |

### Skill 建议机制

`SkillSuggestWatcher` 监听 AI 在工作区中生成的 `SKILL_SUGGEST.md` 文件：

- AI 首次执行定时任务后，追加 prompt 请求其撰写 `SKILL_SUGGEST.md`
- AI 将建议内容写入工作区
- Watcher 检测文件变化，验证内容，SHA-256 去重
- 通过 `responseStream` 广播 `skill_suggest` 消息给前端
- 用户审阅后决定是否保存为正式 Skill

> **设计决策**：Skill 建议机制目前主要为 Gemini 后端设计（因其不支持 MCP 工具）。Rust 重写时如果统一了 MCP 支持，可考虑让 Agent 直接通过 MCP 工具调用 `cron.save-skill` 而非通过文件系统间接传递。

---

## 设计决策

1. **三种调度类型**：`at`（一次性）、`every`（固定间隔）、`cron`（cron 表达式）覆盖了常见的定时需求。Rust 重写时建议使用成熟的 cron 解析库（如 `cron` crate），`every` 类型可用 `tokio::time::interval`，`at` 类型用 `tokio::time::sleep_until`。

2. **执行模式**：`existing` 模式在同一对话中追加消息，保持上下文连续性；`new_conversation` 模式每次创建新对话，适合独立任务。Rust 重写时保留此设计。

3. **忙碌重试机制**：对话正在处理时不发送新消息，而是等待 30 秒后重试，最多 3 次。超过后标记 `skipped` 而非 `error`（表示被跳过而非执行失败）。这避免了消息堆积和并发冲突。

4. **电源管理**：有启用的定时任务时阻止系统休眠（`power.preventSleep()`）。这是 Electron 特有能力，Rust 后端作为服务运行时通常不需要此功能（服务不会被休眠）。如果仍需桌面端支持，可通过平台 API 实现。

5. **孤儿任务清理**：`init()` 时检查每个任务关联的 conversation 是否仍存在，不存在则自动删除任务。这防止了用户删除对话后遗留无效定时任务。

6. **文本命令解析（废弃建议）**：`CronCommandDetector` 从 AI 回复中解析 `[CRON_CREATE]...[/CRON_CREATE]` 等文本命令。这是早期设计，依赖文本解析不可靠且难以扩展。Rust 重写时建议废弃，改为 MCP 工具调用或 REST API。

7. **Skill 文件与 prompt 的关系**：`prompt` 字段（`CreateCronJobParams.prompt`）存储在 DB 中作为基础触发消息；Skill 文件（`SKILL.md`）存储在文件系统中提供更丰富的上下文。执行时优先使用 Skill 文件内容，无 Skill 则使用 prompt。Rust 重写时建议将 Skill 内容也存入 DB，避免文件系统依赖。

---

## 候选公共类型

| 类型 | 说明 | 建议归属 |
|------|------|---------|
| `CronSchedule` | 调度类型枚举（at / every / cron） | `aionui-cron`（导出供前端使用） |
| `CronJob` | 任务完整描述结构体 | `aionui-cron` |
| `CronAgentConfig` | Agent 配置子结构 | `aionui-cron`（与 `06-ai-agent` 的 Agent 配置有重叠，可提取公共部分到 `aionui-common`） |
| `CreateCronJobParams` | 创建参数 DTO | `aionui-api-types` |
| `CronJobExecutedEvent` | 执行结果事件载荷 | `aionui-api-types` |
