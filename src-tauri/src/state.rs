use std::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub struct Hitbox {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Hitbox {
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    pub fn expand(&self, px: i32) -> Hitbox {
        Hitbox {
            x: self.x - px,
            y: self.y - px,
            w: self.w + 2 * px,
            h: self.h + 2 * px,
        }
    }
}

// 主进程全局状态容器。M1 D2:加 hitbox + is_dragging,供 cursor_tracker 与 window commands 共享。
#[derive(Default)]
pub struct AppState {
    pub pet_hitbox: Mutex<Option<Hitbox>>,
    pub is_dragging: Mutex<bool>,
}
