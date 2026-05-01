use chrono::Utc;

// 健康检查 — 返回当前 UTC ISO8601 时间戳,验证 IPC 链路通畅
#[tauri::command]
pub fn ping() -> String {
    Utc::now().to_rfc3339()
}
