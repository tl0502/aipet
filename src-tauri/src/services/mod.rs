// 业务服务层
// M1 D2:cursor_tracker(智能穿透)+ window_snap(边缘吸附)
// M1 D3:tray(系统托盘)+ shortcuts(全局快捷键)+ window_actions(共享窗口操作)
// M1 D3:I.2 crypto(Windows DPAPI 封装,为 secrets 表 ciphertext 提供 protect/unprotect)
// M1 D3+:PersonaService / MemoryService / NicknameService / ChatService
pub mod crypto;
pub mod cursor_tracker;
pub mod shortcuts;
pub mod tray;
pub mod window_actions;
pub mod window_snap;
