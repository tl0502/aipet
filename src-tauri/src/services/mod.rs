// 业务服务层
// M1 D2:cursor_tracker(智能穿透)+ window_snap(边缘吸附)
// M1 D3:tray(系统托盘)+ shortcuts(全局快捷键)+ window_actions(共享窗口操作)
// M1 D3:I.2 crypto(Windows DPAPI 封装,为 secrets 表 ciphertext 提供 protect/unprotect)
// M1 D3:H.1 persona(加载内置 momo + 解析 + 写 personas/persona_snapshots)
// M1 D3:F.1 memory(messages 表 CRUD + summary 占位;**默认无限保留**,用户主动清理)
// M1 D3:F.2 nickname(单行 nicknames 表 facade + nickname.changed event)
// M1 D3:DEV-1 dev_window(debug-only,Ctrl+Shift+D 打开 admin/debug 面板)
// M1 D3:B.1 secrets(secrets 表 CRUD + DPAPI 包装;为 LLM provider api_key 持久化)
// M1 D3:B.1 llm(OpenAI 兼容 streaming chat completion + 6 preset)
// M1 D3+:ChatService
pub mod crypto;
pub mod cursor_tracker;
#[cfg(debug_assertions)]
pub mod dev_window;
pub mod llm;
pub mod memory;
pub mod nickname;
pub mod persona;
pub mod secrets;
pub mod shortcuts;
pub mod tray;
pub mod window_actions;
pub mod window_snap;

// 仅测试期编译:DB 集成测试共享 fixture(详 progress/test-coverage-2026-05-04.md)
#[cfg(test)]
pub mod test_db;
