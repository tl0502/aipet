// DEV-1 IPC commands 元数据 — IpcPlayground 渲染源
//
// 写死的注册表:每个 command 含 name / 描述 / 参数 schema。
// 不从 Rust 端动态发现(Tauri 没暴露 invoke_handler 列表 API)。
// 加新 command 时手动同步本文件。

export type ParamType = 'string' | 'number' | 'boolean' | 'string?' | 'number?'

export interface CommandParam {
  name: string
  type: ParamType
  required: boolean
  description?: string
}

export interface CommandMeta {
  name: string
  description: string
  params: CommandParam[]
  category: 'core' | 'window' | 'nickname' | 'llm' | 'chat' | 'dev'
}

export const COMMAND_REGISTRY: CommandMeta[] = [
  {
    name: 'ping',
    description: '健康检查 — 返回 ISO8601 时间戳',
    params: [],
    category: 'core',
  },
  {
    name: 'update_hitbox',
    description: 'A.3 上报桌宠 hitbox(CSS 像素;主进程转屏幕物理像素)',
    params: [
      { name: 'cssX', type: 'number', required: true },
      { name: 'cssY', type: 'number', required: true },
      { name: 'cssW', type: 'number', required: true },
      { name: 'cssH', type: 'number', required: true },
    ],
    category: 'window',
  },
  {
    name: 'start_drag',
    description: 'A.3 进入拖动状态',
    params: [],
    category: 'window',
  },
  {
    name: 'stop_drag',
    description: 'A.3 退出拖动 + 触发屏幕边缘吸附',
    params: [],
    category: 'window',
  },
  {
    name: 'nickname_get_pet',
    description: 'F.2 读 pet_nickname,fallback 到 active persona name',
    params: [],
    category: 'nickname',
  },
  {
    name: 'nickname_get_user',
    description: 'F.2 读 user_nickname(NULL 时返回 null)',
    params: [],
    category: 'nickname',
  },
  {
    name: 'nickname_set_pet',
    description: 'F.2 设置 pet_nickname(自动备份 previous + emit nickname:changed)',
    params: [{ name: 'name', type: 'string', required: true }],
    category: 'nickname',
  },
  {
    name: 'nickname_set_user',
    description: 'F.2 设置 user_nickname + emit nickname:changed',
    params: [{ name: 'name', type: 'string', required: true }],
    category: 'nickname',
  },
  {
    name: 'nickname_restore_pet',
    description: 'F.2 swap pet_nickname ↔ pet_nickname_previous(可来回切)',
    params: [],
    category: 'nickname',
  },
  // ===== B.1 LLMProvider(架构 §6 / ADR-005)=====
  {
    name: 'llm_list_presets',
    description: 'B.1 列 6 个 provider preset(openai/deepseek/moonshot/qwen/ollama/custom)',
    params: [],
    category: 'llm',
  },
  {
    name: 'secrets_set_api_key',
    description: 'B.1 写入 provider api_key(DPAPI 加密后入 secrets 表)',
    params: [
      { name: 'provider', type: 'string', required: true, description: 'preset id 如 openai' },
      { name: 'key', type: 'string', required: true, description: '明文 api key,不会回显' },
    ],
    category: 'llm',
  },
  {
    name: 'secrets_delete_api_key',
    description: 'B.1 删除 provider api_key(幂等)',
    params: [{ name: 'provider', type: 'string', required: true }],
    category: 'llm',
  },
  {
    name: 'secrets_test',
    description: 'B.1 探活:GET {base_url}/models;返回 { ok, latency_ms, message }',
    params: [
      { name: 'provider', type: 'string', required: true },
      { name: 'baseUrl', type: 'string', required: true, description: '如 https://api.openai.com/v1' },
      { name: 'model', type: 'string', required: true, description: '如 gpt-4o-mini(ping 不验模型)' },
    ],
    category: 'llm',
  },
  {
    name: 'dev_llm_test_stream',
    description: 'B.1 debug-only 端到端 streaming;Events tab 看 dev:llm:token 流',
    params: [
      { name: 'provider', type: 'string', required: true },
      { name: 'baseUrl', type: 'string', required: true },
      { name: 'model', type: 'string', required: true },
      { name: 'userMessage', type: 'string', required: true, description: '如「你好」' },
    ],
    category: 'llm',
  },
  // ===== B.2 ChatService(架构 §3 / ADR-015)=====
  {
    name: 'chat_send',
    description: 'B.2 启动一轮对话:立即返 { messageId, conversationId };流式 emit chat:token/done/error。先 listen events 再 invoke。',
    params: [
      { name: 'conversationId', type: 'string?', required: false, description: '为空自动新建 ULID conversation' },
      { name: 'provider', type: 'string', required: true },
      { name: 'baseUrl', type: 'string', required: true },
      { name: 'model', type: 'string', required: true },
      { name: 'userMessage', type: 'string', required: true, description: '用户本轮输入' },
    ],
    category: 'chat',
  },
  {
    name: 'chat_cancel',
    description: 'B.2 取消进行中的 chat — 通过 message_id 触发 CancellationToken,运行中 stream 早终止',
    params: [{ name: 'messageId', type: 'string', required: true }],
    category: 'chat',
  },
  {
    name: 'chat_history',
    description: 'B.2 列 conversation 内历史消息(默认 limit=50,按 created_at 升序)',
    params: [
      { name: 'conversationId', type: 'string', required: true },
      { name: 'limit', type: 'number?', required: false },
    ],
    category: 'chat',
  },
  {
    name: 'conversation_create',
    description: 'B.2 创建新 conversation(M3 之前最小 helper;返回 { id })',
    params: [
      { name: 'personaId', type: 'string', required: true, description: '通常 momo' },
      { name: 'title', type: 'string?', required: false },
    ],
    category: 'chat',
  },
  {
    name: 'dev_list_tables',
    description: 'DEV-1 列 sqlite 数据库内所有非系统表名',
    params: [],
    category: 'dev',
  },
  {
    name: 'dev_query_table',
    description: 'DEV-1 查表前 N 行(白名单:personas/messages/nicknames/persona_snapshots/conversations)',
    params: [
      { name: 'table', type: 'string', required: true },
      { name: 'limit', type: 'number?', required: false, description: '默认 100,最大 500' },
    ],
    category: 'dev',
  },
  {
    name: 'dev_get_logs',
    description: 'DEV-1 拉日志缓冲(MVP 占位,待 ringbuffer logger 接入)',
    params: [],
    category: 'dev',
  },
]

/// 架构 §711 列出的 IPC events — EventLog tab 订阅这些
/// 注:Tauri 2.x event name 仅允许 [a-zA-Z0-9\-/:_],不允许 `.`
/// 历史 `xxx.yyy` 命名会触发 emit 运行时 panic,2026-05-04 全量改 `:` 分隔
export const TRACKED_EVENTS: readonly string[] = [
  'nickname:changed',
  'persona:activated',
  'network:changed',
  'shortcut:chat',
  'shortcut:boss-key',
  'tray:show',
  'tray:hide',
  'tray:settings',
  // B.1 dev_llm_test_stream(debug-only;release build 不会触发,留这里 dev 期监控)
  'dev:llm:token',
  'dev:llm:done',
  // B.2 ChatService 流式事件(架构 §5.2)
  'chat:token',
  'chat:done',
  'chat:error',
] as const
