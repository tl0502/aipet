use crate::services::window_snap;
use crate::state::{AppState, Hitbox};
use tauri::{State, WebviewWindow};

#[tauri::command]
pub fn update_hitbox(x: i32, y: i32, w: i32, h: i32, state: State<'_, AppState>) {
    *state.pet_hitbox.lock().unwrap() = Some(Hitbox { x, y, w, h });
}

#[tauri::command]
pub fn start_drag(state: State<'_, AppState>, window: WebviewWindow) -> Result<(), String> {
    *state.is_dragging.lock().unwrap() = true;
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_drag(state: State<'_, AppState>, window: WebviewWindow) -> Result<(), String> {
    *state.is_dragging.lock().unwrap() = false;
    window_snap::snap_to_edge(&window).map_err(|e| e.to_string())
}
